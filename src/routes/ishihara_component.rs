use crate::ishihara::{generate_plate, Blindness};
use crate::ishihara_form::{IshiharaArgs, IshiharaInput};
use leptos::html::Canvas;
use leptos::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, ImageData};

#[component]
pub fn show_plate(ishihara_args: ReadSignal<IshiharaArgs>) -> impl IntoView {
    let canvas_element: NodeRef<Canvas> = create_node_ref();
    create_effect(move |_| {
        let args: IshiharaArgs = ishihara_args.get();
        let plate = generate_plate(&args.text, args.blindness);
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
    });
    view! { <canvas _ref={canvas_element} /> }
}

#[component]
pub fn ishihara_plate() -> impl IntoView {
    let (ishihara_args, set_ishihara_args) = create_signal(IshiharaArgs {
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
        <div class="input">
            <IshiharaInput set_data={set_ishihara_args}/>
        </div>
        <div class="readout">
            <ShowPlate ishihara_args/>
        </div>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}
