use crate::color::{hex_color, Color};
use crate::ishihara_form::IshiharaArgs;
use crate::ishihara_form::IshiharaInput;
use crate::point2d::Point2D;
use image::imageops::FilterType::Lanczos3;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use leptos::html::Canvas;
use leptos::prelude::*;
use rand::{Rng, RngCore};
use rand::distr::uniform::Uniform;
use rand::distr::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rusttype::{Font, Rect, Scale, point};
use std::{f64, fmt};
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

#[derive(Clone, Copy)]
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
    fn assign_colors(&mut self, image: &RgbaImage) {
        let pixel = image.get_pixel(self.center.x as u32, self.center.y as u32);
        if pixel.0 == [0, 0, 0, 0] {
            self.ishihara_color = Some(IshiharaColor::Inside);
        } else {
            self.ishihara_color = Some(IshiharaColor::Outside);
        }
    }

    fn draw(&self, image: &mut RgbaImage, upscaling_factor: u32, rng: &mut rand::rngs::ThreadRng, blindness: Blindness) {
        let upscaling_factor = upscaling_factor as i32;

        let color = match &self.ishihara_color {
            Some(color) => get_color(*color, blindness, rng),
            None => hex_color("#ffffff").unwrap().1,
        };

        draw_filled_circle_mut(
            image,
            (self.center.x * upscaling_factor, self.center.y * upscaling_factor),
            self.radius as i32 * upscaling_factor,
            Rgba([color.red, color.green, color.blue, 255]),
        );
    }
}

fn render_text(text: &str) -> RgbaImage {
    const PADDING: u32 = 50;

    let font_data = include_bytes!("../../public/Roboto-Regular.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale = Scale::uniform(FONT_SCALE);
    let color = Color {
        red: 0,
        green: 0,
        blue: 0,
    }; // black

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = font
        .layout(text, scale, point(0.0, 0.0))
        .collect();

    // work out the layout bounds
    let glyphs_bounds = glyphs
        .iter()
        .filter_map(|g| g.pixel_bounding_box())
        .reduce(|a, b| {
            let min = point(a.min.x.min(b.min.x), a.min.y.min(b.min.y));
            let max = point(a.max.x.max(b.max.x), a.max.y.max(b.max.y));

            Rect { min, max }
        })
        .unwrap();

    // Create a new rgba image with some padding
    let mut image = DynamicImage::new_rgba8(glyphs_bounds.width() as u32 + PADDING * 2, glyphs_bounds.height() as u32 + PADDING * 2).to_rgba8();

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                // Offset the position by the glyph bounding box and padding
                // Subtract glyphs_bounds.min to move the origin to (0, 0)
                // glyphs_bounds ends up negative, so we need to use i32
                let x = PADDING as i32 + x as i32 + bounding_box.min.x - glyphs_bounds.min.x;
                let y = PADDING as i32 + y as i32 + bounding_box.min.y - glyphs_bounds.min.y;
                
                image.put_pixel(
                    // These should always be positive, but just in case.
                    x.try_into().unwrap(),
                    y.try_into().unwrap(),
                    // Turn the coverage into an alpha value
                    Rgba([color.red, color.green, color.blue, (v * 255.0) as u8]),
                )
            });
        }
    }
    image
}

pub fn generate_plate(text: &str, blindness: Blindness) -> RgbaImage {
    const AA_FACTOR: u32 = 2;
    log::info!("Generating Plate: {}", text);
    // Get an image buffer from rendering the text
    let mut image = render_text(text);
    let mut rng = rand::rng();

    // Create circles based based on the image buffer's dimensions
    let (x, y) = image.dimensions();
    let mut circles = CircleGenerator::new(&mut rng)
        .size(x, y)
        .generate();

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

    // Create new buffer to draw upscaled circles
    let mut image = ImageBuffer::new(x * AA_FACTOR, y * AA_FACTOR);
    draw_filled_rect_mut(
        &mut image,
        imageproc::rect::Rect::at(0, 0).of_size(x * AA_FACTOR, y * AA_FACTOR),
        Rgba([255, 255, 255, 255]),
    );

    // Draw Circles
    circles
        .iter()
        .for_each(|circle| circle.draw(&mut image, AA_FACTOR, &mut rng, blindness));

    // Shrink image to original size
    // Supposedly Lanczos3 produces good image quality when downscaling.
    image::imageops::resize(&image, x, y, Lanczos3)
}

struct CircleGrid {
    width: u32,
    height: u32,
    edge_bias: f64,
    cells: Box<[Option<f64>]>
}

impl CircleGrid {
    pub fn new(width: u32, height: u32, edge_bias: f64) -> CircleGrid {
        let cells = vec![None; (width * height) as usize].into_boxed_slice();

        CircleGrid { width, height, edge_bias, cells }
    }

    fn fill(&mut self, point: Point2d, circle: Circle, max_distance: f64) {
        // My code was too slow, so I just yoinked this from wikipedea
        // https://en.wikipedia.org/wiki/Flood_fill#cite_ref-90Heckbert_8-0

        let Point2d { x, y } = point;

        fn distance(this: &CircleGrid, x: i32, y: i32, circle: &Circle) -> f64 {
            // Distance from edge of circle, inside being negative.
            let distance = circle.center.distance(&Point2d { x, y }) - circle.radius;

            // Distance from edge of canvas
            let edge_distance = [
                // Left edge
                x as u32,
                // Right edge
                this.width - x as u32,
                // Top edge
                y as u32,
                // Bottom edge
                this.height - y as u32
            ].into_iter().min().unwrap();

            // Min with edge_distance to prevent the circles from lining up on the edges of the canvas.
            distance.min(edge_distance as f64 * this.edge_bias)
        }

        fn inside(this: &CircleGrid, x: i32, y: i32, circle: &Circle, max_distance: &f64) -> bool {
            if x < 0 || y < 0 || x as u32 >= this.width || y as u32 >= this.height {
                return false;
            }

            let distance  = distance(this, x, y, circle);

            this.get((x, y)).is_none_or(|cell_distance| distance < cell_distance && distance <= *max_distance)
        }

        fn set(this: &mut CircleGrid, x: i32, y: i32, circle: &Circle) {
            if x < 0 || y < 0 || x as u32 >= this.width || y as u32 >= this.height {
                return;
            }

            *this.get_mut((x, y)) = Some(distance(this, x, y, circle));
        }

        if !inside(self, x, y, &circle, &max_distance) {
            return
        }

        let mut stack = Vec::new();
        stack.push((x, x, y, 1));
        stack.push((x, x, y - 1, -1));
        while  let Some((mut x1, x2, y, dy)) = stack.pop() {
            let mut x = x1;
            if inside(self, x, y, &circle, &max_distance) {
                while inside(self, x - 1, y, &circle, &max_distance) {
                    set(self, x - 1, y, &circle);
                    x -= 1;
                }
                if x < x1 {
                    stack.push((x, x1 - 1, y - dy, -dy));
                }
            }
            while x1 <= x2 {
                while inside(self, x1, y, &circle, &max_distance) {
                    set(self, x1, y, &circle);
                    x1 += 1;
                }
                if x1 > x {
                    stack.push((x, x1 - 1, y + dy, dy));
                }
                if x1 - 1 > x2 {
                    stack.push((x2 + 1, x1 - 1, y - dy, -dy));
                }
                x1 += 1;
                while x1 <= x2 && !inside(self, x1, y, &circle, &max_distance) {
                    x1 += 1;
                }
                x = x1;
            }
        }
    }

    // Maybe these should return an Option/Result instead of panicking.
    fn get<P: Into<Point2d>>(&self, point: P) -> &Option<f64> {
        let point = point.into();
        if point.x < 0 || point.y < 0 || point.x as u32 >= self.width || point.y as u32 >= self.height {
            panic!("Point ({}, {}) is out of bounds: ({}, {}).", point.x, point.y, self.width, self.height);
        }

        let i = self.width as usize * point.y as usize + point.x as usize;
        &self.cells[i]
    }

    fn get_mut<P: Into<Point2d>>(&mut self, point: P) -> &mut Option<f64> {
        let point = point.into();
        if point.x < 0 || point.y < 0 || point.x as u32 >= self.width || point.y as u32 >= self.height {
            panic!("Point ({}, {}) is out of bounds: ({}, {}).", point.x, point.y, self.width, self.height);
        }

        let i = self.width as usize * point.y as usize + point.x as usize;
        &mut self.cells[i]
    }

    // Find local maxiumum in the circle grid
    pub fn find_maximum(&self, mut point: Point2d) -> Point2d {
        loop {
            let Point2d { x, y} = point;
            let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1), (x + 1, y + 1), (x + 1, y - 1), (x - 1, y - 1), (x, y + 1)];

            // Get the largest neighbor
            let max = neighbors.into_iter().filter_map(|(x, y)| {
                if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
                    return None;
                }

                Some((x, y, self.get((x, y)).unwrap_or(f64::NEG_INFINITY)))
            }).reduce(|a, b| if a.2 > b.2 {
                a
            } else {
                b
            }).unwrap();

            // Return point if it is the maximum, otherwise set point to the
            // largest neighbor and continue searching.
            if self.get((x, y)).unwrap_or(f64::NEG_INFINITY) >= max.2 {
                return point;
            }

            point = Point2d { x: max.0, y: max.1 };
        }
    }

    pub fn max_radius(&self, point: Point2d, padding: f64) -> f64 {
        self.get(point).map_or(f64::MAX, |distance| distance - padding)
    }
}

pub struct CircleGenerator<'a> {
    rng: &'a mut dyn RngCore,
    width: u32,
    height: u32,
    min_radius: f64,
    max_radius: f64,
    padding: f64,
    coverage: f64,
    edge_bias: f64,
    size_variation: f64
}

impl<'a> CircleGenerator<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> CircleGenerator<'a> {
        CircleGenerator {
            rng,
            width: 1000,
            height: 1000,
            min_radius: 3.0,
            max_radius: 6.9,
            padding: 0.5,
            coverage: 0.6,
            edge_bias: 8.0,
            size_variation: 0.5
        }
    }

    #[allow(unused)]
    fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;

        self
    }

    #[allow(unused)]
    fn min_radius(mut self, min_radius: f64) -> Self {
        assert!(min_radius > 0.0, "min_radius must be > 0.");
        self.min_radius = min_radius;

        self
    }

    #[allow(unused)]
    fn max_radius(mut self, max_radius: f64) -> Self {
        assert!(max_radius > 0.0, "max_radius must be > 0.");
        self.max_radius = max_radius;

        self
    }

    #[allow(unused)]
    fn padding(mut self, padding: f64) -> Self {
        assert!(padding > 0.0, "padding must be > 0.");
        self.padding = padding;

        self
    }

    #[allow(unused)]
    fn coverage(mut self, coverage: f64) -> Self {
        assert!((0.0..=1.0).contains(&coverage), "coverage must be between 0 and 1.");
        self.coverage = coverage;

        self
    }

    #[allow(unused)]
    fn edge_bias(mut self, edge_bias: f64) -> Self {
        assert!(edge_bias >= 1.0, "edge_bias must be >= 1.");
        self.edge_bias = edge_bias;

        self
    }

    #[allow(unused)]
    fn size_variation(mut self, size_variation: f64) -> Self {
        assert!((0.0..=1.0).contains(&size_variation), "size_variation must be between 0 and 1.");
        self.size_variation = size_variation;

        self
    }

    fn generate(self) -> Vec<Circle> {
        const MAX_MISSES: usize = 1000;

        assert!(self.max_radius >= self.min_radius, "max_radius must be >= than min_radius");

        let total_area = (self.width * self.height) as f64;

        let mut grid = CircleGrid::new(self.width, self.height, self.edge_bias);
        let uniform = Uniform::new(
            Point2d { x: 0, y: 0 },
            Point2d {
                x: self.width as i32,
                y: self.height as i32,
            },
        )
        .unwrap();

        let mut circles = Vec::new();
        let mut area = 0.0;
        let mut missed = 0;
        // I doubt it's possible to determine if our given parameters can actually achieve the
        // desired coverage, so just exit if there are too many misses in a row.
        while area / total_area <= self.coverage && missed < MAX_MISSES {
            let center = uniform.sample(self.rng);
            // Move random point to local maximum to increase the odds of being able to place a circle
            let center = grid.find_maximum(center);
            let max_radius = f64::min(grid.max_radius(center, self.padding), self.max_radius);
            let min_radius = f64::max(max_radius * self.size_variation, self.min_radius);

            // Count as miss if the max radius is smaller than the minimum allowed radius.
            if max_radius < min_radius {
                missed += 1;
                continue;
            }

            // Somehow i've had crashes because these are exaclty equal
            let radius = match max_radius == min_radius {
                true => max_radius,
                false => self.rng.random_range(min_radius..max_radius)
            };

            let circle = Circle { center, radius, ishihara_color: None };

            // Update the circle grid, place the circle, and update the area
            grid.fill(circle.center, circle, self.max_radius + self.padding * 2.0);
            circles.push(circle);
            area += std::f64::consts::PI * radius * radius;

            // Reset missed count
            missed = 0;
        }

        // TODO: Probably remove this
        #[cfg(test)]
        {
            println!("target coverage: {}", self.coverage);
            println!("actual coverage: {}", area / total_area);
        }

        circles
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::create_dir_all, path::Path};

    use image::{ImageBuffer, Rgba, RgbaImage, imageops::{FilterType::Lanczos3, resize}};
    use imageproc::{drawing::{draw_filled_circle_mut, draw_filled_rect_mut}, rect::Rect};
    use rand::{SeedableRng, rngs::StdRng};
    use crate::ishihara::{Circle, CircleGenerator};

    static TEST_DIR: &str = "../test artifacts/";

    const WIDTH: u32 = 1042;
    const HEIGHT: u32 = 296;

    fn generate_circles_new() -> Vec<Circle> {
        let mut rng = StdRng::seed_from_u64(0);
        CircleGenerator::new(&mut rng)
            .size(WIDTH, HEIGHT)
            .generate()
    }

    fn draw(circles: &[Circle]) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut canvas = RgbaImage::new(WIDTH * 2, HEIGHT * 2);
        let (width, height) = canvas.dimensions();
        draw_filled_rect_mut(&mut canvas, Rect::at(0, 0).of_size(width, height), Rgba([255, 255, 255, 255]));

        for circle in circles {
            let center = (circle.center.x * 2, circle.center.y * 2);
            draw_filled_circle_mut(&mut canvas, center, circle.radius as i32 * 2, Rgba([0, 0, 0, 255]));
        }

        let canvas = resize(&canvas, WIDTH, HEIGHT, Lanczos3);

        canvas
    }

    #[test]
    pub fn circle_generator() -> Result<(), Box<dyn std::error::Error>> {
        println!("test dir: {:?}", std::path::absolute(Path::new(TEST_DIR))?.as_os_str());
        create_dir_all(TEST_DIR)?;

        let output_file = Path::new(TEST_DIR).join("circle_generator_output.png");

        let circles = generate_circles_new();

        let image = draw(&circles);

        image.save(output_file)?;

        Ok(())
    }
}