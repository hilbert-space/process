//! Fractional Brownian motion and fractional Gaussian noise.

use complex::Complex;
use probability::distribution::{Distribution, Gaussian};
use probability::generator::Generator;

use {Process, Stationary};
use gaussian::circulant_embedding;

macro_rules! hurst(
    ($value:expr) => ({
        let value = $value;
        debug_assert!(value > 0.0 && value < 1.0);
        value
    });
);

macro_rules! step(
    ($value:expr) => ({
        let value = $value;
        debug_assert!(value > 0.0);
        value
    });
);

/// A fractional Brownian motion.
pub struct Motion {
    hurst: f64,
}

/// A fractional Gaussian noise.
pub struct Noise {
    hurst: f64,
    step: f64,
}

impl Motion {
    /// Create a fractional Brownian motion.
    #[inline]
    pub fn new(hurst: f64) -> Motion {
        Motion { hurst: hurst!(hurst) }
    }

    /// Generate a sample path.
    pub fn sample<G>(&self, points: usize, step: f64, generator: &mut G) -> Vec<f64>
        where G: Generator
    {
        match points {
            0 => vec![],
            1 => vec![0.0],
            _ => {
                let mut data = vec![0.0];
                data.extend(Noise::new(self.hurst, step).sample(points - 1, generator));
                for i in 2..points {
                    data[i] += data[i - 1];
                }
                data
            },
        }
    }
}

impl Noise {
    /// Create a fractional Gaussian noise.
    #[inline]
    pub fn new(hurst: f64, step: f64) -> Noise {
        Noise { hurst: hurst!(hurst), step: step!(step) }
    }

    /// Generate a sample path.
    pub fn sample<G>(&self, points: usize, generator: &mut G) -> Vec<f64>
        where G: Generator
    {
        match points {
            0 => vec![],
            1 => vec![Gaussian::new(0.0, 1.0).sample(generator)],
            _ => {
                let n = points - 1;
                let gaussian = Gaussian::new(0.0, 1.0);
                let scale = (1.0 / n as f64).powf(self.hurst);
                let data = circulant_embedding(self, n, || gaussian.sample(generator));
                data.iter().take(points).map(|point| scale * point.re()).collect()
            },
        }
    }
}

impl Process for Motion {
    type Index = f64;
    type State = f64;

    fn cov(&self, t: f64, s: f64) -> f64 {
        debug_assert!(t >= 0.0 && s >= 0.0);
        let power = 2.0 * self.hurst;
        0.5 * (t.powf(power) + s.powf(power) - (t - s).abs().powf(power))
    }
}

impl Process for Noise {
    type Index = usize;
    type State = f64;

    #[inline]
    fn cov(&self, t: usize, s: usize) -> f64 {
        Stationary::cov(self, if t < s { s - t } else { t - s })
    }
}

impl Stationary for Noise {
    type Index = usize;

    fn cov(&self, tau: usize) -> f64 {
        let tau = tau as f64;
        let power = 2.0 * self.hurst;
        0.5 * self.step.powf(power) * ((tau + 1.0).powf(power) - 2.0 * tau.powf(power) +
                                       (tau - 1.0).abs().powf(power))
    }
}