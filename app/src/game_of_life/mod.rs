mod universe;
mod util;
mod webgl;

use crate::game_of_life::universe::{Universe, UniverseRenderer};
use crate::game_of_life::util::Timer;
use crate::game_of_life::webgl::WebGLRenderer;
use crate::mandelbrot::Bounds;
use crate::point2d::Point2D;
use crate::Footer;
use leptos::html::Canvas;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::{use_interval, UseIntervalReturn};
use universe::DomBounds;
use web_sys::MouseEvent;

#[derive(Clone)]
pub enum Msg {
    ToggleCell(u32, u32),
    InsertGlider(u32, u32),
    InsertPulsar(u32, u32),
}

#[component]
pub fn instructions() -> impl IntoView {
    let instructions = vec![
        "Click => Toggle the State of a Cell",
        "Shift + Click => Insert a Pulsar",
        "Ctrl + Click => Insert a Glider",
    ];
    let instructions = instructions
        .into_iter()
        .map(|n| view! { <li>{n}</li>})
        .collect::<Vec<_>>();

    view! {
        <div class="life-instructions">
            <ul>
                {instructions}
            </ul>
        </div>
    }
}

#[component]
pub fn game_of_life() -> impl IntoView {
    let width = 128;
    let height = 64;
    let (button, press_button) = signal(None);
    let (universe, set_universe) = signal(Universe::new(width, height));

    let canvas: NodeRef<Canvas> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(canvas) = canvas.get() {
            log!("Creating renderer");
            let mut renderer = WebGLRenderer::new(canvas, width, height);
            universe.with(|universe| renderer.render(universe));
            log!("Created renderer");
        }
    });

    let UseIntervalReturn {
        counter,
        reset: _,
        is_active: _,
        pause,
        resume,
    } = use_interval(16);

    match button() {
        Some(Msg::ToggleCell(x, y)) => {
            set_universe.update(|universe| universe.toggle_cell(x, y));
        }
        Some(Msg::InsertGlider(x, y)) => {
            set_universe.update(|universe| universe.insert_glider(x, y));
        }
        Some(Msg::InsertPulsar(x, y)) => {
            set_universe.update(|universe| universe.insert_pulsar(x, y));
        }
        _ => (),
    };

    Effect::new(move |_| {
        if counter.get() > 0 {
            set_universe.update(|universe: &mut Universe| universe.tick());
        }
    });

    let tick = move |_| {
        set_universe.update(|universe: &mut Universe| universe.tick());
    };
    let reset = move |_| {
        set_universe.update(|universe| universe.reset());
    };
    let kill_all = move |_| {
        set_universe.update(|universe| universe.kill_all());
    };

    let on_click = move |event: MouseEvent| {
        if let Some(canvas) = canvas.get() {
            log!("On Click!");
            let rect = canvas.get_bounding_client_rect();
            let bounding_rect = DomBounds {
                origin: Point2D {
                    x: rect.x(),
                    y: rect.y(),
                },
                width: rect.width(),
                height: rect.height(),
            };
            let (x, y) = WebGLRenderer::get_cell_index(
                bounding_rect,
                Bounds { width, height },
                Point2D::<i32> {
                    x: event.x(),
                    y: event.y(),
                },
            );
            press_button(Some(if event.shift_key() {
                Msg::InsertPulsar(x, y)
            } else if event.ctrl_key() {
                Msg::InsertGlider(x, y)
            } else {
                Msg::ToggleCell(x, y)
            }));
        }
    };

    view! {
        <div>
            <section class="life-container">
                <header class="header">
                    <h1 class="title">{ "Game of Life" }</h1>
                </header>
                <section class="life-area">
                    <div class="game-of-life">
                        <canvas node_ref=canvas on:click=on_click />
                    </div>
                    <div class="life-buttons">
                        <button class="game-button" on:click=move |_| {resume();}>{ "Start" }</button>
                        <button class="game-button" on:click=move |_| {pause();}>{ "Stop" }</button>
                        <button class="game-button" on:click=tick >{ "Tick" }</button>
                        <button class="game-button" on:click=reset >{ "Reset" }</button>
                        <button class="game-button" on:click=kill_all >{ "KillAll" }</button>
                    </div>
                </section>
            </section>
            <Footer text=String::from("Game of Life - a rust experiment ")/>
        </div>
    }
}
