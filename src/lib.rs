pub mod analytics;
#[cfg(feature = "ssr")]
pub mod catscii;
pub mod color;
#[cfg(feature = "ssr")]
pub mod fileserve;
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

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(root::Root);
}
