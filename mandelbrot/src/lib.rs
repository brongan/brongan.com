use anyhow::anyhow;
use image::{RgbaImage, DynamicImage};
use num::Complex;
use std::fmt::{self, Display};
use std::str::FromStr;
#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Copy)]
pub struct Point2d {
    pub x: u32,
    pub y: u32,
}

impl Display for Point2d {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

struct PixelIterator<I> {
    iter: I,
    count: u32,
    width: u32,
}

impl<I> PixelIterator<I> {
    fn new(iter: I, width: u32) -> PixelIterator<I> {
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
        let point = pixel_to_point(Bounds { width: bounds.0, height: bounds.1}, Point2d{x, y}, upper_left, lower_right);
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