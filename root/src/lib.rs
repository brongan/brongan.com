use ishihara::IshiharaPlate;
use wasm_game_of_life::GameOfLifeModel;
use yew::{function_component, html, Callback, Html};
use yew_router::prelude::*;

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
    let nav1 = use_navigator().unwrap();
    let nav2 = nav1.clone();
    let nav3 = nav1.clone();

    let home = Callback::from(move |_| nav1.push(&Route::Home));
    let game_of_life = Callback::from(move |_| nav2.push(&Route::GameofLife));
    let ishihara = Callback::from(move |_| nav3.push(&Route::Ishihara));
    html! {
        <div>
            <h1>{ "Home" }</h1>
            <button onclick={home}>{ "Home" }</button>
            <h1>{ "Ishihara Plate Generator" }</h1>
            <button onclick={ishihara}>{ "Ishihara Plate Generator" }</button>
            <h1>{ "Game of Life" }</h1>
            <button onclick={game_of_life}>{ "Game of Life" }</button>
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
