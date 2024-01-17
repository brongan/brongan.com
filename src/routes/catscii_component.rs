use leptos::{component, create_resource, server, view, IntoView, ServerFnError, SignalGet};

#[server(GetCatscii, "/api", "Url", "catscii")]
pub async fn get_catscii() -> Result<String, ServerFnError> {
    todo!()
}

#[component]
fn catscii_content(art: String) -> impl IntoView {
    view! { <p>{art}</p> }
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
                    Some(art) => view! { <CatsciiContent art/> }.into_view(),
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }
            }
        </div>
    }
}
