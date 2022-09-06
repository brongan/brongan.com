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

#[function_component(Home)]
fn secure() -> Html {
    let nav1 = use_navigator().unwrap();
    let nav2 = nav1.clone();

    let game_of_life = Callback::from(move |_| nav1.push(&Route::GameofLife));
    let ishihara = Callback::from(move |_| nav2.push(&Route::Ishihara));
    html! {
        <div>
            <h1>{ "Ishihara Plate Generator" }</h1>
            <button onclick={ishihara}>{ "Ishihara Plate Generator" }</button>
            <h1>{ "Game of Life" }</h1>
            <button onclick={game_of_life}>{ "Game of Life" }</button>

        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },
        Route::Ishihara => html! { <IshiharaPlate/> },
        Route::GameofLife => html! { <GameOfLifeModel/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Root)]
pub fn root() -> Html {
    html! {
        <div>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
            <footer class="app-footer">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </div>
    }
}
