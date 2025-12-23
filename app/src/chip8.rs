use crate::chip8::emulator::Emulator;
use leptos::html::Canvas;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub mod cpu;
pub mod emulator;
pub mod quirks;
pub mod screen;

#[component]
pub fn Chip8() -> impl IntoView {
    let emulator = Rc::new(RefCell::new(Emulator::new(None)));

    let canvas_ref = NodeRef::<Canvas>::new();
    let (is_running, set_is_running) = signal(false);

    type LoopClosure = Closure<dyn FnMut(f64)>;
    let loop_closure: Rc<RefCell<Option<LoopClosure>>> = Rc::new(RefCell::new(None));
    let request_id = Rc::new(RefCell::new(None::<i32>));

    let start_loop = {
        let emulator = emulator.clone();
        let loop_closure = loop_closure.clone();
        let request_id = request_id.clone();

        move || {
            let emulator = emulator.clone();
            let loop_closure_clone = loop_closure.clone();
            let request_id_clone = request_id.clone();

            *loop_closure.borrow_mut() = Some(Closure::new(move |_timestamp: f64| {
                if is_running.get_untracked() {
                    let dt = std::time::Duration::from_secs_f32(1.0 / 60.0);
                    // Placeholder Keypad for now
                    let current_keys = crate::chip8::cpu::Keypad::default();
                    emulator.borrow_mut().update(current_keys, dt);
                }

                if let Some(canvas) = canvas_ref.get() {
                    let ctx = canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<CanvasRenderingContext2d>()
                        .unwrap();

                    ctx.set_fill_style_str("#1a1a1a");
                    ctx.fill_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

                    let em = emulator.borrow();
                    let screen = em.screen();
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

                let handle = web_sys::window()
                    .unwrap()
                    .request_animation_frame(
                        loop_closure_clone
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap();
                *request_id_clone.borrow_mut() = Some(handle);
            }));

            let handle = web_sys::window()
                .unwrap()
                .request_animation_frame(
                    loop_closure
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
            *request_id.borrow_mut() = Some(handle);
        }
    };

    Effect::new(move |_| {
        start_loop();
    });

    view! {
        <header class="header">
            <h1 class="title">{ "Chip-8 Emulator" }</h1>
        </header>

        <div class="content chip8-container">
            <div class="chip8-controls">
                // Buttons are styled via SCSS .chip8-controls .btn
                <button class="btn">"Load ROM (Placeholder)"</button>
                <button class="btn" on:click=move |_| set_is_running.update(|r| *r = !*r)>
                    {move || if is_running.get() { "Pause" } else { "Resume" }}
                </button>
                 <button class="btn" on:click=move |_| {
                    emulator.borrow_mut().reset();
                    set_is_running.set(false);
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
                <p><strong>Controls:</strong></p>
                <div class="key-grid">
                    <span>1 2 3 4</span> <span>-></span> <span>1 2 3 C</span> <br/>
                    <span>Q W E R</span> <span>-></span> <span>4 5 6 D</span> <br/>
                    <span>A S D F</span> <span>-></span> <span>7 8 9 E</span> <br/>
                    <span>Z X C V</span> <span>-></span> <span>A 0 B F</span>
                </div>
            </div>
        </div>
    }
}
