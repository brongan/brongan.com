fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<client::Root>::new().render();
}
