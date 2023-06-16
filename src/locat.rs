use std::fmt::Display;
use std::net::IpAddr;

use anyhow::Context;
use anyhow::Result;
use mysql::params;
use mysql::prelude::Queryable;
use opentelemetry::global;
use opentelemetry::trace::Tracer;

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
    MySql(#[from] mysql::Error),
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
    pub fn new(geoip_country_db_path: &str, analytics_db_path: &str) -> Result<Self, Error> {
        Ok(Self {
            geoip: maxminddb::Reader::open_readfile(geoip_country_db_path)?,
            analytics: Db {
                path: analytics_db_path.to_string(),
            },
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

        if let Err(e) = tracer.in_span("analytics.increment", |_| {
            self.analytics.increment(&addr, iso_code)
        }) {
            eprintln!("Could not increment analytics: {e}");
        }

        Ok(iso_code)
    }

    /// Returns a map of country codes to number of requests
    pub async fn get_analytics(&self) -> Result<Vec<Analytics>> {
        Ok(self.analytics.list()?)
    }
}

#[derive(Debug)]
struct Db {
    path: String,
}

impl Db {
    fn migrate(&self, conn: &mut mysql::PooledConn) -> Result<(), mysql::Error> {
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS analytics (
                ip_address VARCHAR(255) PRIMARY KEY,
                iso_code VARCHAR(255),
                count INTEGER NOT NULL
            )",
        )?;
        Ok(())
    }

    fn get_connection(&self) -> Result<mysql::PooledConn, mysql::Error> {
        let builder = mysql::OptsBuilder::from_opts(mysql::Opts::from_url(&self.path)?);
        let pool = mysql::Pool::new(builder.ssl_opts(mysql::SslOpts::default()))?;
        let mut connection = pool.get_conn()?;
        self.migrate(&mut connection)?;
        Ok(connection)
    }

    fn list(&self) -> Result<Vec<Analytics>, mysql::Error> {
        let mut connection = self.get_connection()?;
        connection.query_map(
            "SELECT ip_address, iso_code, count FROM analytics",
            |(ip_address, iso_code, count)| Analytics {
                ip_address,
                iso_code,
                count,
            },
        )
    }

    fn increment(&self, ip_address: &IpAddr, iso_code: &str) -> Result<()> {
        let tracer = global::tracer("");
        let ip_address = ip_address.to_string();
        let mut connection = tracer.in_span("get_connection", |_| self.get_connection())?;
        connection.exec_drop(
            "INSERT INTO analytics (ip_address, iso_code, count)
             VALUES (:ip_address, :iso_code, 1)
             ON DUPLICATE KEY UPDATE count = count + 1",
            params! {ip_address, iso_code},
        )?;
        Ok(())
    }
}
