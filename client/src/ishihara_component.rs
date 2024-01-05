use crate::ishihara::{generate_plate, Blindness};
use crate::ishihara_form::{Data, IshiharaInput};
use leptos::html::Canvas;
use leptos::ReadSignal;
use leptos::{component, create_node_ref, create_signal, view, IntoView, NodeRef};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, ImageData};

#[component]
pub fn render_plate(form_data: Data) -> impl IntoView {
    let plate = generate_plate(&form_data.text, form_data.blindness);
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(plate.as_raw()),
        plate.width(),
        plate.height(),
    );

    let canvas_element: NodeRef<Canvas> = create_node_ref();
    let canvas = canvas_element().expect("has canvas");
    canvas.set_width(plate.width());
    canvas.set_height(plate.height());
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    ctx.put_image_data(&image.unwrap(), 0.0, 0.0).unwrap();
    view! { <canvas _ref={canvas_element} /> }
}

#[component]
pub fn ishihara_plate() -> impl IntoView {
    let (data, set_data) = create_signal(Data {
        blindness: Blindness::Demonstration,
        text: String::from(""),
    });

    view! {
        <header class="header">
            <h1> { "Ishihara Plate Generator" } </h1>
        </header>
        <div class="description">
            <p style="display:inline"> { "Randomly Generates a Colorblindness Test Image in your browser! See: "} </p>
            <a href="https://en.wikipedia.org/wiki/Ishihara_test"> {"wikipedia.org/wiki/Ishihara_test"} </a>
        </div>
        <IshiharaInput set_data={set_data}/>
        <div class="readout">
        {
            move || match data() {
                Some(form_data) => view! { <RenderPlate form_data /> }.into_view(),
                None => view! { <p>"Loading..."</p> }.into_view(),
            }
        }
        </div>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}
