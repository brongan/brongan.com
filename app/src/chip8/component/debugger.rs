use gloo_file::futures::read_as_bytes;
use gloo_file::File;
use leptos::ev;
use leptos::html::Canvas;
use leptos::html::Input;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::{use_raf_fn, utils::Pausable, UseRafFnCallbackArgs};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use super::beep::Beep;
use super::chip8_disassembler::Disassembler;
use super::cpu_state::UiSignals;
use super::memory_viewer::MemoryViewer;
use crate::chip8::component::cpu_state::CpuState;
use crate::chip8::emulator::cpu::Keypad;
use crate::chip8::emulator::cpu::Registers;
use crate::chip8::emulator::emulator::Emulator;
use crate::chip8::emulator::screen::Screen;

#[component]
pub fn Debugger() -> impl IntoView {
    let emulator = StoredValue::new_local(Emulator::new(None));

    let ui = UiSignals {
        pc: RwSignal::new(0x200),
        registers: RwSignal::new(Registers::default()),
        index: RwSignal::new(0),
        timers: RwSignal::new((0, 0)),
        stack: RwSignal::new(vec![]),
        memory: RwSignal::new(vec![0; 4096]),
        instruction_count: RwSignal::new(0),
    };

    let quirks = RwSignal::new(emulator.get_value().quirks);

    let keypad = RwSignal::new(Keypad::default());
    let beep = StoredValue::new_local(None::<Beep>);
    let emulator = StoredValue::new_local(Emulator::new(None));
    let canvas_ref = NodeRef::<Canvas>::new();
    let ctx_ref = StoredValue::new_local(None::<CanvasRenderingContext2d>);

    let Pausable {
        pause,
        resume,
        is_active,
    } = use_raf_fn({
        let ui = ui.clone();
        let emulator = emulator.clone();

        move |args: UseRafFnCallbackArgs| {
            let dt = Duration::from_secs_f64(args.delta / 1000.0);
            emulator.update_value(|emulator| {
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

                ui.pc.set(emulator.cpu().get_pc());
                ui.index.set(emulator.cpu().get_index());
                ui.registers.set(emulator.cpu().get_registers().to_owned());
                ui.timers.set((
                    emulator.cpu().get_delay_timer(),
                    emulator.cpu().get_sound_timer(),
                ));
                ui.instruction_count.set(emulator.instruction_counter());
                ui.memory.set(emulator.cpu().get_memory().to_owned());
            });
        }
    });

    let manual_sync = move || {
        emulator.update_value(|emulator| {
            ui.pc.set(emulator.cpu().get_pc());
            ui.registers.set(emulator.cpu().get_registers().to_owned());
            ui.index.set(emulator.cpu().get_index());
            ui.timers.set((
                emulator.cpu().get_delay_timer(),
                emulator.cpu().get_sound_timer(),
            ));
            ui.stack.set(emulator.cpu().get_stack().to_owned());
            ui.memory.set(emulator.cpu().get_memory().to_owned());
        });
    };

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

    view! {
        <header class="header">
            <h1 class="title">{ "Chip-8 Emulator" }</h1>
        </header>
        <div class="debugger-app">
            <div class="panel">
                /*<EmulatorInfo ui=ui.clone() is_active.get()/>*/
                <hr/>
                <CpuState ui=ui.clone() />
            </div>
            <div class="panel">
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
            /*
                <Controls is_running pause resume reset=move || {
                    emulator.update_value(|e| e.reset());
                    manual_sync();
                }/>
            */
                <div class="screen-wrapper" style="border: 1px solid #333; margin: 10px 0;">
                    <canvas
                        node_ref=canvas_ref
                        width="640"
                        height="320"
                        class="chip8-canvas"
                    />
                </div>
                <MemoryViewer memory=ui.memory pc=ui.pc />
            </div>
            <div class="panel panel-right">
                <Disassembler memory=ui.memory pc=ui.pc />
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
