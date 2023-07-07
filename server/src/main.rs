mod catscii;
mod locat;

use axum::body::{boxed, Body};
use axum::http::{Response, StatusCode};
use axum::{routing::get, Router};
use catscii::{analytics_get, catscii_get};
use clap::Parser;
use locat::Locat;
use opentelemetry_honeycomb::new_pipeline;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::read_to_string;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[clap(name = "server", about = "My server")]
struct Opt {
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,
    #[clap(short = 'p', long = "port", default_value = "8081")]
    port: u16,
    #[clap(long = "static-dir", default_value = "dist/")]
    static_dir: String,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    client: reqwest::Client,
    locat: Arc<Locat>,
}

#[tokio::main]
async fn main() {
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN must be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let (_honeyguard, _tracer) = new_pipeline(
        std::env::var("HONEYCOMB_API_KEY").expect("$HONEYCOMB_API_KEY should be set"),
        "catscii".into(),
    )
    .install()
    .unwrap();

    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let country_db_env_var = "GEOLITE2_COUNTRY_DB";
    let country_db_path =
        std::env::var(country_db_env_var).unwrap_or("db/GeoLite2-Country.mmdb".to_string());

    let analytics_db = std::env::var("ANALYTICS_DB").unwrap_or("db/analytics.db".to_string());

    let state = ServerState {
        locat: Arc::new(Locat::new(&country_db_path, analytics_db).await.unwrap()),
        client: Default::default(),
    };

    let opt = Opt::parse();
    let addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let index = read_to_string(index_path).await.unwrap();

    let app = Router::new()
        .route("/catscii", get(catscii_get))
        .route("/analytics", get(analytics_get))
        .with_state(state)
        .fallback(get(|req| async move {
            match ServeDir::new(&opt.static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => Response::builder()
                            .status(StatusCode::OK)
                            .body(boxed(Body::from(index)))
                            .unwrap(),
                        _ => res.map(boxed),
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .unwrap(),
            }
        }));

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };

    info!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
}
