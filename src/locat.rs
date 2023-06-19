use std::fmt::Display;
use std::net::IpAddr;

use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
};

use anyhow::Context;
use anyhow::Result;
use mysql_async::params;
use mysql_async::prelude::*;
use mysql_async::Pool;

/// Allows geo-locating IPs and keeps analytics
#[derive(Debug)]
pub struct Locat {
    geoip: maxminddb::Reader<Vec<u8>>,
    analytics: Db,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("maxminddb error: {0}")]
    MaxMindDb(#[from] maxminddb::MaxMindDBError),

    #[error("mysql error: {0}")]
    MySql(#[from] mysql_async::Error),
}

#[derive(Debug)]
pub struct Analytics {
    ip_address: String,
    iso_code: String,
    count: usize,
}

impl Display for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}: {}", self.ip_address, self.iso_code, self.count)
    }
}

impl Locat {
    pub async fn new(geoip_country_db_path: &str, analytics_db_path: &str) -> Result<Self, Error> {
        Ok(Self {
            geoip: maxminddb::Reader::open_readfile(geoip_country_db_path)?,
            analytics: Db::create(analytics_db_path.to_string()).await?,
        })
    }

    /// Converts an address to an ISO 3166-1 alpha-2 country code
    pub async fn ip_to_iso_code(&self, addr: IpAddr) -> anyhow::Result<&str> {
        let tracer = global::tracer("");
        let iso_code = tracer.in_span("maxminddb::geoip2::Country::lookup", |_| {
            self.geoip
                .lookup::<maxminddb::geoip2::Country>(addr)?
                .country
                .context("MaxMindDB missing country for ip")?
                .iso_code
                .context("MaxMindDB missing ISO code")
        })?;

        if let Err(e) = self
            .analytics
            .increment(&addr, iso_code)
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("increment"),
            ))
            .await
        {
            eprintln!("Could not increment analytics: {e}");
        }

        Ok(iso_code)
    }

    /// Returns a map of country codes to number of requests
    pub async fn get_analytics(&self) -> Result<Vec<Analytics>> {
        let tracer = global::tracer("");
        Ok(self
            .analytics
            .list()
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("list"),
            ))
            .await?)
    }
}

#[derive(Debug)]
struct Db {
    pool: Pool,
}

impl Db {
    pub async fn create(path: String) -> Result<Db, mysql_async::Error> {
        let builder = mysql_async::OptsBuilder::from_opts(mysql_async::Opts::from_url(&path)?);
        let pool = mysql_async::Pool::new(builder.ssl_opts(mysql_async::SslOpts::default()));
        let db = Db { pool };
        let mut connection = db.pool.get_conn().await?;

        "CREATE TABLE IF NOT EXISTS analytics (
                ip_address VARCHAR(255) PRIMARY KEY,
                iso_code VARCHAR(255),
                count INTEGER NOT NULL
            )"
        .ignore(&mut connection)
        .await?;
        Ok(db)
    }

    async fn list(&self) -> Result<Vec<Analytics>, mysql_async::Error> {
        let mut connection = self.pool.get_conn().await?;

        "SELECT ip_address, iso_code, count FROM analytics"
            .with(())
            .map(&mut connection, |(ip_address, iso_code, count)| Analytics {
                ip_address,
                iso_code,
                count,
            })
            .await
    }

    async fn increment(&self, ip_address: &IpAddr, iso_code: &str) -> Result<()> {
        let tracer = global::tracer("");
        let ip_address = ip_address.to_string();
        let mut connection = self
            .pool
            .get_conn()
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("get_connection"),
            ))
            .await?;
        Ok("INSERT INTO analytics (ip_address, iso_code, count)
             VALUES (:ip_address, :iso_code, 1)
             ON DUPLICATE KEY UPDATE count = count + 1"
            .with(params! {ip_address, iso_code})
            .ignore(&mut connection)
            .await?)
    }
}
