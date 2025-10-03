//! White noise generator.

use rand::prelude::*;
use crate::generators::Generator;

/// White noise generator using uniform random distribution.
pub struct White {
    gain: f32,
    rng: rand::rngs::StdRng,
}

impl White {
    pub fn new(gain: f32) -> Self {
        Self {
            gain,
            rng: StdRng::from_entropy(),
        }
    }
}

impl Generator for White {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Uniform white noise in [-1,1], scaled by gain
        let s: f32 = (self.rng.gen::<f32>() * 2.0 - 1.0) * self.gain;
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
