extern crate rand;

use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use rand::distributions::uniform::Uniform;
use rand::distributions::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rusttype::{point, Font, Scale};
use std::fmt;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::{function_component, html, use_node_ref, Html};

mod color;
use color::{hex_color, Color};

mod point2d;
use point2d::Point2D;

enum IshiharaColor {
    Inside,
    Outside,
}

struct Circle {
    center: Point2D,
    radius: f64,
    ishihara_color: Option<IshiharaColor>,
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.center, self.radius)
    }
}

//Red, Red, Orange, Yellow, Light Red, Light Red, Tan
const RED_GREEN_INSIDE: &[&str] = &[
    "#cf5f47", "#cf5f47", "#fd9500", "#ffd500", "#ee8568", "#ee8568", "#eebd7a",
];

//Dark Green, Green, Light Green
const RED_GREEN_OUTSIDE: &[&str] = &["#5a8a50", "#a2ab5a", "#c9cc7d"];

const FONT_SCALE: f32 = 256.0;

impl Circle {
    const MAX_RADIUS: f64 = 6.9;
    const MIN_RADIUS: f64 = 3.0;
    const GOAL_AREA_RATIO: f64 = 0.57;
    fn create_circles(x: u32, y: u32, rng: &mut ThreadRng) -> Vec<Circle> {
        let goal_area = Circle::GOAL_AREA_RATIO * x as f64 * y as f64;
        let mut circles: Vec<Circle> = Vec::new();
        let mut area: f64 = 0.0;
        let uniform = Uniform::new(
            Point2D { x: 0, y: 0 },
            Point2D {
                x: x as i32,
                y: y as i32,
            },
        );

        //Create circles with random coordinates and radii with size based on its distance from the closest circle
        while area < goal_area {
            let candidate_point = uniform.sample(rng);
            if let Some(radius) = max_allowed_radius(&candidate_point, &circles) {
                area += std::f64::consts::PI * radius.powi(2) as f64;
                let new_circle = Circle {
                    center: candidate_point,
                    radius,
                    ishihara_color: None,
                };
                circles.push(new_circle);
            }
        }
        circles
    }

    fn assign_colors(&mut self, image: &RgbaImage) {
        let pixel = image.get_pixel(self.center.x as u32, self.center.y as u32);
        if pixel.0 == [0, 0, 0, 0] {
            self.ishihara_color = Some(IshiharaColor::Inside);
        } else {
            self.ishihara_color = Some(IshiharaColor::Outside);
        }
    }

    fn draw(&self, image: &mut RgbaImage, rng: &mut rand::rngs::ThreadRng) {
        let (_remainder, color) = match self.ishihara_color {
            Some(IshiharaColor::Inside) => hex_color(RED_GREEN_INSIDE.choose(rng).unwrap()),
            Some(IshiharaColor::Outside) => hex_color(RED_GREEN_OUTSIDE.choose(rng).unwrap()),
            None => hex_color("#ffffff"),
        }
        .unwrap();

        draw_filled_circle_mut(
            image,
            (self.center.x as i32, self.center.y as i32),
            self.radius as i32,
            Rgba([color.red, color.green, color.blue, 255]),
        );
    }
}

fn max_allowed_radius(candidate_point: &Point2D, circles: &[Circle]) -> Option<f64> {
    let mut curr_radius = Circle::MAX_RADIUS;
    for other in circles {
        let edge_distance = candidate_point.distance(&other.center) - other.radius;
        curr_radius = curr_radius.min(edge_distance - 1.0);
        if curr_radius < Circle::MIN_RADIUS {
            return None;
        }
    }
    Some(curr_radius)
}

fn render_text(text: &str) -> RgbaImage {
    let font_data = include_bytes!("../resources/fonts/Roboto-Regular.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale = Scale::uniform(FONT_SCALE);
    let color = Color {
        red: 0,
        green: 0,
        blue: 0,
    }; // black
    let v_metrics = font.v_metrics(scale);

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    // work out the layout size
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    // Create a new rgba image with some padding
    let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                image.put_pixel(
                    // Offset the position by the glyph bounding box
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    // Turn the coverage into an alpha value
                    Rgba([color.red, color.green, color.blue, (v * 255.0) as u8]),
                )
            });
        }
    }
    image
}

pub fn generate_plate(text: &str) -> RgbaImage {
    log::info!("Generating Plate: {}", text);
    // Get an image buffer from rendering the text
    let mut image = render_text(&text);
    let mut rng = rand::thread_rng();

    // Create circles based based on the image buffer's dimensions
    let (x, y) = image.dimensions();
    let mut circles = Circle::create_circles(x, y, &mut rng);

    // Assign circles colors based on rendered text
    circles
        .iter_mut()
        .for_each(|circle| circle.assign_colors(&image));

    // Erase the text on the image buffer
    draw_filled_rect_mut(
        &mut image,
        imageproc::rect::Rect::at(0, 0).of_size(x, y),
        Rgba([255, 255, 255, 255]),
    );

    // Draw Circles
    circles
        .iter()
        .for_each(|circle| circle.draw(&mut image, &mut rng));
    image
}

#[function_component(IshiharaPlate)]
pub fn render_plate() -> Html {
    let canvas_ref = use_node_ref();
    let input_ref = use_node_ref();

    let onclick = {
        let canvas_ref = canvas_ref.clone();
        let input_ref = input_ref.clone();
        move |_| {
            if let (Some(input), Some(canvas)) = (
                input_ref.cast::<HtmlInputElement>(),
                canvas_ref.cast::<HtmlCanvasElement>(),
            ) {
                let plate = generate_plate(&input.value());
                let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(
                    Clamped(plate.as_raw()),
                    plate.width(),
                    plate.height(),
                );
                canvas.set_width(plate.width());
                canvas.set_height(plate.height());
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap();
                ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0)
                    .unwrap();
            }
        }
    };

    html! {
        <main class="ishihara-main">
            <header class="ishihara-header">
                <h1> { "Color Blind Message Encrypter" } </h1>
            </header>
            <div class="description">
                <p style="display:inline"> { "Randomly Generates a Colorblindness Test Image in your browser! See: "} </p>
                <a href="https://en.wikipedia.org/wiki/Ishihara_test"> {"wikipedia.org/wiki/Ishihara_test"} </a>
            </div>
            <form class="ishihara-entry">
                <input ref={input_ref} placeholder="Text to Encrypt" type="string" />
                <button type="button" onclick={onclick}>{ "Encrypt" }</button>
            </form>
            <div class="ishihara-readout">
                <canvas ref={canvas_ref} />
            </div>
            <footer class="ishihara-footnote">
                <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
            </footer>
        </main>
    }
}
