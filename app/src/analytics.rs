use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Analytics {
    pub ip_address: String,
    pub path: String,
    pub iso_code: String,
    pub count: usize,
}

impl Display for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.ip_address, self.iso_code, self.path, self.count
        )
    }
}

#[server(GetAnalytics, "/api")]
pub async fn get_analytics() -> Result<Vec<Analytics>, ServerFnError> {
    let locat = crate::locat::locat()?;
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
    let analytics = Resource::new(|| (), async move |_| get_analytics().await.unwrap());
    let (_pending, set_pending) = signal(false);

    view! {
        <div>
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match analytics.get() {
                    Some(analytics) => view! { <AnalyticsTable analytics/> }.into_any(),
                    None => view! { <p>"Loading..."</p> }.into_any(),
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
