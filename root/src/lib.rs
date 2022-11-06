use crate::ishihara::IshiharaPlate;
use wasm_game_of_life::GameOfLifeModel;
use yew::{function_component, html, Callback, Html};
use yew_router::prelude::*;

mod ishihara;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/ishihara")]
    Ishihara,
    #[at("/game-of-life")]
    GameofLife,
    #[not_found]
    #[at("/404")]
    NotFound,
}

struct Page {
    id: usize,
    title: String,
    route: Route,
}

fn main_panel(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },
        Route::Ishihara => html! { <IshiharaPlate/> },
        Route::GameofLife => html! { <GameOfLifeModel/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Nav)]
fn nav() -> Html {
    let nav_buttons = vec![
        Page {
            id: 1,
            title: "Home".to_string(),
            route: Route::Home,
        },
        Page {
            id: 2,
            title: "Colorblind Message Encrypter".to_string(),
            route: Route::Ishihara,
        },
        Page {
            id: 3,
            title: "Game of Life".to_string(),
            route: Route::GameofLife,
        },
    ];

    let nav = use_navigator().unwrap();

    let nav_buttons = nav_buttons
        .iter()
        .map(|nav_button| {
            let nav = nav.clone();
            let route = nav_button.route.clone();
            let callback = Callback::from(move |_| nav.push(&route));
            html! {
                <div>
                    <button onclick={callback}>{ nav_button.title.clone() }</button>
                    <h1>{ nav_button.title.clone() }</h1>
                </div>
            }
        })
        .collect::<Html>();

    html! {
        <div>
        { nav_buttons }
        </div>
    }
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <p>{"Hello my name is Brennan I like Rust"}</p>
    }
}

#[function_component(Root)]
pub fn root() -> Html {
    html! {
        <div class="Root">
            <BrowserRouter>
                <Nav/>
                <Switch<Route> render={main_panel} />
            </BrowserRouter>
        </div>
    }
}
