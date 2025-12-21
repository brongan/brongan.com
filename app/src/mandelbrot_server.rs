use crate::point2d::Point2D;
use leptos::prelude::*;
use num::Complex;
use image::{GrayImage};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use anyhow::anyhow;

type Point2d = Point2D<u32>;

#[server]
pub async fn mandelbrot_get(
    bounds: Bounds,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Result<(), ServerFnError> {
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
