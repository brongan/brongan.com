use std::fmt::Display;
use std::net::IpAddr;

use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
};

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

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

    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
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
    path: String,
}

impl Db {
    fn create(&self, path: String) -> Result<Db, rusqlite::Error> {
        let db = Db { path };
        let connection = Connection::open(&self.path)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS analytics (
                ip_address TEXT PRIMARY KEY,
                iso_code TEXT,
                count INTEGER NOT NULL
            )",
            [],
        )?;
        Ok((db))
    }

    async fn list(&self) -> Result<Vec<Analytics>, rusqlite::Error> {
        let connection = self.get_connection()?;
        let mut statement =
            connection.prepare("SELECT ip_address, iso_code, count FROM analytics")?;
        let mut rows = statement.query([])?;
        let mut analytics = Vec::new();
        while let Some(row) = rows.next()? {
            let iso_code = row.get(0)?;
            let count = row.get(1)?;
            analytics.push((iso_code, count))
        }
        Ok(analytics)
    }

    async fn increment(&self, ip_address: &IpAddr, iso_code: &str) -> Result<()> {
        let ip_address = ip_address.to_string();
        let connection = self.get_connection()?;
        let mut statement = connection.prepare(
            "INSERT INTO analytics (ip_address, iso_code, count)
             VALUES (?, ?, 1)
             ON CONFLICT (ip_address)
             DO UPDATE SET count = count + 1",
        )?;
        statement.execute([&ip_address, iso_code])?;
        Ok(())
    }
}
