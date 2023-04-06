#![allow(dead_code)]
use anyhow::anyhow;
use anyhow::{Context, Result};
use clap::Parser;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num::Complex;
use rayon::prelude::*;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default)]
struct Bounds {
    width: usize,
    height: usize,
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

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    parse_pair(s, ',').map(|(re, im)| Complex { re, im })
}

impl FromStr for Bounds {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parse_pair(s, 'x')
            .map(|(width, height)| Bounds { width, height })
            .ok_or_else(|| anyhow!("Failed to parse bounds."))
    }
}

#[derive(Clone, Copy)]
struct Point2d {
    x: usize,
    y: usize,
}

impl Display for Point2d {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
struct ImageBuffer {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

struct PixelIterator<I> {
    iter: I,
    count: usize,
    width: usize,
}

impl<I> PixelIterator<I> {
    fn new(iter: I, width: usize) -> PixelIterator<I> {
        PixelIterator {
            iter,
            count: 0,
            width,
        }
    }
}

impl<I> Iterator for PixelIterator<I>
where
    I: Iterator,
{
    type Item = (Point2d, <I as Iterator>::Item);
    fn next(&mut self) -> Option<(Point2d, <I as Iterator>::Item)> {
        let a = self.iter.next()?;
        let i = self.count;
        self.count += 1;
        Some((
            Point2d {
                x: i % self.width,
                y: i / self.width,
            },
            a,
        ))
    }
}

impl ImageBuffer {
    fn new(width: usize, height: usize) -> ImageBuffer {
        ImageBuffer {
            pixels: vec![0; width * height],
            width,
            height,
        }
    }

    fn iter_pixels(&mut self) -> PixelIterator<std::slice::IterMut<'_, u8>> {
        PixelIterator::new(self.pixels.iter_mut(), self.width)
    }

    fn write_image(&self, filename: &str) -> Result<()> {
        let output = File::create(filename)?;
        let encoder = PngEncoder::new(output);
        encoder
            .write_image(
                &self.pixels,
                self.width as u32,
                self.height as u32,
                ColorType::L8,
            )
            .context("Failed to write image to {}, filename")?;
        Ok(())
    }

    fn bounds(&self) -> Bounds {
        Bounds {
            width: self.width,
            height: self.height,
        }
    }
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn pixel_to_point(
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

fn render(image: &mut ImageBuffer, upper_left: Complex<f64>, lower_right: Complex<f64>) {
    let bounds = image.bounds();
    for (point, pixel) in image.iter_pixels() {
        let point = pixel_to_point(bounds, point, upper_left, lower_right);
        *pixel = match escape_time(point, 255) {
            None => 0,
            Some(count) => 255 - count as u8,
        };
    }
}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
