use leptos::*;

#[cfg(feature = "ssr")]
pub mod ssr {
    use leptos::*;

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
pub fn catscii_ascii() -> impl IntoView {
    let cats = create_resource(|| (), |_| async move { get_catscii().await.unwrap() });
    let (_pending, set_pending) = create_signal(false);

    view! {
        <div>
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match cats.get() {
                    Some(html) => view! { <div inner_html=html/> }.into_view(),
                    None => view! { <p>"Loading..."</p> }.into_view(),
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
