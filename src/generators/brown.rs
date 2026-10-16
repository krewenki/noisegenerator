//! Brown (Brownian) noise generator.

use crate::generators::Generator;
use rand::prelude::*;

/// Brown(ian) noise: integrate white with a leaky integrator to avoid drift/clipping.
pub struct Brown {
    gain: f32,
    rng: rand::rngs::StdRng,
    y: f32,
    leak: f32, // 0..1; closer to 1 = less leak (darker)
}

impl Brown {
    pub fn new(gain: f32, _sample_rate: f32) -> Self {
        Self::with_rng(gain, _sample_rate, StdRng::from_entropy())
    }

    pub(crate) fn with_rng(gain: f32, _sample_rate: f32, rng: StdRng) -> Self {
        // Fixed leak constant works well for typical 44.1–48 kHz
        Self {
            gain,
            rng,
            y: 0.0,
            leak: 0.997,
        }
    }
}

impl Generator for Brown {
    fn next_frame(&mut self, out: &mut [f32]) {
        let w: f32 = self.rng.gen::<f32>() * 2.0 - 1.0;
        self.y = self.leak * self.y + 0.02 * w; // small step to keep variance reasonable
        let s = (self.y.tanh()) * self.gain; // soft limiting to avoid runaway peaks
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
