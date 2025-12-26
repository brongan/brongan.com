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
use leptos_use::use_raf_fn;
use leptos_use::utils::Pausable;
use universe::DomBounds;
use web_sys::MouseEvent;

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

    let Pausable {
        pause,
        resume,
        is_active,
    } = use_raf_fn(move |_| {
        set_universe.update(|universe: &mut Universe| universe.tick());
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

            set_universe.update(|universe| {
                if event.shift_key() {
                    universe.insert_pulsar(x, y);
                } else if event.ctrl_key() {
                    universe.insert_glider(x, y);
                } else {
                    universe.toggle_cell(x, y);
                }
            });
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
                        <Show when=move || { !is_active.get() }
                              fallback=move || {
                                  let pause = pause.clone();
                                  view! { <button class="game-button" on:click=move |_| pause()>{ "Stop" }</button> }
                              }>
                            {
                                let resume = resume.clone();
                                view! { <button class="game-button" on:click=move |_| resume()>{ "Start" }</button> }
                            }
                        </Show>
                        <button class="game-button" on:click=tick >{ "Tick" }</button>
                        <button class="game-button" on:click=reset >{ "Reset" }</button>
                        <button class="game-button" on:click=kill_all >{ "KillAll" }</button>
                    </div>
                    <Instructions />
                </section>
            </section>
            <Footer text=String::from("Game of Life - a rust experiment ")/>
        </div>
    }
}
