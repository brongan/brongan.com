use crate::game_of_life::GameOfLife;
use crate::routes::ishihara_component::IshiharaPlate;
use crate::routes::mandelbrot_component::MandelbrotModel;
use crate::{
    mandelbrot::Bounds,
    routes::{analytics_component::AnalyticsComponent, catscii_component::Catscii},
};
use leptos::{component, view, CollectView, IntoView};
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
            <a type="image" src={nav_item.thumbnail} href={nav_item.route}/>
            <h3>{ nav_item.title }</h3>
        </div>
    }
}

#[component]
fn nav() -> impl IntoView {
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
        <>
            <header class="header">
                <h1 class="title">{ "Welcome to brongan.com" }</h1>
            </header>
            <p>{"Hello my name is Brennan I like Rust"}</p>
            <Nav/>
        </>
    }
}

#[component]
pub fn root() -> impl IntoView {
    view! {
      <Router>
        <nav>
          /* ... */
        </nav>
          <main class="main-panel">
              <Routes>
                  <Route path="/" view=Home/>
                  <Route path="/ishihara" view=IshiharaPlate/>
                  <Route path="/game-of-ilfe" view=GameOfLife/>
                  <Route path="/mandelbrot" view=||view! { <MandelbrotModel bounds={Bounds {width: 800, height: 500}}/> } />
                  <Route path="/catscii" view=Catscii />
                  <Route path="/analytics" view=AnalyticsComponent />
                  <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
              </Routes>
          </main>
      </Router>
    }
}
