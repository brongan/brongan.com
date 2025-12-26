#[cfg(feature = "ssr")]
use image::DynamicImage;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum CatsciiError {
    #[error("cat api error: {0}")]
    ApiError(String),
    #[error("cat api did not return cats")]
    NoCatsFound,
    #[error("failed to decode image")]
    ImageError,
}

impl FromStr for CatsciiError {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CatsciiError::ApiError(s.to_string()))
    }
}

#[cfg(feature = "ssr")]
pub fn client() -> Result<reqwest::Client, ServerFnError<CatsciiError>> {
    use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::ServerError("Reqwest Client missing.".into()))
}

#[server(GetCatscii, "/api")]
pub async fn get_catscii() -> Result<String, ServerFnError<CatsciiError>> {
    let client = client()?;
    use artem::config::TargetType::HtmlFile;
    use artem::ConfigBuilder;
    use opentelemetry::{
        global,
        trace::{FutureExt, TraceContextExt, Tracer},
        Context, KeyValue,
    };

    let tracer = global::tracer("");
    let image = get_cat_url(&client)
        .with_context(Context::current_with_span(tracer.start("get_cat_url")))
        .await
        .map_err(|e| CatsciiError::ApiError(e.to_string()))?
        .ok_or(CatsciiError::NoCatsFound)?;

    let image = download_file(&image, &client)
        .with_context(Context::current_with_span(tracer.start("download_file")))
        .await
        .map_err(|e: reqwest::Error| CatsciiError::ApiError(e.to_string()))?;

    let image: DynamicImage = tracer
        .in_span("image::load_from_memory", |cx| {
            let img = image::load_from_memory(&image)?;
            cx.span()
                .set_attribute(KeyValue::new("width", img.width() as i64));
            cx.span()
                .set_attribute(KeyValue::new("height", img.height() as i64));
            Ok(img)
        })
        .map_err(|_e: image::ImageError| CatsciiError::ImageError)?;

    let ascii_art = tracer.in_span("artem::convert", |_cx| {
        artem::convert(
            image,
            &ConfigBuilder::new()
                .target(HtmlFile)
                .background_color(true)
                .build(),
        )
    });
    Ok(ascii_art)
}

#[component]
pub fn catscii_ascii() -> impl IntoView {
    let cats = Resource::new(|| (), async move |_| get_catscii().await.unwrap());
    let (_pending, set_pending) = signal(false);

    view! {
        <div class="ascii-art">
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match cats.get() {
                    Some(html) => view! { <div inner_html=html/> }.into_any(),
                    None => view! { <p>"Loading..."</p> }.into_any(),
                }
            }
            </Transition>
        </div>
    }
}

#[component]
pub fn catscii() -> impl IntoView {
    view! {
        <header class="header">
            <h1 class="title">{ "Catscii" }</h1>
        </header>
        <div class="catscii-container">
            <CatsciiAscii/>
        </div>
    }
}

#[cfg(feature = "ssr")]
async fn get_cat_url(client: &reqwest::Client) -> Result<Option<String>, reqwest::Error> {
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
        .map(|x| x.url))
}

#[cfg(feature = "ssr")]
async fn download_file(url: &str, client: &reqwest::Client) -> Result<Vec<u8>, reqwest::Error> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
