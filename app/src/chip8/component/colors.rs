use leptos::prelude::*;

#[component]
pub fn ColorSettings(
    #[prop(into)] on_color: RwSignal<String>,
    #[prop(into)] off_color: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="colors-panel">
            <label for="col_on">"On Color"</label>
            <div class="picker-wrapper">
                <input
                    type="color"
                    id="col_on"
                    prop:value=move || on_color.get()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        on_color.set(val);
                    }
                />
                <span class="hex-label">{move || on_color.get()}</span>
            </div>

            <label for="col_off">"Off Color"</label>
            <div class="picker-wrapper">
                <input
                    type="color"
                    id="col_off"
                    prop:value=move || off_color.get()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        off_color.set(val);
                    }
                />
                <span class="hex-label">{move || off_color.get()}</span>
            </div>
        </div>
    }
}
