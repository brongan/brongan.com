use leptos::{component, create_resource, server, view, IntoView, ServerFnError, SignalGet};
use shared::Analytics;

#[server(GetAnalytics, "/api", "Url", "analytics")]
pub async fn get_analytics() -> Result<Analytics, ServerFnError> {
    todo!()
}

#[component]
pub fn analytics_content(analytics: Analytics) -> impl IntoView {
    view! {
        <p>
            { &analytics.ip_address }
            { &analytics.path }
            { &analytics.iso_code }
            { &analytics.count }
        </p>
    }
}

#[component]
pub fn analytics_component() -> impl IntoView {
    let once = create_resource(|| (), |_| async move { get_analytics().await.unwrap() });
    view! {
        <header class="header">
            <h1 class="title">{ "Analytics" }</h1>
        </header>
        <div class="analytics"> {
                move || match once.get() {
                    Some(analytics) => view! { <AnalyticsContent analytics/> }.into_view(),
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }
            }
        </div>
    }
}
