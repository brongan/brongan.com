use crate::analytics_component::AnalyticsComponent;
use crate::catscii_component::Catscii;
use crate::game_of_life::GameOfLifeModel;
use crate::ishihara_component::IshiharaPlate;
use crate::mandelbrot_component::MandelbrotModel;
use yew::html;
use yew::{function_component, Callback, Html};
use yew_router::prelude::*;

mod analytics_component;
mod catscii_component;
mod color;
mod game_of_life;
mod ishihara;
mod ishihara_component;
mod ishihara_form;
mod mandelbrot;
mod mandelbrot_component;
mod point2d;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/ishihara")]
    Ishihara,
    #[at("/game-of-life")]
    GameofLife,
    #[at("/mandelbrot")]
    Mandelbrot,
    #[at("/catscii/")]
    Catscii,
    #[at("/analytics/")]
    Analytics,
    #[not_found]
    #[at("/404")]
    NotFound,
}

struct Page {
    title: &'static str,
    route: Route,
    thumbnail: &'static str,
}

fn main_panel(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },
        Route::Ishihara => html! { <IshiharaPlate/> },
        Route::GameofLife => html! { <GameOfLifeModel/> },
        Route::Mandelbrot => html! { <MandelbrotModel/> },
        Route::Catscii => html! { <Catscii/> },
        Route::Analytics => html! { <AnalyticsComponent/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Nav)]
fn nav() -> Html {
    let nav_buttons = vec![
        Page {
            title: "Home",
            route: Route::Home,
            thumbnail: "img/brongan.jpg",
        },
        Page {
            title: "Ishihara Plate Generator",
            route: Route::Ishihara,
            thumbnail: "img/color-blind-test.png",
        },
        Page {
            title: "Game of Life",
            route: Route::GameofLife,
            thumbnail: "img/game-of-life.png",
        },
        Page {
            title: "Mandelbrot",
            route: Route::Mandelbrot,
            thumbnail: "img/mandelbrot.png",
        },
        Page {
            title: "Catscii",
            route: Route::Catscii,
            thumbnail: "img/catscii.png",
        },
        Page {
            title: "Analytics",
            route: Route::Analytics,
            thumbnail: "img/analytics.png",
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
                <>
                    <input type="image" onclick={callback} src={nav_button.thumbnail.clone()} />
                    <h3>{ nav_button.title.clone() }</h3>
                </>
            }
        })
        .collect::<Html>();

    html! {
        <div class="nav">
        { nav_buttons }
        </div>
    }
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <>
            <header class="header">
                <h1 class="title">{ "Analytics" }</h1>
            </header>
            <p>{"Hello my name is Brennan I like Rust"}</p>
        </>
    }
}

#[function_component(Root)]
pub fn root() -> Html {
    html! {
        <div class="root">
            <BrowserRouter>
                <Nav/>
                <div class="main-panel">
                    <Switch<Route> render={main_panel} />
                </div>
            </BrowserRouter>
        </div>
    }
}
