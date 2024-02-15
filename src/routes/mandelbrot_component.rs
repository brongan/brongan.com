use crate::mandelbrot::Bounds;
use leptos::*;
use log::info;
use num::Complex;

#[cfg(not(feature = "ssr"))]
use wasm_bindgen::{Clamped, JsCast};

#[component]
#[cfg(not(feature = "ssr"))]
pub fn show_mandelbrot(image: image::RgbaImage) -> impl IntoView {
    use anyhow::anyhow;
    use image::RgbaImage;
    use leptos::html::Canvas;
    use web_sys::CanvasRenderingContext2d;

    let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_raw()),
        image.width(),
        image.height(),
    )
    .map_err(|_| anyhow!("Failed to convert to ImageData."))
    .unwrap();

    let canvas_element: NodeRef<Canvas> = create_node_ref();
    let canvas = canvas_element().expect("has canvas");
    canvas.set_width(image.width());
    canvas.set_height(image.height());
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    ctx.put_image_data(&image, 0.0, 0.0).unwrap();

    view! {
        <canvas _ref=canvas_element type="text"/>
    }
}

#[component]
#[cfg(not(feature = "ssr"))]
fn render_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    info!("generating mandelbrot image");
    let image = crate::mandelbrot::generate_mandelbrot(bounds, upper_left, lower_right);
    view! { <ShowMandelbrot image/> }
}

#[component]
#[cfg(feature = "ssr")]
fn render_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    view! { <p> Hello from SSR. </p> }
}

#[server(GetMandelbrot, "/api", "Cbor", "mandelbrot")]
pub async fn get_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Result<Vec<u8>, ServerFnError> {
    let image =
        crate::mandelbrot::generate_mandelbrot_multithreaded(bounds, upper_left, lower_right);
    let mut image_bytes: Vec<u8> = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut image_bytes),
            image::ImageOutputFormat::Png,
        )
        .unwrap();
    Ok(image_bytes)
}

#[component]
fn load_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    info!("loading mandelbrot image");
    view! {
        <header class="header">
            <h1> { "Mandelbrot" } </h1>
        </header>
        <div class="readout">
            <RenderMandelbrot bounds={bounds} upper_left={upper_left} lower_right={lower_right} />
        </div>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}

#[component]
pub fn mandelbrot_model(bounds: Bounds) -> impl IntoView {
    info!("mandelbrot");
    let upper_left = Complex::<f64> {
        re: -1.20,
        im: 0.35,
    };
    let lower_right = Complex::<f64> { re: -1.0, im: 0.20 };

    view! {
        <header class="header">
            <h1> { "Mandelbrot" } </h1>
        </header>
        <div class="readout">
            <RenderMandelbrot bounds={bounds} upper_left={upper_left} lower_right={lower_right} />
        </div>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}

#[component]
pub fn mandelbrot() -> impl IntoView {
    view! { <MandelbrotModel bounds={Bounds {width: 800, height: 500}} /> }
}
