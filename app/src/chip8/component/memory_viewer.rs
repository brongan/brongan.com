use leptos::prelude::*;

#[component]
pub fn MemoryViewer(memory: ReadSignal<Vec<u8>>, pc: ReadSignal<u16>) -> impl IntoView {
    let rows = move || {
        let current_pc = pc.get() as usize;

        memory
            .get()
            .chunks(16)
            .enumerate()
            .map(|(i, chunk)| {
                let addr = i * 16;
                let is_active = current_pc >= addr && current_pc < addr + 16;

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
            .collect_view()
    };

    view! {
        <div class="memory-viewer">
            <div class="panel-header">"Memory Viewer"</div>
            <div class="memory-content">
                {rows}
            </div>
        </div>
    }
}
