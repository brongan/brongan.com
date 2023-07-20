use gloo_net::http::Request;
use yew::html;
use yew::html::HtmlResult;
use yew::suspense::use_future;
use yew::Suspense;
use yew::{function_component, AttrValue, Html};

#[function_component(CatsciiContent)]
fn catscii_content() -> HtmlResult {
    let resp = use_future(|| async { Request::get("/api/catscii").send().await?.text().await })?;
    match *resp {
        Ok(ref res) => Ok(Html::from_html_unchecked(AttrValue::from(res.clone()))),
        Err(ref failure) => Ok(failure.to_string().into()),
    }
}

#[function_component(Catscii)]
pub fn catscii() -> Html {
    let fallback = html! {<div>{"Loading..."}</div>};
    html! {
        <Suspense {fallback}>
            <CatsciiContent />
        </Suspense>
    }
}
