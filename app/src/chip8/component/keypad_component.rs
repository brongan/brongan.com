use crate::chip8::emulator::cpu::Keypad;
use leptos::ev;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

fn map_key(code: &str) -> Option<u8> {
    match code {
        "Digit1" => Some(0x1),
        "Digit2" => Some(0x2),
        "Digit3" => Some(0x3),
        "Digit4" => Some(0xC),

        "KeyQ" => Some(0x4),
        "KeyW" => Some(0x5),
        "KeyE" => Some(0x6),
        "KeyR" => Some(0xD),

        "KeyA" => Some(0x7),
        "KeyS" => Some(0x8),
        "KeyD" => Some(0x9),
        "KeyF" => Some(0xE),

        "KeyZ" => Some(0xA),
        "KeyX" => Some(0x0),
        "KeyC" => Some(0xB),
        "KeyV" => Some(0xF),
        _ => None,
    }
}

const KEYS: [(&str, u8); 16] = [
    ("1", 0x1),
    ("2", 0x2),
    ("3", 0x3),
    ("4", 0xC),
    ("Q", 0x4),
    ("W", 0x5),
    ("E", 0x6),
    ("R", 0xD),
    ("A", 0x7),
    ("S", 0x8),
    ("D", 0x9),
    ("F", 0xE),
    ("Z", 0xA),
    ("X", 0x0),
    ("C", 0xB),
    ("V", 0xF),
];

#[component]
pub fn KeypadComponent(keypad: RwSignal<Keypad>) -> impl IntoView {
    Effect::new(move |_| {
        let handle_keydown = window_event_listener(ev::keydown, move |ev| {
            if let Some(k) = map_key(&ev.code()) {
                keypad.update(|keys| keys.enable_key(k));
            }
        });

        let handle_keyup = window_event_listener(ev::keyup, move |ev| {
            if let Some(k) = map_key(&ev.code()) {
                keypad.update(|keys| keys.disable_key(k));
            }
        });

        on_cleanup(move || {
            handle_keydown.remove();
            handle_keyup.remove();
        });
    });

    view! {
        <div class="chip8-instructions">
            <div class="key-grid"> {
                KEYS.iter().map(|(label, val)| {
                    let is_pressed = move || keypad.get().is_pressed(*val);
                    view! {
                        <div class="key" class:pressed=is_pressed
                           on:mousedown=move |_| {
                                keypad.update(|k| k.enable_key(*val));
                            }
                            on:mouseup=move |_| {
                                keypad.update(|k| k.disable_key(*val));
                            }
                            on:mouseleave=move |_| {
                                keypad.update(|k| k.disable_key(*val));
                            }
                            on:touchstart=move |_e| {
                                keypad.update(|k| k.enable_key(*val));
                            }
                            on:touchend=move |_| {
                                keypad.update(|k| k.disable_key(*val));
                            }>
                           <span class="key-label">{*label}</span>
                           <span class="hex-label">{format!("{:X}", *val)}</span>
                        </div>
                    }
                }).collect_view()
            }
            </div>
        </div>
    }
}
