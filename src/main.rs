use yew::{function_component, html, Html};
use root::Root;

#[function_component(Workspace)]
fn main() -> Html {
    html! {
        <Root/>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Workspace>::new().render();
}
