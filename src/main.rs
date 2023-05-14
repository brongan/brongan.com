use artem::options::{OptionBuilder, TargetType::HtmlFile};
use axum::{
    body::BoxBody,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use color_eyre::{eyre::eyre, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

async fn get_cat_image() -> Result<Vec<u8>> {
    let api_url = "https://api.thecatapi.com/v1/images/search";

    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }

    let client = reqwest::Client::default();
    let image = client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .pop()
        .ok_or_else(|| eyre!("The cat API returned no images."))?;

    Ok(client
        .get(image.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

async fn get_cat_ascii_art() -> Result<String> {
    let image = image::load_from_memory(&get_cat_image().await?)?;
    Ok(artem::convert(
        image,
        OptionBuilder::new().target(HtmlFile(true, true)).build(),
    ))
}

async fn root_get() -> Response<BoxBody> {
    match get_cat_ascii_art().await {
        Ok(art) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            eprintln!("Something went wrong: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();
    let app = Router::new().route("/", get(root_get));

    let addr = &"127.0.0.1:8080".parse().unwrap();
    info!("Listening on {addr}");
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
