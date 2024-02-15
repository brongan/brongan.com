use leptos::*;

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::locat::Locat;
    use leptos::*;
    use std::sync::Arc;

    pub fn locat() -> Result<Arc<Locat>, ServerFnError> {
        use_context::<Arc<Locat>>()
            .ok_or_else(|| ServerFnError::ServerError("Locat missing.".into()))
    }

    pub fn client() -> Result<reqwest::Client, ServerFnError> {
        use_context::<reqwest::Client>()
            .ok_or_else(|| ServerFnError::ServerError("Reqwest Client missing.".into()))
    }
}

#[server(GetCatscii, "/api", "Url", "catscii")]
pub async fn get_catscii() -> Result<String, ServerFnError> {
    let client = self::ssr::client()?;
    crate::catscii::get_cat_ascii_art(&client)
        .await
        .map_err(|_e| ServerFnError::ServerError("Failed to get catscii art".into()))
}

#[component]
fn catscii_content(html: String) -> impl IntoView {
    view! { <div inner_html=html/> }
}

#[component]
pub fn catscii() -> impl IntoView {
    let once = create_resource(|| (), |_| async move { get_catscii().await.unwrap() });
    view! {
        <header class="header">
            <h1 class="title">{ "Catscii" }</h1>
        </header>
        <div class="content"> {
                move || match once.get() {
                    Some(html) => view! { <CatsciiContent html/> }.into_view(),
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }
            }
        </div>
    }
}
