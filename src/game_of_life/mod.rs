mod universe;
mod util;
mod webgl;

use crate::game_of_life::universe::{Universe, UniverseRenderer};
use crate::game_of_life::util::Timer;
use crate::game_of_life::webgl::WebGLRenderer;
use crate::root::Footer;
use leptos::html::Canvas;
use leptos::*;
use leptos_use::{use_interval, UseIntervalReturn};
use std::ops::Deref;
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
pub fn game_of_life_canvas(
    width: u32,
    height: u32,
    press_button: WriteSignal<Option<Msg>>,
) -> impl IntoView {
    log::info!("GameOfLifeModel::create");
    let canvas_element: NodeRef<Canvas> = create_node_ref();
    let renderer = WebGLRenderer::new(
        canvas_element().expect("dom has canvas.").deref().clone(),
        width,
        height,
    )
    .unwrap();

    let insert_pulsar = move |event: MouseEvent| {
        let (x, y) = renderer.get_cell_index(event.x() as u32, event.y() as u32);
        press_button(Some(if event.shift_key() {
            Msg::InsertPulsar(x, y)
        } else if event.ctrl_key() {
            Msg::InsertGlider(x, y)
        } else {
            Msg::ToggleCell(x, y)
        }));
    };

    view! {
        <canvas _ref=canvas_element on:click=insert_pulsar type="text"/>
    }
}

#[component]
pub fn game_of_life() -> impl IntoView {
    let width = 128;
    let height = 64;

    let UseIntervalReturn {
        counter: _,
        reset: _,
        is_active: _,
        pause,
        resume,
    } = use_interval(16);

    let (button, press_button) = create_signal(None);
    let (_universe, set_universe) = create_signal(Universe::new(width, height));
    create_effect(move |_| match button() {
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

    view! {
        <div>
            <section class="life-container">
            <header class="header">
            <h1 class="title">{ "Game of Life" }</h1>
            </header>
            <section class="life-area">
            <div class="game-of-life">
            <GameOfLifeCanvas width={width} height={height} press_button={press_button}/>
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
