//! Blue noise generator.

use crate::generators::Generator;
use rand::prelude::*;

/// Blue noise: differentiate white noise (inverse of brown). Emphasizes high frequencies.
pub struct Blue {
    gain: f32,
    rng: rand::rngs::StdRng,
    prev: f32,
}

impl Blue {
    pub fn new(gain: f32, _sample_rate: f32) -> Self {
        Self::with_rng(gain, _sample_rate, StdRng::from_entropy())
    }

    pub(crate) fn with_rng(gain: f32, _sample_rate: f32, rng: StdRng) -> Self {
        Self {
            gain,
            rng,
            prev: 0.0,
        }
    }
}

impl Generator for Blue {
    fn next_frame(&mut self, out: &mut [f32]) {
        let w: f32 = self.rng.gen::<f32>() * 2.0 - 1.0;
        let s = (w - self.prev) * self.gain * 0.5; // difference, scaled down
        self.prev = w;
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
