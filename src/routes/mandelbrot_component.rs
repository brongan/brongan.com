use crate::mandelbrot::Bounds;
use anyhow::anyhow;
use image::io::Reader as ImageReader;
use leptos::*;
use leptos_dom::html::Canvas;
use log::info;
use num::Complex;
use std::io::Cursor;
use wasm_bindgen::Clamped;
use web_sys::CanvasRenderingContext2d;

#[component]
pub fn view_mandelbrot(bounds: Bounds) -> impl IntoView {
    #[server(GetMandelbrot, "/api", "Cbor", "mandelbrot")]
    pub async fn get_mandelbrot(
        bounds: Bounds,
        upper_left: Complex<f64>,
        lower_right: Complex<f64>,
    ) -> Result<Vec<u8>, ServerFnError> {
        let image =
            crate::mandelbrot::generate_mandelbrot_multithreaded(bounds, upper_left, lower_right);
        let mut image_bytes: Vec<u8> = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut image_bytes),
                image::ImageOutputFormat::Png,
            )
            .unwrap();
        Ok(image_bytes)
    }

    fn get_mandelbrot_image(image: Vec<u8>) -> Result<image::RgbaImage, ServerFnError> {
        Ok(ImageReader::new(Cursor::new(image))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .into())
    }

    info!("loading mandelbrot image");
    let upper_left = Complex::<f64> {
        re: -1.20,
        im: 0.35,
    };
    let lower_right = Complex::<f64> { re: -1.0, im: 0.20 };
    let image = get_mandelbrot_image(get_mandelbrot(bounds, upper_left, lower_right));
    let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(image.as_raw()),
        image.width(),
        image.height(),
    )
    .map_err(|_| anyhow!("Failed to convert to ImageData."))
    .unwrap();

    let canvas_element: NodeRef<Canvas> = create_node_ref();
    let canvas = canvas_element().expect("has canvas");
    canvas.set_width(image.width());
    canvas.set_height(image.height());
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    ctx.put_image_data(&image, 0.0, 0.0).unwrap();

    view! {
        <canvas ref=canvas_element type="text"/>
    }
    let mandelbrot = create_resource(
        || (),
        move |_| async move {
            get_mandelbrot(bounds, upper_left, lower_right)
                .await
                .unwrap()
        },
    );
    let (_pending, set_pending) = create_signal(false);

    view! {
        <div class="ascii-art">
            <Transition
                fallback=move || view! {  <p>"Loading..."</p>}
                set_pending
            >
            {
                move || match mandelbrot.get() {
                    Some(image) => {
                        let image = get_mandelbrot_image(image).unwrap();
                        view! { <ShowMandelbrot image/> }.into_view()
                },
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }
            }
            </Transition>
        </div>
    }
}

#[component]
pub fn mandelbrot() -> impl IntoView {
    view! {
        <header class="header">
            <h1> { "Mandelbrot" } </h1>
        </header>
        <div class="readout">
             <LoadMandelbrot bounds={Bounds {width: 800, height: 500}} />
        </div>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}
