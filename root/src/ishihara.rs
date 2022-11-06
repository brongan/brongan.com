use strum::IntoEnumIterator;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::{function_component, html, use_node_ref, use_state, Callback, Html, Properties};

use ishihara::{generate_plate, Blindness};

struct IshiharaFormData {
    selected_blindness: Blindness,
    text: String,
}

#[derive(Clone, Properties, PartialEq)]
struct InputProps {
    selected: Blindness,
    on_submit: Callback<IshiharaFormData>,
}

#[function_component(IshiharaInput)]
fn ishihara_input(InputProps { selected, on_blindness_selected }: &InputProps) -> Html {
    let on_blindness_selected = on_click.clone();
    let input_ref = use_node_ref();

    let blindness_choices = Blindness::iter().map(|blindness| {
        let on_blindness_select = {
            let on_click = on_click.clone();
            let blindness = blindness.clone();
            Callback::from(move |_| {
                on_click.emit(blindness.clone());
            })
        };
        let choice = format!("{}-{}", blindness.to_string(), "choice");
        let checked = *selected == blindness;
        html! {
            <>
            <input type="radio" id={choice.clone()}  name="blindness" value={blindness.to_string()} checked={checked}/>
            <label onclick={on_blindness_select} for={choice}> {blindness.to_string()} </label>
            </>

        }
    }).collect();

    let selected_blindness = use_state(|| Some(Blindness::Demonstration));
    let on_blindness_select = {
        let selected_blindness = selected_blindness.clone();
        Callback::from(move |blindness: Blindness| selected_blindness.set(Some(blindness)))
    };
    let input_ref = input_ref.clone();

    html!{
        <div class="ishihara-blindness">
            <BlindnessList selected={*selected_blindness.as_ref().unwrap()} on_blindness_selected={on_blindness_select}/>
        </div>
        <form class="ishihara-entry" action="">
            <input ref={input_ref} placeholder="Text to Encrypt" type="string" />
            <button type="button" onclick={onclick}>{ "Encrypt" }</button>
        </form>
    }
}

#[function_component(IshiharaPlate)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();

    let ishihara_input = InputProps {
        selected: Blindness::Demonstration,
        on_submit: |form_data: IshiharaFormData| {
            let canvas_ref = canvas_ref.clone();
            let selected_blindness = form_data.selected_blindness;
            move |_| {
                if let (Some(input), Some(canvas)) = (
                    input_ref.cast::<HtmlInputElement>(),
                    canvas_ref.cast::<HtmlCanvasElement>(),
                    ) {
                    let plate = generate_plate(form_data.text, selected_blindness);
                    let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(
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
                    ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0)
                        .unwrap();
                }
            }
        }
    }

    html! {
        <main class="ishihara-main">
            <header class="ishihara-header">
                <h1> { "Color Blind Message Encrypter" } </h1><h1>{ "Ishihara Plate Generator" }</h1>
            </header>
            <div class="description">
                <p style="display:inline"> { "Randomly Generates a Colorblindness Test Image in your browser! See: "} </p>
                <a href="https://en.wikipedia.org/wiki/Ishihara_test"> {"wikipedia.org/wiki/Ishihara_test"} </a>
            </div>
            <IshiharaInput on
            <div class="ishihara-readout">
                <canvas ref={canvas_ref} />
            </div>
            <footer class="ishihara-footnote">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </main>
    }
}
