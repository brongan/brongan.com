use crate::color::{hex_color, Color};
use crate::ishihara_form::IshiharaArgs;
use crate::ishihara_form::IshiharaInput;
use crate::point2d::{Point2D, UniformPoint2D};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use leptos::html::Canvas;
use leptos::prelude::*;
use rand::Rng;
use rand::distr::uniform::Uniform;
use rand::distr::Distribution;
use rand::rngs::{StdRng, ThreadRng};
use rand::seq::IndexedRandom;
use rusttype::{point, Font, Scale};
use simple_profiler::profile_macro::{self, profile};
use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use strum::EnumIter;
use strum::EnumString;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, ImageData};

type Point2d = Point2D<i32>;

#[derive(Clone, Copy)]
enum IshiharaColor {
    Inside,
    Outside,
}

struct Circle {
    center: Point2d,
    radius: f64,
    ishihara_color: Option<IshiharaColor>,
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.center, self.radius)
    }
}

const FONT_SCALE: f32 = 256.0;

#[component]
pub fn show_plate(ishihara_args: ReadSignal<IshiharaArgs>) -> impl IntoView {
    let canvas_element: NodeRef<Canvas> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(canvas) = canvas_element.get() {
            let args: IshiharaArgs = ishihara_args.get();
            let plate = generate_plate(&args.text, args.blindness);
            let image = ImageData::new_with_u8_clamped_array_and_sh(
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
            ctx.put_image_data(&image.unwrap(), 0.0, 0.0).unwrap();
        }
    });
    view! { <canvas node_ref={canvas_element} /> }
}

#[component]
pub fn ishihara_plate() -> impl IntoView {
    let (ishihara_args, set_ishihara_args) = signal(IshiharaArgs {
        blindness: Blindness::Demonstration,
        text: String::from(""),
    });
    let (display, set_display) = signal(false);

    view! {
        <header class="header">
            <h1> { "Ishihara Plate Generator" } </h1>
        </header>
        <div class="description">
            <p style="display:inline"> { "Randomly Generates a Colorblindness Test Image in your browser! See: "} </p>
            <a href="https://en.wikipedia.org/wiki/Ishihara_test"> {"wikipedia.org/wiki/Ishihara_test"} </a>
        </div>
        <div class="input">
            <IshiharaInput set_data={set_ishihara_args} toggle_display={set_display}/>
        </div>
        <Show
            when=move || { display.get() }
            fallback=|| view! {}>
            <div class="readout">
                <ShowPlate ishihara_args/>
            </div>
        </Show>
        <footer class="footnote">
            <p><a href="https://github.com/HBBrennan/brongan.com" target="_blank">{ "source" }</a></p>
        </footer>
    }
}

fn get_color(color: IshiharaColor, blindness: Blindness, rng: &mut ThreadRng) -> Color {
    match (color, blindness) {
        (IshiharaColor::Inside, Blindness::Demonstration) => hex_color("#f0712a").unwrap().1,
        (IshiharaColor::Outside, Blindness::Demonstration) => hex_color("#2aa790").unwrap().1,
        //Red, Red, Orange, Yellow, Light Red, Light Red, Tan
        (IshiharaColor::Inside, Blindness::RedGreen) => {
            hex_color(
                [
                    "#cf5f47", "#cf5f47", "#fd9500", "#ffd500", "#ee8568", "#ee8568", "#eebd7a",
                ]
                .choose(rng)
                .unwrap(),
            )
            .unwrap()
            .1
        }
        //Dark Green, Green, Light Green
        (IshiharaColor::Outside, Blindness::RedGreen) => {
            hex_color(["#5a8a50", "#a2ab5a", "#c9cc7d"].choose(rng).unwrap())
                .unwrap()
                .1
        }
        (IshiharaColor::Inside, Blindness::BlueYellow) => {
            hex_color(["#0f3179", "#0270bf", "#696983"].choose(rng).unwrap())
                .unwrap()
                .1
        }
        (IshiharaColor::Outside, Blindness::BlueYellow) => {
            hex_color(
                ["#9e6e0c", "#cb850c", "#cb850c", "#ad8b10"]
                    .choose(rng)
                    .unwrap(),
            )
            .unwrap()
            .1
        }
    }
}

#[derive(EnumIter, Clone, Copy, strum::Display, Eq, PartialEq, EnumString, Default, Debug)]
pub enum Blindness {
    Demonstration,
    #[default]
    RedGreen,
    BlueYellow,
}

impl Circle {
    const MAX_RADIUS: f64 = 6.9;
    const MIN_RADIUS: f64 = 3.0;
    const GOAL_AREA_RATIO: f64 = 0.57;
    fn create_circles(x: u32, y: u32, rng: &mut ThreadRng) -> Vec<Circle> {
        let goal_area = Circle::GOAL_AREA_RATIO * x as f64 * y as f64;
        let mut circles: Vec<Circle> = Vec::new();
        let mut area: f64 = 0.0;
        let uniform = Uniform::new(
            Point2d { x: 0, y: 0 },
            Point2d {
                x: x as i32,
                y: y as i32,
            },
        )
        .unwrap();

        //Create circles with random coordinates and radii with size based on its distance from the closest circle
        while area < goal_area {
            let candidate_point = uniform.sample(rng);
            if let Some(radius) = max_allowed_radius(&candidate_point, &circles) {
                area += std::f64::consts::PI * radius.powi(2);
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

    fn draw(&self, image: &mut RgbaImage, rng: &mut rand::rngs::ThreadRng, blindness: Blindness) {
        let color = match &self.ishihara_color {
            Some(color) => get_color(*color, blindness, rng),
            None => hex_color("#ffffff").unwrap().1,
        };

        draw_filled_circle_mut(
            image,
            (self.center.x, self.center.y),
            self.radius as i32,
            Rgba([color.red, color.green, color.blue, 255]),
        );
    }
}

fn max_allowed_radius(candidate_point: &Point2d, circles: &[Circle]) -> Option<f64> {
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
    let font_data = include_bytes!("../../public/Roboto-Regular.ttf");
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

pub fn generate_plate(text: &str, blindness: Blindness) -> RgbaImage {
    log::info!("Generating Plate: {}", text);
    // Get an image buffer from rendering the text
    let mut image = render_text(text);
    let mut rng = rand::rng();

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
        .for_each(|circle| circle.draw(&mut image, &mut rng, blindness));
    image
}

struct CircleGrid {
    pixel_width: u32,
    pixel_height: u32,
    padding: f64,
    cell_size: u32,
    rows: u32,
    columns: u32,
    cells: Box<[Option<f64>]>
}

impl CircleGrid {
    pub fn new(width: u32, height: u32, cell_size: u32, padding: f64) -> CircleGrid {
        let columns = width.div_ceil(cell_size);
        let rows = height.div_ceil(cell_size);

        println!("rows: {}, columns: {}", rows, columns);

        let cells = vec![None; (rows * columns) as usize].into_boxed_slice();

        CircleGrid { pixel_width: width, pixel_height: height, padding, cell_size, rows, columns, cells }
    }

    fn fill(&mut self, row: i32, column: i32, circle: Circle2, max_distance: f64) {
        let mut visited = vec![false; (self.rows * self.columns) as usize];
        let mut queue = VecDeque::new();
        queue.push_back((row, column));

        while queue.len() > 0 {
            let (row, column) = queue.pop_front().unwrap();

            if row < 0 || column < 0 || row as u32 >= self.rows || column as u32 >= self.columns {
                continue;
            }

            let i = self.index(row as u32, column as u32);
            if visited[i] {
                continue;
            }

            visited[i] = true;

            let cell_center = Point2d {
                x: column * self.cell_size as i32 + self.cell_size as i32 / 2,
                y: row * self.cell_size as i32 + self.cell_size as i32 / 2
            };

            // Distance from edge, inside being 0.
            let distance = (circle.center.distance(&cell_center) - circle.radius).max(0.0);

            // Replace distance if current distance is None or new distance is less.
            if self.cells[i].map_or(true, |cell_distance| distance < cell_distance && distance <= max_distance) {
                self.cells[i] = Some(distance);

                // Only spread when distance gets replaced
                queue.push_back((row + 1, column));
                queue.push_back((row, column + 1));
                queue.push_back((row - 1, column));
                queue.push_back((row, column - 1));
            }
        }
    }

    fn index(&self, row: u32, column: u32) -> usize {
        (self.columns * row + column) as usize
    }

    pub fn max_radius(&self, point: Point2d) -> f64 {
        if point.x < 0 || point.y < 0 || point.x >= self.pixel_width as i32 || point.y >= self.pixel_height as i32 {
            panic!("Circle out of bounds: center: ({}, {}), canvas size: ({} , {})", point.x, point.y, self.pixel_width, self.pixel_height);
        }

        let row = point.y as u32/ self.cell_size;
        let column = point.x as u32 / self.cell_size;
        let i = self.index(row, column);

        return self.cells[i].map_or(f64::MAX, |distance| distance - self.padding)
    }

    pub fn insert_circle(&mut self, circle: Circle2, max_distance: f64) -> bool {
        if circle.center.x < 0 || circle.center.y < 0 || circle.center.x >= self.pixel_width as i32 || circle.center.y >= self.pixel_height as i32 {
            panic!("Circle out of bounds: center: ({}, {}), canvas size: ({} , {})", circle.center.x, circle.center.y, self.pixel_width, self.pixel_height);
        }

        let row = circle.center.y as u32/ self.cell_size;
        let column = circle.center.x as u32 / self.cell_size;
        let i = self.index(row, column);

        // Add circle if the nearest circle is < radius + padding away
        if self.cells[i].map_or(true, |distance| distance >= circle.radius + self.padding) {
            self.fill(row as i32, column as i32, circle, max_distance);

            return true;
        };

        false
    }
}

#[derive(Clone, Copy, Debug)]
struct Circle2 {
    center: Point2d,
    radius: f64
}

impl Circle2 {
    pub fn generate_circles(width: u32, height: u32, min_radius: f64, max_radius: f64, padding: f64, coverage: f64, rng: &mut StdRng) -> Vec<Circle2> {
        assert!(coverage >= 0.0 && coverage <= 1.0, "coverage must be between 0 and 1.");

        const MAX_MISSES: usize = 1000;
        const SIZE_VARIATION: f64 = 0.5;

        let total_area = (width * height) as f64;

        let mut grid = CircleGrid::new(width, height, 1, padding);
        let uniform = Uniform::new(
            Point2d { x: 0, y: 0 },
            Point2d {
                x: width as i32,
                y: height as i32,
            },
        )
        .unwrap();

        let mut circles = Vec::new();
        let mut area = 0.0;
        let mut missed = 0;
        let mut count = 0;
        while area / total_area <= coverage && missed < MAX_MISSES {

            count += 1;
            // println!("count: {}", count);
            let center = uniform.sample(rng);
            let max_radius = f64::min(grid.max_radius(center), max_radius);
            let min_radius = f64::max(max_radius * SIZE_VARIATION, min_radius);

            if max_radius < min_radius {
                missed += 1;
                continue;
            }
            let radius = rng.random_range(min_radius..max_radius);
            let circle = Circle2 { center, radius };

            if grid.insert_circle(circle, max_radius + padding * 2.0) {
                circles.push(circle);
                area += std::f64::consts::PI * radius * radius;
                missed = 0;
            } else {
                missed += 1;
            }
        }

        // TODO: Probably remove this
        #[cfg(test)]
        {
            println!("target coverage: {}", coverage);
            println!("actual coverage: {}", area / total_area);
        }

        return circles;
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::create_dir_all, path::Path};

    use image::{Rgba, RgbaImage};
    use imageproc::{drawing::{draw_filled_circle_mut, draw_filled_rect_mut}, rect::Rect};
    use rand::{SeedableRng, rngs::StdRng};
    use simple_profiler::profile_macro::profile;

    use crate::ishihara::Circle2;

    static TEST_DIR: &str = "../test artifacts/";

    type Circles = Vec<Circle2>;

    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    #[profile]
    fn circles() -> Circles {
        let mut rng = StdRng::seed_from_u64(0);

        Circle2::generate_circles(1920, 1080, 8.0, 16.0, 2.0, 0.5, &mut rng)
    }

    #[test]
    pub fn circle_generator() -> Result<(), Box<dyn std::error::Error>> {
        println!("test dir: {:?}", std::path::absolute(Path::new(TEST_DIR))?.as_os_str());
        create_dir_all(TEST_DIR)?;

        let output_file = Path::new(TEST_DIR).join("circle_generator_output.png");

        let circles = circles();

        simple_profiler::profiler::profile_current_thread(simple_profiler::analysis::Sort::TotalTime, simple_profiler::analysis::Unit::Millisecond);

        let mut canvas = RgbaImage::new(WIDTH, HEIGHT);
        draw_filled_rect_mut(&mut canvas, Rect::at(0, 0).of_size(WIDTH, HEIGHT), Rgba([255, 255, 255, 255]));

        for circle in circles {
            let center = (circle.center.x, circle.center.y);
            draw_filled_circle_mut(&mut canvas, center, circle.radius as i32, Rgba([0, 0, 0, 255]));
        }

        canvas.save(output_file)?;

        Ok(())
    }
}