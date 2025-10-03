//! Pink noise generator.

use rand::prelude::*;
use crate::generators::Generator;

/// Pink noise via Voss-McCartney (6 rows). Cheap and decent.
pub struct Pink {
    gain: f32,
    rng: rand::rngs::StdRng,
    rows: [f32; 6],
    counter: u64,
}

impl Pink {
    pub fn new(gain: f32) -> Self {
        Self {
            gain,
            rng: StdRng::from_entropy(),
            rows: [0.0; 6],
            counter: 0,
        }
    }
}

impl Generator for Pink {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Update rows where the corresponding bit toggles
        let mut c = self.counter;
        let mut acc = 0.0f32;
        for (i, r) in self.rows.iter_mut().enumerate() {
            if c & 1 == 0 {
                *r = self.rng.gen::<f32>() * 2.0 - 1.0;
            }
            acc += *r;
            c >>= 1;
            if i == 5 {
                break;
            }
        }
        self.counter = self.counter.wrapping_add(1);
        let s = (acc / self.rows.len() as f32) * self.gain;
        for x in out.iter_mut() {
            *x = s;
        }
    }
}
