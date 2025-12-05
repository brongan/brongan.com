use leptos::prelude::*;

pub mod cpu;
pub mod emulator;
pub mod quirks;
pub mod screen;

#[component]
pub fn Chip8() -> impl IntoView {
    view! {
        <header class="header">
            <h1 class="title">{ "Chip-8" }</h1>
        </header>
        <div class="content">
          "Todo"
        </div>
    }
}
