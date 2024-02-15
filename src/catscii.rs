use crate::server::ServerState;
use artem::options::{OptionBuilder, TargetType::HtmlFile};
use axum::{
    body::Body,
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap},
    response::{IntoResponse, Response},
};
use color_eyre::{eyre::eyre, Result};
use http::{header, StatusCode};
use image::DynamicImage;
use opentelemetry::{
    global,
    trace::{get_active_span, FutureExt, Span, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use serde::Deserialize;

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

pub async fn get_cat_ascii_art(client: &reqwest::Client) -> Result<String> {
    let tracer = global::tracer("");
    let image = get_cat_url(client)
        .with_context(Context::current_with_span(tracer.start("get_cat_url")))
        .await?;
    let image = download_file(&image, client)
        .with_context(Context::current_with_span(tracer.start("download_file")))
        .await?;

    let image: DynamicImage = tracer.in_span("image::load_from_memory", |cx| {
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

async fn root_get_inner(client: &reqwest::Client) -> Response<Body> {
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

pub async fn catscii_get(headers: HeaderMap, State(state): State<ServerState>) -> Response<Body> {
    let tracer = global::tracer("");
    let mut span = tracer.start("root_get");
    span.set_attribute(KeyValue::new(
        "user_agent",
        headers
            .get(header::USER_AGENT)
            .map(|h| h.to_str().unwrap_or_default().to_owned())
            .unwrap_or_default(),
    ));

    root_get_inner(&state.client)
        .with_context(Context::current_with_span(span))
        .await
}
