use crate::chip8::emulator::cpu::{Register, Registers};
use leptos::prelude::*;
use strum::IntoEnumIterator;

#[component]
pub fn CpuState(
    pc: ReadSignal<u16>,
    registers: ReadSignal<Registers>,
    index: ReadSignal<u16>,
    delay_timer: ReadSignal<u8>,
    sound_timer: ReadSignal<u8>,
    memory: ReadSignal<Vec<u8>>,
) -> impl IntoView {
    let get_ir = move || {
        let mem = memory.get();
        let pc = pc.get() as usize;
        if pc + 1 < mem.len() {
            (mem[pc] as u16) << 8 | (mem[pc + 1] as u16)
        } else {
            0
        }
    };

    let render_cell = |label: String, val: u16, is_16bit: bool| {
        let hex_str = if is_16bit {
            format!("0x{:04X}", val)
        } else {
            format!("0x{:02X}", val)
        };
        view! {
            <div class="reg-cell">
                <span class="lbl">{label}</span>
                <span class="val">{hex_str}</span>
                <span class="dec">{val}</span>
            </div>
        }
    };

    view! {
        <div class="cpu-state-panel">
            <div class="panel-header">"CPU State"</div>
            <div class="grid-3col">
                {move || render_cell("PC".to_string(), pc.get(), true)}
                {move || render_cell("IR".to_string(), get_ir(), true)}
                {move || render_cell("I".to_string(), index.get(), true)}
            </div>
            <div class="grid-2col timers">
                {move || render_cell("Delay Timer".to_string(), delay_timer.get() as u16, false)}
                {move || render_cell("Sound Timer".to_string(), sound_timer.get() as u16, false)}
            </div>
            <hr class="divider" />
            <div class="grid-4col">
                {move || {
                    let vals = registers.get();
                    Register::iter().map(|reg| {
                        render_cell(format!("{}", reg), vals.get(reg) as u16, false)
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
