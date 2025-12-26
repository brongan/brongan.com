use leptos::prelude::*;

#[component]
pub fn Controls(
    #[prop(into)] is_active: Signal<bool>,
    #[prop(into)] pause: Callback<()>,
    #[prop(into)] resume: Callback<()>,
    #[prop(into)] step: Callback<u32>,
    #[prop(into)] reset: Callback<()>,
    #[prop(into)] load: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="controls-panel">
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
            </div>
        </div>
    }
}
