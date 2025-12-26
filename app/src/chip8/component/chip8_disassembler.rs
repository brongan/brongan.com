use leptos::prelude::*;

use crate::chip8::emulator::cpu::Instruction;

#[component]
pub fn Disassembler(memory: ReadSignal<Vec<u8>>, pc: ReadSignal<u16>) -> impl IntoView {
    let instruction_rows = move || {
        let mem = memory.get();
        let current_pc = pc.get() as usize;
        let start_idx = std::cmp::min(current_pc.saturating_sub(10 * 2), 0x200);
        let end_idx = (current_pc + 20 * 2).min(mem.len());

        (start_idx..end_idx)
            .step_by(2)
            .map(|addr| {
                if addr + 1 >= mem.len() {
                    return (addr, 0x0000, "EOF".to_string(), false);
                }

                let hi = mem[addr] as u16;
                let lo = mem[addr + 1] as u16;
                let opcode = (hi << 8) | lo;
                let mnemonic = Instruction::decode(opcode)
                    .map(|instr| instr.to_string())
                    .unwrap_or_else(|| format!("UNK 0x{:04X}", opcode));

                let is_active = addr == current_pc;
                (addr, opcode, mnemonic, is_active)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="disassembler-panel">
            <div class="panel-header">"Disassembly"</div>
            <div class="code-window">
                {move || instruction_rows().into_iter().map(|(addr, opcode, mnemonic, is_active)| {
                    view! {
                        <div class="code-row" class:active=is_active>
                            <span class="addr">{format!("0x{:03X}", addr)}</span>
                            <span class="hex">{format!("{:04X}", opcode)}</span>
                            <span class="mnemonic">{mnemonic}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
