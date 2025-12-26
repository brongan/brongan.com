use std::time::Duration;

use leptos::prelude::*;

#[component]
pub fn EmulatorInfo(
    is_active: Signal<bool>,
    rom_name: ReadSignal<Option<String>>,
    frame_time: ReadSignal<Duration>,
    fps: ReadSignal<f64>,
    instruction_count: ReadSignal<u64>,
    beep: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="emulator-info-panel">
            <div class="panel-header">"Emulator Info"</div>
            <dl class="info-grid">
                <dt>"ROM:"</dt>
                <dd class="rom-name">
                    {move || rom_name.get().unwrap_or_else(|| "None".to_string())}
                </dd>
                <dt>"GUI FPS:"</dt>
                <dd>{move || format!("{:.1}", fps.get())}</dd>
                <dt>"Frame Time:"</dt>
                <dd>{move || format!("{:.1} ms", frame_time.get().as_millis())}</dd>
                <dt>"Current:"</dt>
                <dd
                    class:state-running=move || is_active.get()
                    class:state-stopped=move || !is_active.get()
                >
                    {move || if is_active.get() { "Running" } else { "Stopped" }}
                </dd>
                <dt>"Ops:"</dt>
                <dd>{move || instruction_count.get().to_string()}</dd>
                <dt>"Sound:"</dt>
                <dd
                    class:audio-beep=move || beep.get()
                    class:audio-ok=move || !beep.get()
                >
                    {move || if beep.get() { "BEEP" } else { "OK" }}
                </dd>
            </dl>
        </div>
    }
}
