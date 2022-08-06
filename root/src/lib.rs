use yew::{function_component, html, Html};
use ishihara::IshiharaPlate;
use wasm_game_of_life::GameOfLifeModel;

#[function_component(Root)]
pub fn root() -> Html {
    html! {
        <div>
            <GameOfLifeModel/>
            <IshiharaPlate/>
            <footer class="app-footer">
                <strong class="footer-text">
                    { "Welcome to my website!" }
                </strong>
                <a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a>
            </footer>
        </div>
    }
}
