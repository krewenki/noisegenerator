//! Velvet noise generator.

use rand::prelude::*;
use crate::generators::Generator;

/// Velvet noise: sparse random impulses. Sounds like crackling or light rain.
pub struct Velvet {
    gain: f32,
    rng: rand::rngs::StdRng,
    density: f32, // probability of impulse per sample
}

impl Velvet {
    pub fn new(gain: f32, sample_rate: f32) -> Self {
        // Target ~1000 impulses per second
        let density = 1000.0 / sample_rate;
        Self {
            gain,
            rng: StdRng::from_entropy(),
            density,
        }
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
