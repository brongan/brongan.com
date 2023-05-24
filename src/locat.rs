use std::net::IpAddr;

use anyhow::Context;

/// Allows geo-locating IPs and keeps analytics
#[derive(Debug)]
pub struct Locat {
    geoip: maxminddb::Reader<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("maxminddb error: {0}")]
    MaxMindDb(#[from] maxminddb::MaxMindDBError),
}

impl Locat {
    pub fn new(geoip_country_db_path: &str, analytics_db_path: &str) -> Result<Self, Error> {
        Ok(Self {
            geoip: maxminddb::Reader::open_readfile(geoip_country_db_path)?,
        })
    }

    /// Converts an address to an ISO 3166-1 alpha-2 country code
    pub async fn ip_to_iso_code(&self, addr: IpAddr) -> anyhow::Result<&str> {
        Ok(self
            .geoip
            .lookup::<maxminddb::geoip2::Country>(addr)?
            .country
            .context("MaxMindDB missing country for ip")?
            .iso_code
            .context("MaxMindDB missing ISO code")?)
    }

    /// Returns a map of country codes to number of requests
    pub fn get_analytics(&self) -> Vec<(String, u64)> {
        Default::default()
    }
}
