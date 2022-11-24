use yew::prelude::*;
use strum::IntoEnumIterator;
use web_sys::{FormData, HtmlFormElement};
use std::str::FromStr;

use ishihara::Blindness;

#[derive(Debug, Default)]
pub struct Data {
    pub blindness: Blindness,
    pub text: String,
}

const TEXT_INPUT: &str = "text-input";

impl From<FormData> for Data {
    fn from(data: FormData) -> Self {
        Self {
            text: data.get(TEXT_INPUT).as_string().unwrap(),
            blindness: Blindness::from_str(&data.get("blindness").as_string().unwrap()).unwrap(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onsubmit: Callback<Data>
}

#[function_component(IshiharaInput)]
pub fn ishihara_input(props: &Props) -> Html {
    let form_ref = use_node_ref();
    let form_callback_ref = form_ref.clone();
    let blindness_choices: Html = Blindness::iter().map(|blindness| {
        let choice = format!("{}-{}", blindness, "choice");
        html! {
            <>
                <input type="radio" id={choice.clone()} name="blindness" value={blindness.to_string()} checked=true />
                <label for={choice}> {blindness.to_string()} </label>
            </>
        }
    }).collect();

    let form_callback = props.onsubmit.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default(); 
        if let Some(form) = form_callback_ref.cast::<HtmlFormElement>() {
        let data = Data::from(FormData::new_with_form(&form).unwrap());
        form_callback.emit(data);
        }
        
    });

    html! {
        <form onsubmit={onsubmit} ref={form_ref}>
            <div class="blindness-selector">
                {blindness_choices}
            </div>
            <div class="ishihara-entry">
                <input name={TEXT_INPUT} placeholder="Text to Encrypt" type="string" />
                <button type="submit"> {"Encrypt"} </button>
            </div>
        </form>
    }
}
