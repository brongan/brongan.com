use wasm_bindgen::{Clamped, JsCast};
use web_sys::{HtmlCanvasElement, HtmlInputElement, CanvasRenderingContext2d, ImageData};
use yew::{function_component, html, use_node_ref, Html};

use ishihara::generate_plate;

#[function_component(IshiharaPlate)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();
    let input_ref = use_node_ref();

    let onclick = {
        let canvas_ref = canvas_ref.clone();
        let input_ref = input_ref.clone();
        move |_| {
            if let (Some(input), Some(canvas)) = (input_ref.cast::<HtmlInputElement>(), canvas_ref.cast::<HtmlCanvasElement>()) {
                let plate = generate_plate(&input.value());
                let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(plate.as_raw()), plate.width(), plate.height());
                canvas.set_width(plate.width());
                canvas.set_height(plate.height());
                let ctx = 
                    canvas
                    .get_context("2d").unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>().unwrap();
                ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0).unwrap();
            }
        }
    };
    html! {
        <div>
            <input ref={input_ref} type="string" />
            <button {onclick}/>
            <canvas ref={canvas_ref} />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<IshiharaPlate>::new().render();
}

