use gloo_net::http::Request;
use shared::Analytics;
use yew::html;
use yew::html::HtmlResult;
use yew::suspense::use_future;
use yew::Suspense;
use yew::{function_component, Html};

fn to_html(analytics: &Analytics) -> Html {
    html! {
                <p>
                    { &analytics.ip_address }
                    { &analytics.path }
                    { &analytics.iso_code }
                    { &analytics.count }
                </p>
    }
}

#[function_component(AnalyticsContent)]
fn analytics_content() -> HtmlResult {
    let resp = use_future(|| async {
        Request::get("/api/analytics")
            .send()
            .await?
            .json::<Vec<Analytics>>()
            .await
    })?;
    match *resp {
        Ok(ref res) => {
            let analytics: Html = res.iter().map(to_html).collect();
            let html = html! {
                <>
                    <header class="header">
                        <h1 class="title">{ "Analytics" }</h1>
                    </header>
                    <div class="analytics">
                        { analytics }
                    </div>
                </>
            };
            Ok(html.into())
        }
        Err(ref failure) => {
            let message = format!("Failed to get analytics: {}", failure.to_string());
            Ok(message.into())
        }
    }
}

#[function_component(AnalyticsComponent)]
pub fn analytics() -> Html {
    let fallback = html! {<div>{"Loading..."}</div>};
    html! {
        <Suspense {fallback}>
            <AnalyticsContent />
        </Suspense>
    }
}
