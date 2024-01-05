mod universe;
mod util;
mod webgl;

use crate::game_of_life::universe::{Universe, UniverseRenderer};
use crate::game_of_life::util::Timer;
use crate::game_of_life::webgl::WebGLRenderer;
use leptos::{
    component, create_effect, create_node_ref, create_signal, html::Canvas, view, IntoAttribute,
    IntoView, NodeRef, WriteSignal,
};
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
    let instructions = vec![
        "Click => Toggle the State of a Cell",
        "Shift + Click => Insert a Pulsar",
        "Ctrl + Click => Insert a Glider",
    ];
    let instructions = instructions
        .into_iter()
        .map(|n| view! { <li>{n}</li>})
        .collect::<Vec<_>>();

    let width = 128;
    let height = 64;
    let universe = Universe::new(width, height);

    let UseIntervalReturn {
        counter,
        reset,
        is_active,
        pause,
        resume,
    } = use_interval(16);

    let (button, press_button) = create_signal(None);
    create_effect(move |_| match button() {
        Some(Msg::ToggleCell(x, y)) => {
            universe.toggle_cell(x, y);
        }
        Some(Msg::InsertGlider(x, y)) => {
            universe.insert_glider(x, y);
        }
        Some(Msg::InsertPulsar(x, y)) => {
            universe.insert_pulsar(x, y);
        }
        _ => (),
    });

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
                        <button class="game-button" onclick=move |_| {resume();}>{ "Start" }</button>
                        <button class="game-button" onclick=move |_| {pause();}>{ "Stop" }</button>
                        <button class="game-button" onclick=move |_| {universe.tick();}>{ "Tick" }</button>
                        <button class="game-button" onclick=move |_| {universe.reset(); }>{ "Reset" }</button>
                        <button class="game-button" onclick=move |_| {universe.kill_all()}>{ "KillAll" }</button>
                    </div>
                    <div class="life-instructions">
                        <ul>
                            {instructions}
                        </ul>
                    </div>
                </section>
            </section>
            <footer class="app-footer">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
                <strong class="footer-text">
                    { "Game of Life - a rust experiment " }
                </strong>
            </footer>
        </div>
    }
}
