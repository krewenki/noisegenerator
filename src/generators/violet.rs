//! Violet noise generator.

use rand::prelude::*;
use crate::generators::Generator;

/// Violet noise: double differentiation (even more high-frequency emphasis than blue).
pub struct Violet {
    gain: f32,
    rng: rand::rngs::StdRng,
    prev: f32,
    prev2: f32,
}

impl Violet {
    pub fn new(gain: f32, _sample_rate: f32) -> Self {
        Self {
            gain,
            rng: StdRng::from_entropy(),
            prev: 0.0,
            prev2: 0.0,
        }
    }
}

impl Generator for Violet {
    fn next_frame(&mut self, out: &mut [f32]) {
        let w: f32 = self.rng.gen::<f32>() * 2.0 - 1.0;
        let diff = w - 2.0 * self.prev + self.prev2; // second derivative
        let s = diff * self.gain * 0.25; // scaled down more
        self.prev2 = self.prev;
        self.prev = w;
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
