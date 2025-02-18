use analytics::record_analytics;
use app::*;
use axum::{
    http::{uri::Scheme, Request, StatusCode, Uri},
    middleware::{self, Next},
    response::Response,
    serve, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use opentelemetry_honeycomb::new_pipeline;
use sentry::ClientInitGuard;
use state::{create_server_state, ServerState};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::net::TcpListener;

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
    log!("Starting brongan.com");
    let leptos_options =
        get_configuration(None).map_or(LeptosOptions::default(), |conf| conf.leptos_options);
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    log!("Creating Server State with options: {leptos_options:?}");
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
        log!("Binding handlers to sockets: ({addr}, {https_addr})",);
        tokio::spawn(http_server(addr));
        log!("HTTPS listening at: {https_addr}");
        axum_server::bind_rustls(https_addr, rustls_config(&PathBuf::from(&cert_dir)).await)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        log!("Binding handler to socket: {}", addr);
        let quit_sig = async {
            _ = tokio::signal::ctrl_c().await;
            log!("Initiating graceful shutdown");
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

async fn http_server(addr: SocketAddr) {
    let app = Router::new().layer(axum::middleware::from_fn(redirect_http_to_https));

    println!("http listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn redirect_http_to_https<B>(req: Request<B>, _next: Next) -> Response {
    let uri = req.uri();
    let mut parts = uri.clone().into_parts();
    parts.scheme = Some(Scheme::HTTPS);

    let port = match uri.port_u16() {
        Some(80 | 443) => uri.port_u16(),
        _ => None,
    };

    let host = parts.authority.as_ref().map_or("", |auth| auth.host());
    let path_and_query = parts.path_and_query.as_ref().map_or("/", |pq| pq.as_str());

    let new_uri = match port {
        Some(p) => format!("https://{}:{}{}", host, p, path_and_query),
        None => format!("https://{}{}", host, path_and_query),
    }
    .parse::<Uri>()
    .unwrap();

    Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header("Location", new_uri.to_string())
        .body(().into())
        .unwrap()
}
