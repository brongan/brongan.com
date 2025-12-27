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

    let universe = StoredValue::new_local(Universe::new(width, height));
    let renderer = StoredValue::new_local(None::<WebGLRenderer>);
    let canvas: NodeRef<Canvas> = NodeRef::new();

    let render = move || {
        renderer.update_value(|r| {
            if let Err(_) = if let Some(r) = r {
                universe.with_value(|u| r.render(u))
            } else {
                Err(())
            } {
                log!("Context lost or renderer missing! Recreating...");
                if let Some(canvas_ref) = canvas.get() {
                    let mut new_renderer = WebGLRenderer::new(canvas_ref, width, height);
                    let _ = universe.with_value(|u| new_renderer.render(u));
                    *r = Some(new_renderer);
                }
            }
        });
    };

    Effect::new(move |_| {
        // Initial creation when canvas mounts
        if let Some(canvas) = canvas.get() {
            if renderer.with_value(|r| r.is_none()) {
                renderer.set_value(Some(WebGLRenderer::new(canvas, width, height)));
                render();
            }
        }
    });

    let Pausable {
        pause,
        resume,
        is_active,
    } = use_raf_fn(move |_| {
        universe.update_value(|universe| universe.tick());
        render();
    });

    let tick = move |_| {
        universe.update_value(|universe| universe.tick());
        render();
    };
    let reset = move |_| {
        universe.update_value(|universe| universe.reset());
        render();
    };
    let kill_all = move |_| {
        universe.update_value(|universe| universe.kill_all());
        render();
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

            universe.update_value(|universe| {
                if event.shift_key() {
                    universe.insert_pulsar(x, y);
                } else if event.ctrl_key() {
                    universe.insert_glider(x, y);
                } else {
                    universe.toggle_cell(x, y);
                }
            });
            render();
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
