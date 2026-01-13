use leptos::prelude::*;

#[component]
pub fn StackViewer(
    #[prop(into)] stack: ReadSignal<[u16; 16]>,
    #[prop(into)] sp: ReadSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="stack-viewer">
            <div class="panel-header">"Call Stack"</div>
            <div class="stack-grid">
                {move || {
                    let current_sp = sp.get();
                    let current_stack = stack.get();

                    (0..16).map(|i| {
                        let val = current_stack[i];
                        // 0..sp are valid return addresses
                        let is_stored = i < current_sp;
                        // sp is the insertion point for the NEXT call
                        let is_pointer = i == current_sp;

                        view! {
                            <div
                                class="stack-row"
                                class:row-stored=is_stored
                                class:row-sp=is_pointer
                            >
                                // Index Column (0-F)
                                <div class="idx">
                                    {format!("{:X}", i)}
                                    {if is_pointer { " <" } else { "" }}
                                </div>

                                // Value Column
                                <div class="val">
                                    {if is_stored {
                                        format!("0x{:04X}", val)
                                    } else if is_pointer {
                                        "[ SP ]".to_string()
                                    } else {
                                        "-".to_string()
                                    }}
                                </div>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
