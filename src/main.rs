mod locat;
use anyhow::anyhow;
use artem::options::{OptionBuilder, TargetType::HtmlFile};
use axum::{
    body::BoxBody,
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use color_eyre::{eyre::eyre, Result};
use futures::future::join;
use locat::Locat;
use opentelemetry::{
    global,
    trace::{get_active_span, FutureExt, Span, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_honeycomb::new_pipeline;
use reqwest::{header, StatusCode};
use serde::Deserialize;
use std::{net::IpAddr, str::FromStr, sync::Arc};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug)]
struct ServerState {
    client: reqwest::Client,
    locat: Arc<Locat>,
}

fn get_client_addr(headers: &HeaderMap) -> anyhow::Result<IpAddr> {
    let header = headers
        .get("fly-client-ip")
        .ok_or(anyhow!("Missing fly-client-ip"))?;
    let header = header.to_str()?;
    Ok(header.parse::<IpAddr>()?)
}

async fn get_cat_url(client: &reqwest::Client) -> Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }

    Ok(client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .pop()
        .ok_or_else(|| eyre!("The cat API returned no images."))?
        .url)
}

async fn download_file(url: &str, client: &reqwest::Client) -> Result<Vec<u8>> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

async fn get_cat_ascii_art(client: &reqwest::Client) -> Result<String> {
    let tracer = global::tracer("");
    let image = get_cat_url(&client)
        .with_context(Context::current_with_span(tracer.start("get_cat_url")))
        .await?;
    let image = download_file(&image, &client)
        .with_context(Context::current_with_span(tracer.start("download_file")))
        .await?;

    let image = tracer.in_span("image::load_from_memory", |cx| {
        let img = image::load_from_memory(&image)?;
        cx.span()
            .set_attribute(KeyValue::new("width", img.width() as i64));
        cx.span()
            .set_attribute(KeyValue::new("height", img.height() as i64));
        Ok::<_, color_eyre::eyre::Report>(img)
    })?;

    let ascii_art = tracer.in_span("artem::convert", |_cx| {
        artem::convert(
            image,
            OptionBuilder::new().target(HtmlFile(true, true)).build(),
        )
    });
    Ok(ascii_art)
}

async fn root_get_inner(client: &reqwest::Client) -> Response<BoxBody> {
    match get_cat_ascii_art(client).await {
        Ok(art) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            get_active_span(|span| {
                span.set_status(Status::Error {
                    description: format!("{e}").into(),
                })
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {e}!"),
            )
                .into_response()
        }
    }
}

async fn get_iso_code(headers: &HeaderMap, locat: &Locat) {
    if let Ok(addr) = get_client_addr(&headers) {
        let iso_code = locat.ip_to_iso_code(addr).await;
        match iso_code {
            Ok(country) => {
                info!("Got request from {country}");
                get_active_span(|span| {
                    span.set_attribute(KeyValue::new("country", country.to_string()))
                });
            }
            Err(err) => warn!("Could not determine country for IP address: {err}"),
        }
    } else {
        info!("Failed to get client ip. Are we running locally?");
    }
}

async fn root_get(headers: HeaderMap, State(state): State<ServerState>) -> Response<BoxBody> {
    let tracer = global::tracer("");
    let mut span = tracer.start("root_get");
    span.set_attribute(KeyValue::new(
        "user_agent",
        headers
            .get(header::USER_AGENT)
            .map(|h| h.to_str().unwrap_or_default().to_owned())
            .unwrap_or_default(),
    ));

    let (response, _) = join(
        root_get_inner(&state.client),
        get_iso_code(&headers, &state.locat),
    )
    .with_context(Context::current_with_span(span))
    .await;
    response
}

async fn analytics_get(State(state): State<ServerState>) -> Response<BoxBody> {
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
    let country_db_path = std::env::var(country_db_env_var)
        .unwrap_or_else(|_| panic!("${country_db_env_var} must be set"));

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found");

    let state = ServerState {
        locat: Arc::new(Locat::new(&country_db_path, &db_url).await.unwrap()),
        client: Default::default(),
    };

    let app = Router::new()
        .route("/", get(root_get))
        .route("/analytics", get(analytics_get))
        .route("/panic", get(|| async { panic!("This is a test panic") }))
        .with_state(state);

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };

    let addr = &"0.0.0.0:8080".parse().unwrap();
    info!("Listening on {addr}");
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
}
