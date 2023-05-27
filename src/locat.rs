use std::net::IpAddr;

use anyhow::Context;
use anyhow::Result;

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
        let iso_code = self
            .geoip
            .lookup::<maxminddb::geoip2::Country>(addr)?
            .country
            .context("MaxMindDB missing country for ip")?
            .iso_code
            .context("MaxMindDB missing ISO code")?;

        if let Err(e) = self.analytics.increment(iso_code) {
            eprintln!("Could not increment analytics: {e}");
        }

        Ok(iso_code)
    }

    /// Returns a map of country codes to number of requests
    pub async fn get_analytics(&self) -> Result<Vec<(String, u64)>> {
        Ok(self.analytics.list()?)
    }
}

#[derive(Debug)]
struct Db {
    path: String,
}

impl Db {
    fn migrate(&self, conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS analytics (
                iso_code TEXT PRIMARY KEY,
                count INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn get_connection(&self) -> Result<rusqlite::Connection, rusqlite::Error> {
        let connection = rusqlite::Connection::open(&self.path)?;
        self.migrate(&connection)?;
        Ok(connection)
    }

    fn list(&self) -> Result<Vec<(String, u64)>, rusqlite::Error> {
        let connection = self.get_connection()?;
        let mut statement = connection.prepare("SELECT iso_code, count FROM analytics")?;
        let mut rows = statement.query([])?;
        let mut analytics = Vec::new();
        while let Some(row) = rows.next()? {
            let iso_code = row.get(0)?;
            let count = row.get(1)?;
            analytics.push((iso_code, count))
        }
        Ok(analytics)
    }

    fn increment(&self, iso_code: &str) -> Result<()> {
        let connection = self.get_connection()?;
        let mut statement = connection.prepare(
            "INSERT INTO analytics (iso_code, count)
             VALUES (?, 1)
             ON CONFLICT (iso_code)
             DO UPDATE SET count = count + 1",
        )?;
        statement.execute([iso_code])?;
        Ok(())
    }
}
