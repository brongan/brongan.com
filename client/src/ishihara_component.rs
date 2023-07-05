use crate::ishihara::generate_plate;
use crate::ishihara_form::{Data, IshiharaInput};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;

#[function_component(IshiharaPlate)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();
    let canvas_ref_callback = canvas_ref.clone();

    let onsubmit_func = move |form_data: Data| {
        if let Some(canvas) = canvas_ref_callback.cast::<HtmlCanvasElement>() {
            let plate = generate_plate(&form_data.text, form_data.blindness);
            let image = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(plate.as_raw()),
                plate.width(),
                plate.height(),
            );
            canvas.set_width(plate.width());
            canvas.set_height(plate.height());
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&image.unwrap(), 0.0, 0.0).unwrap();
        }
    };

    html! {
        <>
            <header class="ishihara-header">
                <h1> { "Ishihara Plate Generator" } </h1>
            </header>
            <div class="ishihara-description">
                <p style="display:inline"> { "Randomly Generates a Colorblindness Test Image in your browser! See: "} </p>
                <a href="https://en.wikipedia.org/wiki/Ishihara_test"> {"wikipedia.org/wiki/Ishihara_test"} </a>
            </div>
            <IshiharaInput onsubmit={onsubmit_func}/>
            <div class="ishihara-readout">
                <canvas ref={canvas_ref} />
            </div>
            <footer class="ishihara-footnote">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </>
    }
}
