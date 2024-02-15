use crate::analytics::Analytics;
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
}

#[server(GetAnalytics, "/api", "Url", "analytics")]
pub async fn get_analytics() -> Result<Vec<Analytics>, ServerFnError> {
    let locat = self::ssr::locat()?;
    locat
        .get_analytics()
        .await
        .map_err(|_e| ServerFnError::ServerError("Failed to get locat".into()))
}

#[component]
pub fn analytics_row(analytic: Analytics) -> impl IntoView {
    view! {
        <tr>
            <td>{ analytic.ip_address }</td>
            <td>{ analytic.path }</td>
            <td>{ analytic.iso_code }</td>
            <td>{ analytic.count }</td>
        </tr>
    }
}

#[component]
pub fn analytics_table(analytics: Vec<Analytics>) -> impl IntoView {
    let table = analytics
        .into_iter()
        .map(|analytic| view! { <AnalyticsRow analytic/> })
        .collect_view();
    view! {
        <table>
            <thead>
                <tr>
                    <th class="ip-address">IP Address</th>
                    <th class="path">Path Requested</th>
                    <th class="iso-code">ISO Code</th>
                    <th class="count">Count</th>
                </tr>
            </thead>
            <tbody>
            { table }
            </tbody>
        </table>
    }
}

#[component]
pub fn analytics() -> impl IntoView {
    let analytics = create_resource(|| (), |_| async move { get_analytics().await.unwrap() });
    let (_pending, set_pending) = create_signal(false);

    view! {
        <div>
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match analytics.get() {
                    Some(analytics) => view! { <AnalyticsTable analytics/> }.into_view(),
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }
            }
            </Transition>
        </div>
    }
}

#[component]
pub fn analytics_component() -> impl IntoView {
    view! {
        <header class="header">
            <h1 class="title">{ "Analytics" }</h1>
        </header>
        <div class="analytics">
            <Analytics />
        </div>
    }
}
