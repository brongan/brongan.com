use std::fmt::Display;
use std::net::IpAddr;

use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
};

use anyhow::Context;
use anyhow::Result;
use tokio_rusqlite::Connection;

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

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] tokio_rusqlite::Error),
}

#[derive(Debug)]
pub struct Analytics {
    ip_address: String,
    path: String,
    iso_code: String,
    count: usize,
}

impl Display for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}/{}: {}",
            self.ip_address, self.iso_code, self.path, self.count
        )
    }
}

impl Locat {
    pub async fn new(
        geoip_country_db_path: &str,
        analytics_db_path: String,
    ) -> Result<Self, Error> {
        let geoip_data = tokio::fs::read(geoip_country_db_path).await?;
        Ok(Self {
            geoip: maxminddb::Reader::from_source(geoip_data)?,
            analytics: Db::create(analytics_db_path).await?,
        })
    }

    pub async fn get_iso_code(&self, addr: IpAddr) -> anyhow::Result<&str> {
        let tracer = global::tracer("");
        tracer.in_span("maxminddb::geoip2::Country::lookup", |_| {
            self.geoip
                .lookup::<maxminddb::geoip2::Country>(addr)?
                .country
                .context("MaxMindDB missing country for ip")?
                .iso_code
                .context("MaxMindDB missing ISO code")
        })
    }

    pub async fn record_request(
        &self,
        addr: IpAddr,
        iso_code: String,
        path: String,
    ) -> Result<(), Error> {
        let tracer = global::tracer("");
        self.analytics
            .increment(&addr, iso_code, path)
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("increment"),
            ))
            .await?;
        Ok(())
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
    connection: Connection,
}

impl Db {
    async fn create(path: String) -> Result<Self, tokio_rusqlite::Error> {
        let connection = Connection::open(path).await?;
        connection
            .call(|connection| {
                // create analytics table
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS analytics (
                ip_address TEXT PRIMARY KEY,
                path TEXT KEY,
                iso_code TEXT,
                count INTEGER NOT NULL
            )",
                    [],
                )?;

                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(Self { connection })
    }

    async fn list(&self) -> Result<Vec<Analytics>, tokio_rusqlite::Error> {
        self.connection
            .call(|connection| {
                let mut statement = connection
                    .prepare("SELECT ip_address, path, iso_code, count FROM analytics")?;
                let mut rows = statement.query([])?;
                let mut analytics = Vec::new();
                while let Some(row) = rows.next()? {
                    let ip_address = row.get(0)?;
                    let path = row.get(1)?;
                    let iso_code = row.get(2)?;
                    let count = row.get(3)?;
                    analytics.push(Analytics {
                        ip_address,
                        path,
                        iso_code,
                        count,
                    });
                }
                Ok(analytics)
            })
            .await
    }

    async fn increment(
        &self,
        ip_address: &IpAddr,
        iso_code: String,
        path: String,
    ) -> Result<(), tokio_rusqlite::Error> {
        let ip_address = ip_address.to_string();
        self.connection
            .call(move |connection| {
                let mut statement = connection.prepare(
                    "INSERT INTO analytics (ip_address, iso_code, path, count)
                VALUES (?, ?, ?, 1)
                ON CONFLICT (ip_address)
                DO UPDATE SET count = count + 1",
                )?;
                statement.execute([&ip_address, &iso_code, &path])?;
                Ok(())
            })
            .await
    }
}
