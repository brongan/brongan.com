use analytics::record_analytics;
use app::*;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{uri::Scheme, Uri},
    middleware,
    response::Redirect,
    serve, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use opentelemetry_honeycomb::new_pipeline;
use reqwest::StatusCode;
use sentry::ClientInitGuard;
use state::{create_server_state, ServerState};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::net::TcpListener;
use tracing::{info, warn, Level};
use tracing_subscriber::{filter, prelude::*};

mod analytics;
mod fileserv;

#[derive(Parser, Debug)]
#[clap(name = "server", about = "My server")]
struct Opt {
    #[clap(long)]
    ssl_port: Option<u16>,
    #[clap(long)]
    cert_dir: Option<String>,
}

fn sentry_guard(dsn: String) -> ClientInitGuard {
    sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ))
}

#[tokio::main]
async fn main() {
    info!("Starting brongan.com");
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    info!("Creating Server State with options: {leptos_options:?}");
    let app_state = create_server_state(leptos_options.clone(), routes.clone())
        .await
        .expect("Set GEOLITE2_COUNTRY_DB and DB");

    let opt = Opt::parse();
    let _sentry = std::env::var("SENTRY_DSN").map(sentry_guard);
    let (_honeyguard, _tracer) = new_pipeline(
        std::env::var("HONEYCOMB_API_KEY").expect("$HONEYCOMB_API_KEY should be set"),
        "catscii".into(),
    )
    .install()
    .unwrap();

    let filter = filter::Targets::new().with_target("brongan.com", Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || {
                    provide_context(app_state.locat.clone());
                    provide_context(app_state.client.clone());
                }
            },
            move || shell(leptos_options.clone()),
        )
        .fallback(leptos_axum::file_and_error_handler::<ServerState, AnyView>(
            shell,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            record_analytics,
        ))
        .with_state(app_state);

    if let (Some(ssl_port), Some(cert_dir)) = (opt.ssl_port, opt.cert_dir) {
        let https_addr = SocketAddr::from((addr.ip(), ssl_port));
        info!("Binding handlers to sockets: ({addr}, {https_addr})",);
        tokio::spawn(redirect_http_to_https(addr, https_addr));
        info!("HTTPS listening at: {https_addr}");
        axum_server::bind_rustls(https_addr, rustls_config(&PathBuf::from(&cert_dir)).await)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        info!("Binding handler to socket: {}", addr);
        let quit_sig = async {
            _ = tokio::signal::ctrl_c().await;
            warn!("Initiating graceful shutdown");
        };
        serve(
            TcpListener::bind(addr).await.unwrap(),
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
    }
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
