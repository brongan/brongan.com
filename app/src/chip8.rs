use crate::chip8::{cpu::Keypad, emulator::Emulator, screen::Screen};
use gloo_file::futures::read_as_bytes;
use gloo_file::File;
use leptos::ev;
use leptos::html::Canvas;
use leptos::html::Input;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::{use_raf_fn, utils::Pausable, UseRafFnCallbackArgs};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::{AudioContext, GainNode, OscillatorNode};

pub mod cpu;
pub mod emulator;
pub mod quirks;
pub mod screen;

#[derive(Clone)]
pub struct Beep {
    ctx: AudioContext,
    _oscillator: OscillatorNode,
    gain: GainNode,
}

impl Beep {
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;

        let oscillator = ctx.create_oscillator()?;
        oscillator.set_type(web_sys::OscillatorType::Square); // Chip-8 style square wave
        oscillator.frequency().set_value(440.0); // A4 pitch

        let gain = ctx.create_gain()?;
        gain.gain().set_value(0.0); // Start muted

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&ctx.destination())?;

        oscillator.start()?;

        Ok(Self {
            ctx,
            _oscillator: oscillator,
            gain,
        })
    }

    pub fn play(&self) {
        self.gain.gain().set_value(0.1);
    }

    pub fn pause(&self) {
        self.gain.gain().set_value(0.0);
    }

    pub fn resume_context(&self) {
        if self.ctx.state() == web_sys::AudioContextState::Suspended {
            let _ = self.ctx.resume();
        }
    }
}

#[component]
pub fn Chip8() -> impl IntoView {
    let keypad = RwSignal::new(Keypad::default());
    let beep = StoredValue::new_local(None::<Beep>);
    let emulator = StoredValue::new_local(Emulator::new(None));
    let canvas_ref = NodeRef::<Canvas>::new();
    let ctx_ref = StoredValue::new_local(None::<CanvasRenderingContext2d>);

    let Pausable {
        is_active,
        pause,
        resume,
    } = use_raf_fn({
        move |args: UseRafFnCallbackArgs| {
            emulator.update_value(|emulator| {
                let dt = Duration::from_secs_f64(args.delta / 1000.0);
                emulator.update(keypad.get(), dt);
                if let Some(audio) = beep.get_value() {
                    if emulator.is_beep() {
                        audio.play();
                    } else {
                        audio.pause();
                    }
                }
                ctx_ref.with_value(|ctx| {
                    if let Some(ctx) = ctx {
                        draw_screen(ctx, emulator.screen());
                    }
                });
            });
        }
    });

    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            ctx.set_fill_style_str("#1a1a1a");
            ctx.fill_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
            ctx_ref.set_value(Some(ctx));
        }
    });

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

    let file_input = NodeRef::<Input>::new();
    let on_file_upload = move |ev: ev::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);

        if let Some(file) = input.files().and_then(|files| files.get(0)).map(File::from) {
            spawn_local(async move {
                match read_as_bytes(&file).await {
                    Ok(bytes) => {
                        leptos::logging::log!("ROM loaded: {} bytes", bytes.len());
                        emulator.update_value(|emulator| {
                            emulator.reset();
                            emulator.update_rom(bytes);
                        });
                    }
                    Err(e) => leptos::logging::error!("Error reading file: {:?}", e),
                }
            });
        }
    };

    let init_audio = move || {
        beep.update_value(|b| {
            if b.is_none() {
                if let Ok(new_beep) = Beep::new() {
                    *b = Some(new_beep);
                }
            }
            // Always try to resume context (browser requirement)
            if let Some(audio) = b {
                audio.resume_context();
            }
        });
    };

    let keys = vec![
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

    view! {
        <header class="header">
            <h1 class="title">{ "Chip-8 Emulator" }</h1>
        </header>

        <div class="content chip8-container">
            <div class="chip8-controls">
                <input
                    type="file"
                    node_ref=file_input
                    on:change=on_file_upload
                    accept=".ch8"
                    style="display:none"
                />
                <button
                    class="btn"
                    on:click=move |_| {
                        init_audio();
                        if let Some(input) = file_input.get() {
                            input.click();
                        }
                    }
                >
                    "Load ROM"
                </button>
                <button class="btn" on:click=move |_| if is_active.get() { pause() } else { resume() }>
                    {move || if is_active.get() { "Pause" } else { "Resume" }}
                </button>
                 <button class="btn" on:click=move |_| {
                     init_audio();
                    emulator.update_value(|e| e.reload_rom());
                }>
                    "Reset"
                </button>
            </div>

            <canvas
                node_ref=canvas_ref
                width="640"
                height="320"
                class="chip8-canvas"
            />

            <div class="chip8-instructions">
                 <p><strong>"Keypad Mapping"</strong></p>
                 <div class="key-grid">
                    {keys.into_iter().map(|(label, hex_val)| {
                        let is_pressed = move || keypad.get().is_pressed(hex_val as u8);
                        view! {
                            <div
                                class="key"
                                class:pressed=is_pressed
                            >
                                <span class="key-label">{label}</span>
                                <span class="hex-label">{format!("{:X}", hex_val)}</span>
                            </div>
                        }
                    }).collect_view()}
                 </div>
             </div>
         </div>
    }
}

fn draw_screen(ctx: &CanvasRenderingContext2d, screen: &Screen) {
    ctx.set_fill_style_str("#1a1a1a");
    ctx.fill_rect(0.0, 0.0, 640.0, 320.0);

    ctx.set_fill_style_str("#00ff00");
    ctx.begin_path();

    let scale = 10.0;

    for (y, row) in screen.0.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel {
                ctx.rect(x as f64 * scale, y as f64 * scale, scale, scale);
            }
        }
    }
    ctx.fill();
}

pub fn map_key(code: &str) -> Option<u8> {
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
