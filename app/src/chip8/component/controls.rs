use leptos::prelude::*;

#[component]
pub fn Controls(
    #[prop(into)] is_active: Signal<bool>,
    #[prop(into)] pause: Callback<()>,
    #[prop(into)] resume: Callback<()>,
    #[prop(into)] step: Callback<u32>,
    #[prop(into)] reset: Callback<()>,
    #[prop(into)] load: Callback<()>,
    #[prop(into)] roms: Vec<(&'static str, &'static str)>,
    #[prop(into)] on_rom_select: Callback<String>,
    #[prop(into)] selected_rom_url: RwSignal<String>,
    #[prop(into)] debug_mode: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="controls-panel">
            <div class="control-row">
                 <select
                    class="rom-select"
                    prop:value=move || selected_rom_url.get()
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        if !val.is_empty() {
                            on_rom_select.run(val);
                        }
                    }
                 >
                    <option value="" selected disabled>"Select ROM ▾"</option>
                    {roms.into_iter().map(|(name, url)| {
                        view! {
                            <option value=url>{name}</option>
                        }
                    }).collect_view()}
                 </select>
            </div>

            <div class="control-row">
                 <label class="debug-label">
                    <input
                        type="checkbox"
                        prop:checked=move || debug_mode.get()
                        on:change=move |_| debug_mode.update(|d| *d = !*d)
                    />
                    "Debug Mode"
                 </label>
            </div>

            <div class="button-grid">
                <button
                    class="btn-control"
                    on:click=move |_| load.run(())
                    title="Load ROM File"
                >
                    "📂 Load ROM"
                </button>
                <button
                    class="btn-control btn-danger"
                    on:click=move |_| reset.run(())
                    title="Reset Emulator (R)"
                >
                    "↻ Reset"
                </button>
                <button
                    class="btn-control"
                    class:active=move || is_active.get()
                    on:click=move |_| {
                        if is_active.get() {
                            pause.run(());
                        } else {
                            resume.run(());
                        }
                    }
                    title=move || if is_active.get() { "Pause (P)" } else { "Play (P)" }
                >
                    {move || if is_active.get() { "⏸ Pause" } else { "▶ Resume" }}
                </button>

                <Show when=move || debug_mode.get()>
                    <button
                        class="btn-control"
                        disabled=move || is_active.get()
                        on:click=move |_| step.run(1)
                        title="Step 1 Instruction"
                    >
                        "⤵ Step"
                    </button>

                    <button
                        class="btn-control"
                        disabled=move || is_active.get()
                        on:click=move |_| step.run(10)
                        title="Step 10 Instructions"
                    >
                        "⤵⤵ Step 10"
                    </button>
                </Show>
            </div>
        </div>
    }
}
