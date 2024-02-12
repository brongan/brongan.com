use crate::analytics::record_analytics;
use crate::fileserve::file_and_error_handler;
use crate::locat::Locat;
use crate::root::Root;
use anyhow::Result;
use artem::util::fatal_error;
use axum::body::Body;
use axum::extract::{FromRef, Host, State};
use axum::handler::HandlerWithoutStateExt;
use axum::http::{Request, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{middleware, serve, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use http::uri::Scheme;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use opentelemetry_honeycomb::new_pipeline;
use sentry::ClientInitGuard;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub client: reqwest::Client,
    pub locat: Arc<Locat>,
}

#[derive(Parser, Debug)]
#[clap(name = "server", about = "My server")]
struct Opt {
    #[clap(long, default_value_t = true)]
    dev: bool,
    #[clap(long)]
    ssl_port: Option<u16>,
    #[clap(long)]
    cert_dir: Option<String>,
}

async fn create_server_state(leptos_options: LeptosOptions) -> Result<ServerState> {
    let country_db_dev_path = "db/GeoLite2-Country.mmdb".to_string();
    let country_db_path = std::env::var("GEOLITE2_COUNTRY_DB").unwrap_or(country_db_dev_path);
    let db = std::env::var("DB").unwrap_or("db/sqlite.db".to_string());
    Ok(ServerState {
        leptos_options,
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

async fn rustls_config(cert_dir: &Path) -> RustlsConfig {
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
    info!("HTTP Redirect listening at: {http}");
    axum_server::bind(http)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}

async fn leptos_routes_handler(State(state): State<ServerState>, req: Request<Body>) -> Response {
    let handler = leptos_axum::render_app_to_stream_with_context(
        state.leptos_options,
        || {},
        || view! {<Root/>},
    );
    handler(req).await.into_response()
}

pub async fn run() {
    info!("Starting brongan.com");
    let opt = Opt::parse();
    info!("Creating Sentry and Honeyguard Hooks.");
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

    // Leptos
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let http_addr = leptos_options.site_addr;
    let routes = generate_route_list(Root);

    info!("Creating Server State with options: {leptos_options:?}");
    let server_state = match create_server_state(leptos_options).await {
        Ok(state) => state,
        Err(e) => {
            error!("{e}");
            fatal_error("TERMINATING. Failed to get initial server state.", None);
        }
    };

    let app = Router::new()
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(server_state.clone())
        .layer(middleware::from_fn_with_state(
            server_state,
            record_analytics,
        ));

    if let (Some(ssl_port), Some(cert_dir)) = (opt.ssl_port, opt.cert_dir) {
        let https_addr = SocketAddr::from((http_addr.ip(), ssl_port));
        info!("Binding handlers to sockets: ({http_addr}, {https_addr})",);
        tokio::spawn(redirect_http_to_https(http_addr.clone(), https_addr));
        info!("HTTPS listening at: {https_addr}");
        axum_server::bind_rustls(https_addr, rustls_config(&PathBuf::from(&cert_dir)).await)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        info!("Binding handler to socket: {}", http_addr);
        let quit_sig = async {
            _ = tokio::signal::ctrl_c().await;
            warn!("Initiating graceful shutdown");
        };
        let listener = TcpListener::bind(http_addr).await.unwrap();
        serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
    }
}
