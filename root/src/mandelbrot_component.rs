use num::Complex;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;

use mandelbrot::{generate_mandelbrot, Bounds};

#[function_component(MandelbrotModel)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();
    let image = generate_mandelbrot(
        Bounds {
            width: 800,
            height: 500,
        },
        Complex::<f64> {
            re: -1.20,
            im: 0.35,
        },
        Complex::<f64> { re: -1.0, im: 0.20 },
    );

    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_raw()),
        image.width(),
        image.height(),
    )
    .unwrap();

    {
        let canvas_ref = canvas_ref.clone();
        use_effect(move || {
            let canvas = canvas_ref.cast::<HtmlCanvasElement>().unwrap();
            canvas.set_width(image.width());
            canvas.set_height(image.width());
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&image, 0.0, 0.0).unwrap();
        });
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
