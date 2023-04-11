use num::Complex;
use wasm_bindgen::{JsCast, Clamped};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;

use mandelbrot::{generate_mandelbrot, Bounds};

#[function_component(MandelbrotModel)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();
    let canvas_ref_callback = canvas_ref.clone();

    if let Some(canvas) = canvas_ref_callback.cast::<HtmlCanvasElement>() {
            let image = generate_mandelbrot(Bounds {width: 1920, height: 1080}, Complex::<f64>{re: -1.20, im: 0.35}, Complex::<f64>{re: -1.0, im: 0.20});
            let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(&image.pixels),
                image.width,
                image.height,
            );
            canvas.set_width(image.width);
            canvas.set_height(image.width);
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0)
                .unwrap();
    }

    html! {
        <>
            <header class="mandelbrot-header">
                <h1> { "Mandelbrot" } </h1>
            </header>
            <div class="mandelbrot-readout">
                <canvas ref={canvas_ref} />
            </div>
            <footer class="mandelbrot-footnote">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </>
    }
}

