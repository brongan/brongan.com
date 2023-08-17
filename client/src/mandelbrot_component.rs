use anyhow::anyhow;
use gloo_net::http::Request;
use image::io::Reader as ImageReader;
use log::info;
use num::Complex;
use shared::mandelbrot::{generate_mandelbrot, Bounds};
use shared::mandelbrot::{MandelbrotRequest, MandelbrotResponse};
use std::io::Cursor;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;
use yew::suspense::use_future;

#[derive(EnumIter, Debug, PartialEq, Copy, Clone, Display, Eq)]
pub enum RenderingOption {
    Server,
    Client,
}

#[derive(Properties, PartialEq)]
pub struct MandelbrotInputProps {
    selected: RenderingOption,
    on_click: Callback<RenderingOption>,
}

#[function_component(MandelbrotInput)]
pub fn mandelbrot_input(
    MandelbrotInputProps { selected, on_click }: &MandelbrotInputProps,
) -> Html {
    RenderingOption::iter()
        .enumerate()
        .map(|(i, option)| {
            let on_select = {
                let on_click = on_click.clone();
                Callback::from(move |_| {
                    on_click.emit(option.clone());
                })
            };
            html! {
                <>
                    <label key={i} onclick={on_select}>{format!("{}", option)}</label>
                </>
            }
        })
        .collect()
}

#[derive(Properties, PartialEq)]
struct RenderProps {
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
}

#[function_component(RenderLocally)]
fn render(
    RenderProps {
        bounds,
        upper_left,
        lower_right,
    }: &RenderProps,
) -> HtmlResult {
    info!("Client is Rendering Mandelbrot");
    let canvas_ref = use_node_ref();
    let fallback = html! {<div>{"Rendering Mandelbrot On Client"}</div>};
    let image = generate_mandelbrot(*bounds, *upper_left, *lower_right);
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_raw()),
        image.width(),
        image.height(),
    )
    .map_err(|_| anyhow!("Failed to convert to ImageData."));

    {
        let bounds = bounds.clone();
        let canvas_ref = canvas_ref.clone();
        use_effect(move || {
            let canvas = canvas_ref.cast::<HtmlCanvasElement>().unwrap();
            canvas.set_width(bounds.width);
            canvas.set_height(bounds.height);
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&image.unwrap(), 0.0, 0.0).unwrap();
        });
    }

    Ok(html! {
        <Suspense fallback={fallback}>
            <canvas ref={canvas_ref}/>
        </Suspense>
    })
}

#[function_component(RenderServer)]
fn render(
    RenderProps {
        bounds,
        upper_left,
        lower_right,
    }: &RenderProps,
) -> HtmlResult {
    info!("Server is Rendering Mandelbrot");
    let canvas_ref = use_node_ref();
    let path = "/api/mandelbrot";
    info!("???");
    let fallback = html! {<div>{"Rendering Mandelbrot On Server: {path}"}</div>};
    let request = serde_json::to_string(&MandelbrotRequest {
        bounds: *bounds,
        upper_left: *upper_left,
        lower_right: *lower_right,
    })
    .unwrap();
    info!("Gonna request.");
    let resp = use_future(|| async {
        info!("Requesting image.");
        Request::get(path)
            .body(request)
            .send()
            .await?
            .json::<MandelbrotResponse>()
            .await
    })?;

    info!("Converting Image to javascript.");
    let image: &Vec<u8> = &resp.as_ref().unwrap().image;
    let image = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_bytes()),
        image.width(),
        image.height(),
    )
    .unwrap();

    {
        let bounds = bounds.clone();
        let canvas_ref = canvas_ref.clone();
        use_effect(move || {
            info!("Setting Canvas.");
            let canvas = canvas_ref.cast::<HtmlCanvasElement>().unwrap();
            canvas.set_width(bounds.width);
            canvas.set_height(bounds.height);
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&image, 0.0, 0.0).unwrap();
        });
    }

    info!("Returning html.");

    Ok(html! {
        <Suspense fallback={fallback}>
            <canvas ref={canvas_ref}/>
        </Suspense>
    })
}

#[derive(Properties, PartialEq)]
pub struct MandelbrotModelProps {
    pub bounds: Bounds,
}

#[function_component(MandelbrotModel)]
pub fn mandelbrot_model(MandelbrotModelProps { bounds }: &MandelbrotModelProps) -> Html {
    let selected_option = use_state(|| None);
    let on_option_select = {
        let selected_option = selected_option.clone();
        Callback::from(move |option: RenderingOption| {
            selected_option.set(Some(option));
            info!("Selected: {option}");
        })
    };

    let selected_option = selected_option.as_ref().unwrap_or(&RenderingOption::Server);
    let upper_left = Complex::<f64> {
        re: -1.20,
        im: 0.35,
    };
    let lower_right = Complex::<f64> { re: -1.0, im: 0.20 };

    let readout = match *selected_option {
        RenderingOption::Server => {
            html! {<RenderServer bounds={*bounds} upper_left={upper_left} lower_right={lower_right}/>}
        }
        RenderingOption::Client => {
            html! {<RenderLocally bounds={*bounds} upper_left={upper_left} lower_right={lower_right}/>}
        }
    };

    html! {
        <>
            <header class="header">
                <h1> { "Mandelbrot" } </h1>
            </header>
            <div class="entry">
                <MandelbrotInput selected={*selected_option} on_click={on_option_select}/>
            </div>
            <div class="readout">
            { readout }
            </div>
            <footer class="footnote">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </>
    }
}
