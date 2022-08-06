use wasm_game_of_life::GameOfLifeModel;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<GameOfLifeModel>::new().render();
}
