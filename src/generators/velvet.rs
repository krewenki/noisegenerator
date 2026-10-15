//! Velvet noise generator.

use crate::generators::Generator;
use rand::prelude::*;

/// Velvet noise: sparse random impulses. Sounds like crackling or light rain.
pub struct Velvet {
    gain: f32,
    rng: rand::rngs::StdRng,
    density: f32, // probability of impulse per sample
}

impl Velvet {
    pub fn new(gain: f32, _sample_rate: f32) -> Self {
        Self::with_rng(gain, _sample_rate, StdRng::from_entropy())
    }

    pub(crate) fn with_rng(gain: f32, _sample_rate: f32, rng: StdRng) -> Self {
        // Target ~1000 impulses per second
        let density = 1000.0 / _sample_rate;
        Self { gain, rng, density }
    }
    pub(crate) fn set_density(&mut self, density: f32, sr: f32) {
        self.density = density / sr;
    }
}

impl Generator for Velvet {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Randomly place impulses
        let s = if self.rng.gen::<f32>() < self.density {
            // Random polarity impulse
            if self.rng.gen::<bool>() {
                self.gain
            } else {
                -self.gain
            }
        } else {
            0.0
        };
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
