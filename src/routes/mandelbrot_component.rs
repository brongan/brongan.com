use crate::mandelbrot::{generate_mandelbrot, Bounds};
use anyhow::anyhow;
use image::io::Reader as ImageReader;
use image::RgbaImage;
use leptos::{component, view, IntoView};
use leptos::{create_node_ref, html::Canvas, NodeRef};
use log::info;
use num::Complex;
use std::io::Cursor;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, ImageData};

#[component]
fn show_mandelbrot(image: RgbaImage) -> impl IntoView {
    let image = ImageData::new_with_u8_clamped_array_and_sh(
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
fn render_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    info!("generating mandelbrot image");
    let image = generate_mandelbrot(bounds, upper_left, lower_right);
    view! { <ShowMandelbrot image/> }
}

fn convert_to_js(image: &[u8]) -> ImageData {
    info!("Converting Image to javascript.");
    let image = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_bytes()),
        image.width(),
        image.height(),
    )
    .unwrap()
}

#[component]
pub fn mandelbrot_model(bounds: Bounds) -> impl IntoView {
    let upper_left = Complex::<f64> {
        re: -1.20,
        im: 0.35,
    };
    let lower_right = Complex::<f64> { re: -1.0, im: 0.20 };

    view! {
        <>
            <header class="header">
            <h1> { "Mandelbrot" } </h1>
            </header>
            <div class="readout">
            <RenderMandelbrot bounds={bounds} upper_left={upper_left} lower_right={lower_right} />
            </div>
            <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
            </>
    }
}
