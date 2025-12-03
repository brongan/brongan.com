use app::state::ServerState;
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use opentelemetry::{trace::get_active_span, KeyValue};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use tracing::{info, warn};

fn get_fly_client_addr(headers: &HeaderMap) -> Option<IpAddr> {
    let header = headers.get("fly-client-ip")?;
    let header = header.to_str().ok()?;
    let addr = header.parse::<IpAddr>().ok()?;
    Some(addr)
}

pub async fn record_analytics(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.uri().path() == "/_trunk/ws" {
        return next.run(request).await;
    }
    let path = Path::new(request.uri().path());

    if let Some("wasm" | "js" | "webp" | "png" | "jpg" | "vert" | "scss" | "frag" | "css" | "ico") =
        path.extension().and_then(|os_str| os_str.to_str())
    {
        return next.run(request).await;
    }

    let ip = get_fly_client_addr(&headers).unwrap_or(addr.ip());

    let iso_code = if ip.is_loopback() {
        "DEV"
    } else {
        let iso_code: anyhow::Result<&str> = state.locat.get_iso_code(ip).await;
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
        Ok(_) => info!("{ip} requested {:?}", request.uri()),
        Err(err) => warn!("Failed to record request from {ip}: {}", err),
    }

    next.run(request).await
}
