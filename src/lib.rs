pub mod analytics;
#[cfg(feature = "ssr")]
pub mod catscii;
pub mod color;
pub mod game_of_life;
pub mod ishihara;
pub mod ishihara_form;
#[cfg(feature = "ssr")]
pub mod locat;
pub mod mandelbrot;
#[cfg(feature = "ssr")]
pub mod mandelbrot_backend;
pub mod point2d;
pub mod root;
pub mod routes;
#[cfg(feature = "ssr")]
pub mod server;

use axum::extract::FromRef;
use leptos::LeptosOptions;
use locat::Locat;
use std::sync::Arc;

#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct ServerState {
    leptos_options: LeptosOptions,
    client: reqwest::Client,
    locat: Arc<Locat>,
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::root::Root;

    // TODO fix.
    // _ = leptos::leptos_dom::logging::console_log::init_with_level(log::Level::Debug);
    // console_error_panic_hook::set_once();

    leptos::mount_to_body(Root);
}
