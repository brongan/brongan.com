use ishihara::IshiharaPlate;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<IshiharaPlate>::new().render();
}
