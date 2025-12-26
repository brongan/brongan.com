use crate::chip8::emulator::quirks::Quirks;
use leptos::prelude::*;

#[component]
pub fn QuirkSettings(#[prop(into)] quirks: RwSignal<Quirks>) -> impl IntoView {
    view! {
        <div class="quirks-panel">
            <div class="panel-header">"Quirks / Compatibility"</div>

            <div class="settings-list">
                // 1. Shift Vy
                <div class="checkbox-row" title="On: Vx = Vy >> 1. Off: Vx = Vx >> 1 (Original)">
                    <input
                        type="checkbox" id="q_shift"
                        prop:checked=move || quirks.get().shift_vy
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.shift_vy = c);
                        }
                    />
                    <label for="q_shift">"Shift Vy"</label>
                </div>

                // 2. Memory Increment
                <div class="checkbox-row" title="On: I increments after load/store (Original). Off: I stays same.">
                    <input
                        type="checkbox" id="q_mem"
                        prop:checked=move || quirks.get().memory_increment
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.memory_increment = c);
                        }
                    />
                    <label for="q_mem">"Mem Increment"</label>
                </div>

                // 3. Logic VF Reset
                <div class="checkbox-row" title="On: Logic ops reset VF to 0 (Original). Off: VF unchanged.">
                    <input
                        type="checkbox" id="q_vf"
                        prop:checked=move || quirks.get().vf_reset
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.vf_reset = c);
                        }
                    />
                    <label for="q_vf">"Logic Resets VF"</label>
                </div>

                // 4. Clipping
                <div class="checkbox-row" title="On: Sprites wrap at edges. Off: Sprites clip.">
                    <input
                        type="checkbox" id="q_clip"
                        prop:checked=move || quirks.get().clipping
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.clipping = c);
                        }
                    />
                    <label for="q_clip">"Clipping"</label>
                </div>

                // 5. Display Wait
                <div class="checkbox-row" title="On: Limit draw speed to 60hz (Original). Off: Instant draw.">
                    <input
                        type="checkbox" id="q_wait"
                        prop:checked=move || quirks.get().display_wait
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.display_wait = c);
                        }
                    />
                    <label for="q_wait">"Display Wait"</label>
                </div>

                // 6. Jumping (Bnnn)
                <div class="checkbox-row" title="On: Jump to nnn + Vx. Off: Jump to nnn + V0 (Original).">
                    <input
                        type="checkbox" id="q_jump"
                        prop:checked=move || quirks.get().jumping
                        on:change=move |ev| {
                            let c = event_target_checked(&ev);
                            quirks.update(|q| q.jumping = c);
                        }
                    />
                    <label for="q_jump">"Jumping (Bnnn)"</label>
                </div>
            </div>
        </div>
    }
}
