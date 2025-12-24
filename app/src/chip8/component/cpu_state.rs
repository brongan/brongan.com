use crate::chip8::emulator::cpu::{Register, Registers};
use leptos::prelude::*;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct UiSignals {
    pub pc: RwSignal<u16>,
    pub registers: RwSignal<Registers>,
    pub index: RwSignal<u16>,
    pub timers: RwSignal<(u8, u8)>, // (delay, sound)
    pub stack: RwSignal<Vec<u16>>,
    pub memory: RwSignal<Vec<u8>>,
    pub instruction_count: RwSignal<u64>,
}

#[component]
pub fn CpuState(ui: UiSignals) -> impl IntoView {
    let registers = ui.registers.get();
    view! {
        <div class="cpu-state">
            <h3>"Registers"</h3>
            <div class="hex-grid">
                <div>"PC:"</div> <div>{move || format!("0x{:04X}", ui.pc.get())}</div>
                <div>"I:"</div>  <div>{move || format!("0x{:04X}", ui.index.get())}</div>
                <div>"DT:"</div> <div>{move || format!("{:02X}", ui.timers.get().0)}</div>
                <div>"ST:"</div> <div>{move || format!("{:02X}", ui.timers.get().1)}</div>
            </div>

            <div class="regs-grid" style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 5px; margin-top: 10px;">
                {move || Register::iter().map(|register| {
                    view! {
                        <div class="reg-box">
                            <small style="color: #888">{format!("{register}")}</small>
                            <div>{format!("{:02X}", registers.get(register))}</div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
