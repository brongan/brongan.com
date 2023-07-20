use crate::ServerState;
use axum::http::Request;
use axum::middleware::Next;
use axum::{
    body::BoxBody,
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
};
use opentelemetry::trace::get_active_span;
use opentelemetry::KeyValue;
use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
    Context,
};
use std::net::SocketAddr;
use std::path::Path;
use tracing::{info, warn};

pub async fn analytics_get(State(state): State<ServerState>) -> Response<BoxBody> {
    let tracer = global::tracer("");
    let span = tracer.start("analytics_get");
    let analytics = state
        .locat
        .get_analytics()
        .with_context(Context::current_with_span(span))
        .await
        .unwrap();
    info!("Received analytics: {:?}", analytics);
    let mut response = String::new();
    use std::fmt::Write;
    _ = writeln!(&mut response, "Analytics:");
    for analytic in analytics {
        _ = writeln!(&mut response, "{analytic}")
    }
    response.into_response()
}

pub async fn record_analytics<B>(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    if request.uri().path() == "/_trunk/ws" {
        return next.run(request).await;
    }
    let path = Path::new(request.uri().path());

    match path.extension().map(|os_str| os_str.to_str()).flatten() {
        Some("wasm" | "js" | "png" | "jpg" | "vert" | "scss" | "frag" | "css") => {
            return next.run(request).await
        }
        _ => (),
    }

    let ip = addr.ip();
    let iso_code = if ip.is_loopback() {
        "DEV"
    } else {
        let iso_code = state.locat.get_iso_code(ip.clone()).await;
        match &iso_code {
            Ok(country) => {
                info!("Received request from {country}");
                get_active_span(|span| {
                    span.set_attribute(KeyValue::new("country", country.to_string()))
                });
                country
            }
            Err(err) => {
                warn!("Could not determine country for IP {addr}: {err}");
                "N/A"
            }
        }
    };

    match state
        .locat
        .record_request(ip, iso_code.to_owned(), request.uri().path().to_owned())
        .await
    {
        Ok(_) => info!("Recorded request from {ip} for {}", request.uri()),
        Err(err) => warn!("Failed to record request from {ip}: {}", err),
    }

    next.run(request).await
}
