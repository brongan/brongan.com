use anyhow::anyhow;
use image::DynamicImage;
use image::ImageError;
use leptos::prelude::*;
use serde::Deserialize;
use server_fn::error::NoCustomError;

pub fn client() -> Result<reqwest::Client, ServerFnError> {
    use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::ServerError("Reqwest Client missing.".into()))
}

#[server(GetCatscii, "/api")]
pub async fn get_catscii() -> Result<String, ServerFnError> {
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
        .map_err(|e: anyhow::Error| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    let image = download_file(&image, &client)
        .with_context(Context::current_with_span(tracer.start("download_file")))
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    let image: DynamicImage = tracer
        .in_span("image::load_from_memory", |cx| {
            let img = image::load_from_memory(&image)?;
            cx.span()
                .set_attribute(KeyValue::new("width", img.width() as i64));
            cx.span()
                .set_attribute(KeyValue::new("height", img.height() as i64));
            Ok(img)
        })
        .map_err(|e: ImageError| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    let ascii_art = tracer.in_span("artem::convert", |_cx| {
        artem::convert(image, &ConfigBuilder::new().target(HtmlFile).build())
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
        <div class="content">
            <CatsciiAscii/>
        </div>
    }
}

async fn get_cat_url(client: &reqwest::Client) -> anyhow::Result<String> {
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
        .ok_or_else(|| anyhow!("The cat API returned no images."))?
        .url)
}

async fn download_file(url: &str, client: &reqwest::Client) -> anyhow::Result<Vec<u8>> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
