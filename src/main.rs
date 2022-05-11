use gloo_render::{request_animation_frame, AnimationFrame};
use web_sys::HtmlCanvasElement;
use yew::html::Scope;
use yew::{html, Component, Context, Html, NodeRef};

use crate::UniverseController;
pub enum Msg {
    Render(f64)
}

pub struct GameOfLifeModel {
    controller: Option<UniverseController>,
    node_ref: NodeRef,
    _render_loop: Option<AnimationFrame>,
}

impl Component for GameOfLifeModel{
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        log::info!("GameOfLifeModel::create");
        Self {
            controller: None,
            node_ref: NodeRef::default(),
            _render_loop: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        self.controller = Some(UniverseController::new(canvas, 64, 64));

        if first_render {
            log::info!("GameOfLifeModel::first_render");
            // The callback to request animation frame is passed a time value which can be used for
            // rendering motion independent of the framerate which may vary.
            let handle = {
                let link = ctx.link().clone();
                request_animation_frame(move |time| link.send_message(Msg::Render(time)))
            };

            // A reference to the handle must be stored, otherwise it is dropped and the render
            // won't occur.
            self._render_loop = Some(handle);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render(timestamp) => {
                self.controller.unwrap().render();
                // self.render_framerate(timestap);
            }
        }
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas ref={self.node_ref.clone()} />
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<GameOfLifeModel>::new().render();
}

