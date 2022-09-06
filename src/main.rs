use root::Root;
use yew::{function_component, html, Html};

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
