mod universe;
mod util;
mod webgl;

use crate::game_of_life::universe::{Universe, UniverseRenderer};
use crate::game_of_life::util::Timer;
use crate::game_of_life::webgl::WebGLRenderer;
use gloo_events::EventListener;
use gloo_timers::callback::Interval;
use std::default::Default;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlCanvasElement;
use yew::events::Event;
use yew::{html, Component, Context, Html, NodeRef};

pub enum Msg {
    Reset,
    Tick,
    Start,
    Stop,
    KillAll,
    ToggleCell(u32, u32),
    InsertGlider(u32, u32),
    InsertPulsar(u32, u32),
}

pub struct GameOfLifeModel {
    renderer: Option<Box<dyn UniverseRenderer>>,
    universe: Universe,
    width: u32,
    height: u32,
    node_ref: NodeRef,
    _interval: Option<Interval>,
    _listener: Option<EventListener>,
}

impl Component for GameOfLifeModel {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        log::info!("GameOfLifeModel::create");
        let node_ref = NodeRef::default();
        let width = 128;
        let height = 64;

        Self {
            renderer: None,
            universe: Universe::new(width, height),
            width,
            height,
            node_ref,
            _interval: None,
            _listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reset => {
                self.universe.reset();
                log::info!("Reset");
            }
            Msg::Tick => {
                self.universe.tick();
            }
            Msg::Start => {
                let callback = ctx.link().callback(|_| Msg::Tick);
                self._interval = Some(Interval::new(16, move || callback.emit(())));
            }
            Msg::Stop => {
                self._interval = None;
            }
            Msg::KillAll => {
                self.universe.kill_all();
            }
            Msg::ToggleCell(x, y) => {
                if let Some(renderer) = &mut self.renderer {
                    let (x, y) = renderer.get_cell_index(x, y);
                    self.universe.toggle_cell(x, y);
                }
            }
            Msg::InsertGlider(x, y) => {
                if let Some(renderer) = &mut self.renderer {
                    let (x, y) = renderer.get_cell_index(x, y);
                    self.universe.insert_glider(x, y);
                }
            }
            Msg::InsertPulsar(x, y) => {
                if let Some(renderer) = &mut self.renderer {
                    let (x, y) = renderer.get_cell_index(x, y);
                    self.universe.insert_pulsar(x, y);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <section class="life-container">
                <header class="life-header">
                    <h1 class="life-title">{ "Game of Life" }</h1>
                </header>
                    <section class="life-area">
                        <div class="game-of-life">
                            <canvas ref={self.node_ref.clone()} />
                        </div>
                        <div class="life-buttons">
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Start)}>{ "Start" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Stop)}>{ "Stop" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Tick)}>{ "Tick" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Reset)}>{ "Reset" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::KillAll)}>{ "Kill" }</button>
                        </div>
                        <div class="life-instructions">
                            <ul>
                            {["Click => Toggle the State of a Cell", "Shift + Click => Insert a Pulsar", "Ctrl + Click => Insert a Glider"].iter().collect::<Html>()}
                            </ul>
                        </div>
                    </section>
                </section>
                <footer class="app-footer">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
                <strong class="footer-text">
                    { "Game of Life - a yew experiment " }
                </strong>
                </footer>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let insert_pulsar = ctx.link().callback(|e: Event| {
                let event = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
                if event.shift_key() {
                    Msg::InsertPulsar(event.x() as u32, event.y() as u32)
                } else if event.ctrl_key() {
                    Msg::InsertGlider(event.x() as u32, event.y() as u32)
                } else {
                    Msg::ToggleCell(event.x() as u32, event.y() as u32)
                }
            });
            let listener =
                EventListener::new(&canvas, "click", move |e| insert_pulsar.emit(e.clone()));

            self._listener = Some(listener);
            self.renderer = Some(Box::new(
                WebGLRenderer::new(canvas, self.width, self.height).unwrap(),
            ));
        }
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.universe);
        }
    }
}
