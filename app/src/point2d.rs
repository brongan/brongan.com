use rand::distr::uniform::{SampleBorrow, SampleUniform, Uniform, UniformSampler};
use rand::distr::Distribution;
use rand::Rng;
use std::fmt;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Point2D<T> {
    pub x: T,
    pub y: T,
}

#[derive(Clone, Copy, Debug)]
pub struct UniformPoint2D(Uniform<i32>, Uniform<i32>);

impl UniformSampler for UniformPoint2D {
    type X = Point2D<i32>;
    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Ok(UniformPoint2D(
            Uniform::new(low.borrow().x, high.borrow().x)?,
            Uniform::new(low.borrow().y, high.borrow().y)?,
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Point2D {
            x: self.0.sample(rng),
            y: self.1.sample(rng),
        }
    }
}

impl SampleUniform for Point2D<i32> {
    type Sampler = UniformPoint2D;
}

impl Point2D<i32> {
    pub fn distance(&self, other: &Point2D<i32>) -> f64 {
        (((other.x - self.x).pow(2) + (other.y - self.y).pow(2)) as f64).sqrt()
    }
}

impl<T: std::fmt::Display> fmt::Display for Point2D<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<(T, T)> for Point2D<T> {
    fn from(value: (T, T)) -> Self {
        let (x, y) = value;
        Point2D { x, y }
    }
}