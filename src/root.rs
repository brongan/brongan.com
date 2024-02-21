use crate::game_of_life::GameOfLife;
use crate::routes::analytics_component::AnalyticsComponent;
use crate::routes::catscii_component::Catscii;
use crate::routes::ishihara_component::IshiharaPlate;
use crate::routes::mandelbrot_component::Mandelbrot;
use leptos::*;
use leptos_meta::*;
use leptos_router::{Route, Router, Routes};

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
fn navigation() -> impl IntoView {
    let nav_items = vec![
        NavItem {
            title: "Home",
            route: "/",
            thumbnail: "img/brongan.jpg",
        },
        NavItem {
            title: "Ishihara",
            route: "/ishihara",
            thumbnail: "img/color-blind-test.png",
        },
        NavItem {
            title: "Game of Life",
            route: "/game-of-life",
            thumbnail: "img/game-of-life.png",
        },
        NavItem {
            title: "Mandelbrot",
            route: "/mandelbrot",
            thumbnail: "img/mandelbrot.png",
        },
        NavItem {
            title: "Catscii",
            route: "/catscii",
            thumbnail: "img/catscii.png",
        },
        NavItem {
            title: "Analytics",
            route: "/analytics",
            thumbnail: "img/analytics.png",
        },
    ];

    view! {
        <div class="nav">
            { nav_items.into_iter().map(|item| view! { <NavButton nav_item={item}/> }).collect_view()}
        </div>
    }
}

#[component]
fn home() -> impl IntoView {
    view! {
        <header class="header">
            <h1 class="title">{ "Welcome to brongan.com" }</h1>
        </header>
        <p>{"Hello my name is Brennan I like Rust"}</p>
        <Navigation/>
    }
}

#[component]
pub fn root() -> impl IntoView {
    provide_meta_context();
    view! {
      <Router>
        <nav class="sidebar">
          <Navigation/>
        </nav>
        <main class="main-panel">
            <Title text="brongan.com" />
            <Stylesheet href="/pkg/main-dea73183440380c.css"/>
              <Routes>
                  <Route path="/" view=Home/>
                  <Route path="/ishihara" view=IshiharaPlate/>
                  <Route path="/game-of-life" view=GameOfLife/>
                  <Route path="/mandelbrot" view=Mandelbrot /> />
                  <Route path="/catscii" view=Catscii />
                  <Route path="/analytics" view=AnalyticsComponent />
              </Routes>
          </main>
      </Router>
    }
}
