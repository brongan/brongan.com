use crate::locat::Error;
use crate::locat::Locat;
use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use leptos_axum::AxumRouteListing;
use std::sync::Arc;

#[derive(FromRef, Debug, Clone)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub client: reqwest::Client,
    pub locat: Arc<Locat>,
    pub routes: Vec<AxumRouteListing>,
}

pub async fn create_server_state(
    leptos_options: LeptosOptions,
    routes: Vec<AxumRouteListing>,
) -> Result<ServerState, Error> {
    let country_db_dev_path = "db/GeoLite2-Country.mmdb".to_string();
    let country_db_path = std::env::var("GEOLITE2_COUNTRY_DB").unwrap_or(country_db_dev_path);
    let db = std::env::var("DB").unwrap_or("db/sqlite.db".to_string());
    Ok(ServerState {
        leptos_options,
        locat: Arc::new(Locat::new(&country_db_path, db).await?),
        client: Default::default(),
        routes,
    })
}
