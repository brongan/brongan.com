use crate::ishihara::Blindness;
use leptos::html::Form;
use leptos::{component, create_node_ref, view, IntoView, NodeRef, WriteSignal};
use leptos::{CollectView, IntoAttribute};
use std::str::FromStr;
use strum::IntoEnumIterator;
use web_sys::{FormData, SubmitEvent};

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

#[component]
pub fn ishihara_input(set_data: WriteSignal<Data>) -> impl IntoView {
    let blindness_choices = Blindness::iter().map(|blindness| {
        let choice = format!("{}-{}", blindness, "choice");
        view! {
            <input type="radio" id={choice.clone()} name="blindness" value={blindness.to_string()} checked=true />
            <label for={choice}> {blindness.to_string()} </label>
        }
    }).collect_view();

    let form_element: NodeRef<Form> = create_node_ref();
    let form = form_element().expect("form in DOM.");
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = form_element().expect("<input> to exist");
        set_data(Data::from(FormData::new_with_form(form.into())));
    };

    view! {
        <form onsubmit={on_submit} ref={form_ref}>
            <div class="blindness-selector">
                {blindness_choices}
            </div>
            <div class="entry">
                <input name={TEXT_INPUT} placeholder="Text to Encrypt" type="string" />
                <button type="submit"> {"Encrypt"} </button>
            </div>
        </form>
    }
}
