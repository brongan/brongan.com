use rand::distributions::uniform::{SampleBorrow, SampleUniform, Uniform, UniformSampler};
use rand::distributions::Distribution;
use rand::Rng;
use std::fmt;

#[derive(Debug, PartialEq, Default)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct UniformPoint2D(Uniform<i32>, Uniform<i32>);

impl UniformSampler for UniformPoint2D {
    type X = Point2D;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformPoint2D(
            Uniform::new(low.borrow().x, high.borrow().x),
            Uniform::new(low.borrow().y, high.borrow().y),
        )
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
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

impl SampleUniform for Point2D {
    type Sampler = UniformPoint2D;
}

impl Point2D {
    pub fn distance(&self, other: &Point2D) -> f64 {
        (((other.x - self.x).pow(2) + (other.y - self.y).pow(2)) as f64).sqrt()
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
