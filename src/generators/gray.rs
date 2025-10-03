//! Gray noise generator.

use rand::prelude::*;
use crate::generators::Generator;

/// Gray noise: psychoacoustically weighted to sound equally loud across frequencies.
/// Uses approximate equal-loudness weighting based on A-weighting.
pub struct Gray {
    gain: f32,
    rng: rand::rngs::StdRng,
    // Simple filters for approximating equal-loudness curve
    hp_y1: f32,
    hp_x1: f32,
    lp_y1: f32,
}

impl Gray {
    pub fn new(gain: f32, _sample_rate: f32) -> Self {
        Self {
            gain,
            rng: StdRng::from_entropy(),
            hp_y1: 0.0,
            hp_x1: 0.0,
            lp_y1: 0.0,
        }
    }
}

impl Generator for Gray {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Start with white noise
        let w: f32 = self.rng.gen::<f32>() * 2.0 - 1.0;
        
        // Apply high-pass filter (boost mids, cut bass)
        let hp = w - self.hp_x1 + 0.95 * self.hp_y1;
        self.hp_x1 = w;
        self.hp_y1 = hp;
        
        // Apply gentle low-pass to tame extreme highs
        let lp = 0.85 * hp + 0.15 * self.lp_y1;
        self.lp_y1 = lp;
        
        let s = lp * self.gain * 0.7; // Scaled for comfortable listening
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
