use gloo_file::futures::read_as_bytes;
use gloo_file::File;
use leptos::ev;
use leptos::html::Canvas;
use leptos::html::Input;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::{use_raf_fn, utils::Pausable, UseRafFnCallbackArgs};
use std::time::Duration;
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use super::beep::Beeper;
use super::chip8_disassembler::Disassembler;
use super::colors::ColorSettings;
use super::controls::Controls;
use super::cpu_state::CpuState;
use super::emulator_info::EmulatorInfo;
use super::keypad_component::KeypadComponent;
use super::memory_viewer::MemoryViewer;
use super::quirk_settings::QuirkSettings;
use super::stack_viewer::StackViewer;
use crate::chip8::emulator::cpu::Keypad;
use crate::chip8::emulator::emulator::Emulator;
use crate::chip8::emulator::screen::Screen;

#[component]
pub fn Debugger() -> impl IntoView {
    let emulator = Emulator::new(None);
    let (pc, set_pc) = signal(emulator.cpu().get_pc());
    let (registers, set_registers) = signal(emulator.cpu().get_registers().to_owned());
    let (index, set_index) = signal(emulator.cpu().get_index());
    let (sound_timer, set_sound_timer) = signal(emulator.cpu().get_sound_timer());
    let (delay_timer, set_delay_timer) = signal(emulator.cpu().get_delay_timer());
    let (stack, set_stack) = signal(emulator.cpu().get_stack());
    let (sp, set_sp) = signal(emulator.cpu().get_sp());
    let (memory, set_memory) = signal(emulator.cpu().get_memory().to_owned());
    let (instruction_count, set_instruction_count) = signal(emulator.instruction_counter());

    let emulator = StoredValue::new_local(emulator);

    let quirks = RwSignal::new(emulator.get_value().quirks);
    Effect::new(move |_| {
        let new_settings = quirks.get();
        emulator.update_value(|emu| {
            emu.quirks = new_settings;
        });
    });

    let keypad = RwSignal::new(Keypad::default());
    let beeper = StoredValue::new_local(None::<Beeper>);
    let canvas_ref = NodeRef::<Canvas>::new();
    let ctx_ref = StoredValue::new_local(None::<CanvasRenderingContext2d>);

    let (rom_name, set_rom_name) = signal(None);
    let (fps, set_fps) = signal(60.0);
    let (frame_time, set_frame_time) = signal(Duration::default());
    let (beep, set_beep) = signal(false);
    let on_color = RwSignal::new("#000000".to_string());
    let off_color = RwSignal::new("#FFFFFF".to_string());
    let debug_mode = RwSignal::new(false);

    let sync = move || {
        emulator.with_value(|emulator| {
            set_pc(emulator.cpu().get_pc());
            set_instruction_count(emulator.instruction_counter());

            if debug_mode.get() {
                set_registers(emulator.cpu().get_registers().to_owned());
                set_index(emulator.cpu().get_index());
                set_delay_timer(emulator.cpu().get_delay_timer());
                set_sound_timer(emulator.cpu().get_sound_timer());
                set_stack(emulator.cpu().get_stack());
                set_sp(emulator.cpu().get_sp());
                set_memory(emulator.cpu().get_memory().to_owned());
            }
        });
    };



    let Pausable {
        pause,
        resume,
        is_active,
    } = use_raf_fn({
        let emulator = emulator;

        move |args: UseRafFnCallbackArgs| {
            let dt = Duration::from_secs_f64(args.delta / 1000.0);
            set_frame_time(dt);
            set_fps(1000.0 / dt.as_millis_f64());
            emulator.update_value(|emulator| {
                emulator.update(keypad.get(), dt);
                if let Some(audio) = beeper.get_value() {
                    if emulator.is_beep() {
                        set_beep(true);
                        audio.play();
                    } else {
                        set_beep(false);
                        audio.pause();
                    }
                }
                ctx_ref.with_value(|ctx| {
                    if let Some(ctx) = ctx {
                        draw_screen(ctx, emulator.screen(), on_color.get(), off_color.get());
                    }
                });
            });
            sync();
        }
    });

    // When switching to debug mode while paused, ensure we sync once
    Effect::new(move |_| {
        if debug_mode.get() && !is_active.get() {
            sync();
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
        let on = on_color.get();
        let off = off_color.get();
        ctx_ref.with_value(|ctx| {
            if let Some(ctx) = ctx {
                emulator.with_value(|emu| {
                    draw_screen(ctx, emu.screen(), on.clone(), off.clone());
                });
            }
        });
    });

    let selected_rom_url = RwSignal::new(String::new());

    let file_input = NodeRef::<Input>::new();
    let on_file_upload = move |ev: ev::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);
        selected_rom_url.set(String::new());

        if let Some(file) = input.files().and_then(|files| files.get(0)).map(File::from) {
            spawn_local(async move {
                match read_as_bytes(&file).await {
                    Ok(bytes) => {
                        leptos::logging::log!("ROM loaded: {} bytes", bytes.len());
                        set_rom_name(Some(file.name()));
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
        beeper.update_value(|b| {
            if b.is_none() {
                if let Ok(new_beep) = Beeper::new() {
                    *b = Some(new_beep);
                }
            }
            // Always try to resume context (browser requirement)
            if let Some(audio) = b {
                audio.resume_context();
            }
        });
    };



    let reset = move || {
        emulator.update_value(|e| e.reset());
        sync();
    };

    let step = move |steps: u32| {
        emulator.update_value(|e| e.step(keypad.get(), steps));
        sync();
    };

    let roms: Vec<(&'static str, &'static str)> = vec![
        ("IBM Logo", "/roms/IBMLogo.ch8"),
        ("Pong", "/roms/Pong.ch8"),
        ("Brix", "/roms/Brix.ch8"),
    ];

    let on_rom_select = {
        let resume = resume.clone();
        let init_audio = init_audio.clone();

        move |url: String| {
            let resume = resume.clone();
            init_audio();
            selected_rom_url.set(url.clone());

            spawn_local(async move {
                match Request::get(&url).send().await {
                     Ok(res) => {
                          if !res.ok() {
                               leptos::logging::error!("Failed to fetch ROM: Status {}", res.status());
                               return;
                          }
                          match res.binary().await {
                              Ok(bytes) => {
                                  set_rom_name(Some(url));
                                  emulator.update_value(|emulator| {
                                      emulator.reset();
                                      emulator.update_rom(bytes);
                                  });
                                  resume();
                              },
                              Err(e) => leptos::logging::error!("Failed to get bytes: {:?}", e),
                          }
                     },
                     Err(e) => leptos::logging::error!("Failed to fetch ROM: {:?}", e),
                }
            });
        }
    };

    view! {
        <div
            class="debugger-app"
            class:mode-debug=move || debug_mode.get()
            class:mode-game=move || !debug_mode.get()
        >
            // --- COL STATE ---
            <Show when=move || debug_mode.get()>
                <div class="panel col-state">
                    <EmulatorInfo is_active rom_name fps frame_time instruction_count beep />
                    <hr class="divider"/>
                    <CpuState pc registers index delay_timer sound_timer memory />
                    <hr class="divider"/>
                    <StackViewer stack sp />
                </div>
            </Show>

            // --- COL SCREEN & MEMORY ---
            <div class="panel col-canvas">
                <div class="screen-wrapper">
                    <canvas
                        node_ref=canvas_ref
                        width="640"
                        height="320"
                        class="chip8-canvas"
                    />
                </div>
                <hr class="divider"/>
                <Show when=move || debug_mode.get()>
                     <div class="memory-wrapper">
                        <MemoryViewer memory pc />
                     </div>
                </Show>
            </div>

            // --- COL CONTROLS ---
            <div class="panel col-controls">
                <div class="panel-header">"Controls"</div>
                <input
                    type="file"
                    node_ref=file_input
                    on:change=on_file_upload
                    style="display: none"
                    accept=".ch8,.rom"
                />
                <Controls is_active pause resume step reset
                roms=roms
                on_rom_select
                selected_rom_url
                debug_mode
                load=move |_| {
                    init_audio();
                    if let Some(input) = file_input.get() {
                        input.click();
                    }
                }
                />

                <KeypadComponent keypad />

                <Show when=move || debug_mode.get()>
                    <hr class="divider"/>
                    <div class="panel-header">"Quirks / Compatibility"</div>
                    <QuirkSettings quirks />
                    <hr class="divider"/>
                    <div class="panel-header">"Display Colors"</div>
                    <ColorSettings on_color off_color />
                </Show>
            </div>

            // --- COL DISASSEMBLY ---
            <Show when=move || debug_mode.get()>
                <div class="panel col-disassembly">
                    <Disassembler memory pc />
                </div>
            </Show>
        </div>
    }
}

fn draw_screen(
    ctx: &CanvasRenderingContext2d,
    screen: &Screen,
    on_color: String,
    off_color: String,
) {
    ctx.set_fill_style_str(&on_color);
    ctx.fill_rect(0.0, 0.0, 640.0, 320.0);

    ctx.set_fill_style_str(&off_color);
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
