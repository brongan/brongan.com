use analytics::AnalyticsComponent;
use catscii::Catscii;
use game_of_life::GameOfLife;
use ishihara::IshiharaPlate;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use mandelbrot::Mandelbrot;

mod analytics;
mod catscii;
mod color;
mod game_of_life;
mod ishihara;
mod ishihara_form;
mod mandelbrot;
mod point2d;

#[cfg(feature = "ssr")]
mod locat;
#[cfg(feature = "ssr")]
pub mod state;

pub fn shell(options: LeptosOptions) -> AnyView {
    view! { <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
    .into_any()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet href="/pkg/start-axum-workspace.css"/>
        <Title text="brongan.com" />
        <Router>
            <main class="main-panel">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("ishihara") view=IshiharaPlate/>
                    <Route path=StaticSegment("/game-of-life") view=GameOfLife/>
                    <Route path=StaticSegment("/mandelbrot") view=Mandelbrot />
                    <Route path=StaticSegment("/catscii") view=Catscii />
                    <Route path=StaticSegment("/analytics") view=AnalyticsComponent />
                </Routes>
            </main>
      </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <header class="header">
            <h1 class="title">{ "Welcome to brongan.com" }</h1>
        </header>
        <p>{"Hello my name is Brennan I like Rust"}</p>
        <Navigation/>
    }
}

struct NavItem {
    title: &'static str,
    route: &'static str,
    thumbnail: &'static str,
}

#[component]
pub fn footer(text: String) -> impl IntoView {
    view! {
        <footer class="app-footer">
            <p><a href="https://github.com/brongan/brongan.com" target="_blank">{ "source" }</a></p>
            <strong class="footer-text">
            { text  }
            </strong>
        </footer>
    }
}

#[component]
fn nav_button(nav_item: NavItem) -> impl IntoView {
    view! {
        <div class="nav-item">
            <a href={nav_item.route}>
                <img src={nav_item.thumbnail} />
            </a>
            <h3>{ nav_item.title }</h3>
        </div>
    }
}

#[component]
pub fn navigation() -> impl IntoView {
    let nav_items = vec![
        NavItem {
            title: "Home",
            route: "/",
            thumbnail: "brongan.jpg",
        },
        NavItem {
            title: "Ishihara",
            route: "/ishihara",
            thumbnail: "color-blind-test.png",
        },
        NavItem {
            title: "Game of Life",
            route: "/game-of-life",
            thumbnail: "game-of-life.png",
        },
        NavItem {
            title: "Mandelbrot",
            route: "/mandelbrot",
            thumbnail: "mandelbrot.png",
        },
        NavItem {
            title: "Catscii",
            route: "/catscii",
            thumbnail: "catscii.png",
        },
        NavItem {
            title: "Analytics",
            route: "/analytics",
            thumbnail: "analytics.png",
        },
    ];

    view! {
        <div class="nav">
            { nav_items.into_iter().map(|item| view! { <NavButton nav_item={item}/> }).collect_view()}
        </div>
    }
}
