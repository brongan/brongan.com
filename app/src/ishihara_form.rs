use crate::ishihara::Blindness;
use leptos::html::Form;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use std::str::FromStr;
use strum::IntoEnumIterator;
use web_sys::{FormData, SubmitEvent};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct IshiharaArgs {
    pub blindness: Blindness,
    pub text: String,
}

const TEXT_INPUT: &str = "text";

impl From<FormData> for IshiharaArgs {
    fn from(data: FormData) -> Self {
        Self {
            text: data.get(TEXT_INPUT).as_string().unwrap(),
            blindness: Blindness::from_str(&data.get("blindness").as_string().unwrap()).unwrap(),
        }
    }
}

#[component]
pub fn ishihara_input() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();

    let blindness_choices = move || {
        let current_blindness = query
            .get()
            .get("blindness")
            .as_ref()
            .and_then(|b| Blindness::from_str(b).ok())
            .unwrap_or_default();

        Blindness::iter()
            .map(|blindness| {
                let choice = format!("{}-{}", blindness, "choice");
                let checked = blindness == current_blindness;
                view! {
                    <input
                        type="radio"
                        id={choice.clone()}
                        name="blindness"
                        value={blindness.to_string()}
                        checked=checked
                    />
                    <label for={choice}> {blindness.to_string()} </label>
                }
            })
            .collect_view()
    };

    let form_element: NodeRef<Form> = NodeRef::new();
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let form = form_element.get().expect("form should be mounted");
        let data = FormData::new_with_form(&form).expect("good form.");
        let args = IshiharaArgs::from(data);

        navigate(
            &format!("?text={}&blindness={}", args.text, args.blindness),
            Default::default(),
        );
    };

    let initial_text = move || query.get().get(TEXT_INPUT).unwrap_or_default();

    view! {
        <form on:submit=on_submit node_ref={form_element}>
            <div class="blindness-selector">
                {blindness_choices}
            </div>
            <div class="entry">
                <input
                    name={TEXT_INPUT}
                    placeholder="Text to Encrypt"
                    type="text"
                    prop:value=initial_text
                />
                <button type="submit"> {"Encrypt"} </button>
            </div>
        </form>
    }
}
