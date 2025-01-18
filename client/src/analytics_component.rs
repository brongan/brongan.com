use gloo_net::http::Request;
use shared::Analytics;
use yew::html;
use yew::html::HtmlResult;
use yew::suspense::use_future;
use yew::Suspense;
use yew::{function_component, Html};

fn to_html(analytics: &Analytics) -> Html {
    html! {
        <tr>
            <th class="ip-cell">{ &analytics.ip_address }</th>
            <th class="path-cell">{ &analytics.path }</th>
            <th class="country-cell">{ &analytics.iso_code }</th>
            <th class="count-cell">{ &analytics.count }</th>
        </tr>
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
            Ok(html! {
                <div class="analytics">
                    <table>
                        <thead>
                            <tr>
                                <th>{"IP Address"}</th>
                                <th>{"URL Path"}</th>
                                <th>{"Country"}</th>
                                <th>{"Count"}</th>
                            </tr>
                        </thead>
                        { analytics }
                    </table>
                </div>
            })
        }
        Err(ref failure) => {
            let message = format!("Failed to get analytics: {}", failure);
            Ok(message.into())
        }
    }
}

#[function_component(AnalyticsComponent)]
pub fn analytics() -> Html {
    let fallback = html! {<div>{"Loading..."}</div>};
    html! {
        <>
            <header class="header">
                <h1 class="title">{ "Analytics" }</h1>
            </header>
            <Suspense {fallback}>
                <AnalyticsContent />
            </Suspense>
        </>
    }
}
