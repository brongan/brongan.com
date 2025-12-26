use crate::point2d::Point2D;
use anyhow::anyhow;
use image::{DynamicImage, RgbaImage};
use leptos::html::Canvas;
use leptos::logging::log;
use leptos::prelude::*;
use num::Complex;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::CanvasRenderingContext2d;

type Point2d = Point2D<u32>;

#[component]
pub fn show_mandelbrot(image: image::RgbaImage) -> impl IntoView {
    let canvas_element: NodeRef<Canvas> = NodeRef::new();

    Effect::new(move |_| {
        if let Some(canvas) = canvas_element.get() {
            let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(image.as_raw()),
                image.width(),
                image.height(),
            )
            .map_err(|_| anyhow!("Failed to convert to ImageData."))
            .unwrap();

            canvas.set_width(image.width());
            canvas.set_height(image.height());
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            ctx.put_image_data(&image, 0.0, 0.0).unwrap();
        }
    });

    view! {
        <canvas node_ref=canvas_element />
    }
}

#[component]
fn render_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    log!("generating mandelbrot image");
    let image = generate_mandelbrot(bounds, upper_left, lower_right);
    view! { <ShowMandelbrot image/> }
}

/*
#[component]
fn render_remote_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> impl IntoView {
    log!("requesting mandelbrot image");
    let image = OnceResource::new(mandelbrot_get(bounds, upper_left, lower_right));
    let (_pending, set_pending) = signal(false);

    view! {
        <div>
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match image.get() {
                    Some(Ok(image)) => view! { <ShowMandelbrot image/> }.into_any(),
                    Some(Err(e)) => view! {<p> "Failed to load mandelbrot..." </p> }.into_any(),
                    None => view! { <p>"Loading..."</p> }.into_any(),
                }
            }
            </Transition>
        </div>
    }
}
*/

#[component]
pub fn mandelbrot_model(bounds: Bounds) -> impl IntoView {
    log!("mandelbrot");
    let upper_left = Complex::<f64> {
        re: -1.20,
        im: 0.35,
    };
    let lower_right = Complex::<f64> { re: -1.0, im: 0.20 };

    view! {
        <div class="mandelbrot-container">
            <header class="header">
                <h1> { "Mandelbrot" } </h1>
            </header>
            <div class="mandelbrot-viewer">
                <RenderMandelbrot bounds={bounds} upper_left={upper_left} lower_right={lower_right} />
            </div>
            <p class="mandelbrot-caption">
                "Visualizing the set of complex numbers c for which the function f_c(z) = z^2 + c does not diverge."
            </p>
            <footer class="mandelbrot-footer">
                <p><a href="https://github.com/brongan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </div>
    }
}

#[component]
pub fn mandelbrot() -> impl IntoView {
    view! { <MandelbrotModel bounds={Bounds {width: 800, height: 500}} /> }
}

#[server]
pub async fn mandelbrot_get(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Result<(), ServerFnError> {
    use image::GrayImage;
    use opentelemetry::global;
    use opentelemetry::trace::Tracer;
    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefMutIterator;
    use rayon::iter::ParallelIterator;

    let tracer = global::tracer("");
    let _ = tracer.start("mandelbrot_get");

    let mut image = GrayImage::new(bounds.width, bounds.height);
    let bounds = Bounds {
        width: image.width(),
        height: image.height(),
    };
    image.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let i = i as u32;
        let point = Point2d {
            x: i % bounds.width,
            y: i / bounds.width,
        };
        let point = pixel_to_point(bounds, point, upper_left, lower_right);
        *pixel = match escape_time(point, 255) {
            None => 0,
            Some(count) => 255 - count as u8,
        };
    });
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub width: u32,
    pub height: u32,
}

impl Display for Bounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.width, self.height)
    }
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

impl FromStr for Bounds {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parse_pair(s, 'x')
            .map(|(width, height)| Bounds { width, height })
            .ok_or_else(|| anyhow!("Failed to parse bounds."))
    }
}

pub fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

pub fn pixel_to_point(
    bounds: Bounds,
    pixel: Point2d,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.x as f64 * width / bounds.width as f64,
        im: upper_left.im - pixel.y as f64 * height / bounds.height as f64,
    }
}

fn render(image: &mut RgbaImage, upper_left: Complex<f64>, lower_right: Complex<f64>) {
    let bounds = image.dimensions();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let point = pixel_to_point(
            Bounds {
                width: bounds.0,
                height: bounds.1,
            },
            Point2d { x, y },
            upper_left,
            lower_right,
        );
        let brightness = match escape_time(point, 255) {
            None => 0,
            Some(count) => 255 - count as u8,
        };
        pixel.0[0] = brightness;
        pixel.0[1] = brightness;
        pixel.0[2] = brightness;
        pixel.0[3] = 255;
    }
}

pub fn generate_mandelbrot(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> RgbaImage {
    let mut image = DynamicImage::new_rgba8(bounds.width, bounds.height).to_rgba8();
    render(&mut image, upper_left, lower_right);
    image
}
