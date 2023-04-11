#![allow(dead_code)]
use clap::Parser;
use num::Complex;
use std::fmt::{self, Display, Formatter};
use rayon::prelude::*;
use mandelbrot::{ImageBuffer, Bounds, Point2d, pixel_to_point, escape_time};

fn render_multithreaded(
    image: &mut ImageBuffer,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    let bounds = image.bounds();
    let mut pixels: Vec<&mut [u8]> = image.pixels.chunks_mut(1).collect();
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let point = Point2d {
            x: i % image.width,
            y: i / image.width,
        };
        let point = pixel_to_point(bounds, point, upper_left, lower_right);
        pixel[0] = match escape_time(point, 255) {
            None => 0,
            Some(count) => 255 - count as u8,
        };
    })
}

#[derive(Parser, Default, Debug)]
#[clap(
    author = "Brennan",
    version,
    about = "multithreaded mandelbrot printer"
)]
struct Args {
    bounds: Bounds,
    #[arg(allow_hyphen_values = true)]
    upper_left: Complex<f64>,
    #[arg(allow_hyphen_values = true)]
    lower_right: Complex<f64>,
    #[arg(value_name = "FILE", default_value = "/dev/stdout")]
    path: String,
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.bounds, self.upper_left, self.lower_right, self.path
        )
    }
}

fn main() {
    let args = Args::parse();
    let mut image = ImageBuffer::new(args.bounds.width, args.bounds.height);
    render_multithreaded(&mut image, args.upper_left, args.lower_right);
    image
        .write_image(&args.path)
        .expect("error writing PNG file");
}
