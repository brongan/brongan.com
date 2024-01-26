use locat::Locat;
use std::sync::Arc;
pub mod analytics;
#[cfg(feature = "ssr")]
pub mod catscii;
pub mod color;
pub mod game_of_life;
pub mod ishihara;
#[cfg(feature = "hydrate")]
pub mod ishihara_form;
#[cfg(feature = "ssr")]
pub mod locat;
pub mod mandelbrot;
pub mod point2d;
pub mod root;
pub mod routes;

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct ServerState {
    client: reqwest::Client,
    locat: Arc<Locat>,
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::root::Root;

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(Root);
}
