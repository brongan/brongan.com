mod analytics;
mod catscii;
mod locat;

use analytics::{analytics_get, record_analytics};
use anyhow::Result;
use axum::body::{boxed, Body};
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{Response as HttpResponse, StatusCode};
use axum::middleware;
use axum::response::{Redirect, Response};
use axum::routing::get;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use catscii::catscii_get;
use clap::Parser;
use hyper::http::uri::Scheme;
use hyper::Uri;
use locat::Locat;
use opentelemetry_honeycomb::new_pipeline;
use sentry::ClientInitGuard;
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
    #[clap(short, long = "addr")]
    addr: Option<String>,
    #[clap(short, long, default_value = "80")]
    port: u16,
    #[clap(long, default_value = "443")]
    ssl_port: u16,
    #[clap(long, default_value = "")]
    static_dir: String,
    #[clap(long, default_value = "cert/")]
    cert_dir: String,
    #[clap(long)]
    dev: bool,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    client: reqwest::Client,
    locat: Arc<Locat>,
}

async fn get_server_state() -> Result<ServerState> {
    let country_db_dev_path = "db/GeoLite2-Country.mmdb".to_string();
    let country_db_path = std::env::var("GEOLITE2_COUNTRY_DB").unwrap_or(country_db_dev_path);
    let db = std::env::var("DB").unwrap_or("db/sqlite.db".to_string());
    Ok(ServerState {
        locat: Arc::new(Locat::new(&country_db_path, db).await?),
        client: Default::default(),
    })
}

fn sentry_guard() -> Result<ClientInitGuard> {
    Ok(sentry::init((
        std::env::var("SENTRY_DSN")?,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    )))
}

async fn rustls_config(cert_dir: &PathBuf) -> RustlsConfig {
    let cert_path = cert_dir.join("fullchain.pem");
    let privkey_path = cert_dir.join("privkey.pem");
    match RustlsConfig::from_pem_file(&cert_path, &privkey_path).await {
        Ok(config) => config,
        Err(err) => panic!(
            "Failed to read cert/key at {}, {}: {err}",
            cert_path.display(),
            privkey_path.display()
        ),
    }
}

async fn https_server(opt: Opt, server_state: ServerState, listen_address: SocketAddr) {
    info!("Starting with: {opt:?}");
    let static_dir = PathBuf::from(&opt.static_dir);
    let cert_dir = PathBuf::from(&opt.cert_dir);
    let index_path = static_dir.join("index.html");
    let index = match read_to_string(&index_path).await {
        Ok(index) => index,
        Err(err) => panic!("Failed to read index at {}: {err}", index_path.display()),
    };

    let api = Router::new()
        .route("/catscii", get(catscii_get))
        .route("/analytics", get(analytics_get));
    let app = Router::new()
        .nest("/api", api)
        .with_state(server_state.clone())
        .fallback(get(|req| async move {
            match ServeDir::new(&opt.static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => HttpResponse::builder()
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
        }))
        .layer(middleware::from_fn_with_state(
            server_state,
            record_analytics,
        ));

    info!("https listening on: {listen_address}");
    axum_server::bind_rustls(listen_address, rustls_config(&cert_dir).await)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn redirect_http_to_https(http: SocketAddr, https: SocketAddr) {
    fn make_https(host: String, uri: Uri, http: u16, https: u16) -> Result<Uri> {
        let mut parts = uri.into_parts();
        parts.scheme = Some(Scheme::HTTPS);
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }
        let https_host = host.replace(&http.to_string(), &https.to_string());
        parts.authority = Some(https_host.parse()?);
        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, http.port(), https.port()) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };
    info!("Http listening on : {http}");
    axum_server::bind(http)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    let _guard = if opt.dev {
        None
    } else {
        Some(sentry_guard().unwrap())
    };
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
    let server_state = get_server_state().await.unwrap();

    let addr = if let Some(ip) = &opt.addr {
        IpAddr::from_str(&ip).unwrap()
    } else {
        IpAddr::V6(Ipv6Addr::LOCALHOST)
    };
    let http_addr = SocketAddr::from((addr, opt.port));
    let https_addr = SocketAddr::from((addr, opt.ssl_port));
    tokio::spawn(redirect_http_to_https(http_addr, https_addr));
    https_server(opt, server_state, https_addr).await;
}
