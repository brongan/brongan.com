use leptos::prelude::*;

#[component]
pub fn MemoryViewer(memory: RwSignal<Vec<u8>>, pc: RwSignal<u16>) -> impl IntoView {
    let pc = pc.get() as usize;

    let rows = memory
        .get()
        .chunks(16)
        .enumerate()
        .map(|(i, chunk)| {
            let addr = i * 16;
            let is_active = pc >= addr && pc < addr + 16;

            let hex_str = chunk
                .iter()
                .map(|b| format!("{:02X} ", b))
                .collect::<String>();
            let ascii_str = chunk
                .iter()
                .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
                .collect::<String>();

            view! {
                <div class="memory-row" class:active=is_active>
                    <div class="addr">{format!("0x{:04X}", addr)}</div>
                    <div class="hex">{hex_str}</div>
                    <div class="ascii">{ascii_str}</div>
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="memory-viewer">
            <h3>"Memory"</h3>
            <div class="memory-content" style="font-family: monospace; font-size: 12px;">
                {rows}
            </div>
        </div>
    }
}
