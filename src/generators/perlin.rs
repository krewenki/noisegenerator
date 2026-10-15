//! Perlin noise generator.

use crate::generators::Generator;
use rand::prelude::*;

/// Perlin noise: smooth, organic-sounding noise with natural-feeling transitions.
/// Uses 1D Perlin noise with controllable frequency for evolving textures.
pub struct Perlin {
    gain: f32,
    t: f32,               // time parameter
    speed: f32,           // how fast we move through noise space
    octaves: usize,       // number of octaves for fractal noise
    permutation: Vec<u8>, // permutation table for Perlin
}

impl Perlin {
    pub fn new(gain: f32, sample_rate: f32) -> Self {
        Self::with_rng(gain, sample_rate, StdRng::from_entropy())
    }

    pub(crate) fn with_rng(gain: f32, sample_rate: f32, mut rng: StdRng) -> Self {
        // Create permutation table
        let mut permutation: Vec<u8> = (0..=255).collect();
        permutation.shuffle(&mut rng);
        // Duplicate for easy wrapping
        permutation.extend_from_slice(&permutation.clone());

        // Speed: move faster through noise space to get audible content
        // Target about 50 Hz worth of traversal for base frequency
        let speed = 50.0 / sample_rate;

        Self {
            gain,
            t: rng.gen::<f32>() * 256.0, // Random starting position
            speed,
            octaves: 6,
            permutation,
        }
    }

    pub(crate) fn set_texture(&mut self, speed: f32, octaves: usize, sr: f32) {
        self.speed = speed / sr;
        self.octaves = octaves;
    }

    // Fade function for smooth interpolation (6t^5 - 15t^4 + 10t^3)
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    // Linear interpolation
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    // Gradient function for 1D Perlin noise
    fn grad(&self, hash: u8, x: f32) -> f32 {
        // Simpler gradient: just use hash to pick +1 or -1
        if hash & 1 == 0 {
            x
        } else {
            -x
        }
    }

    // 1D Perlin noise
    fn noise_1d(&self, x: f32) -> f32 {
        let xi = (x.floor() as i32) & 255;
        let xf = x - x.floor();

        let u = Self::fade(xf);

        let a = self.permutation[xi as usize];
        let b = self.permutation[(xi + 1) as usize];

        Self::lerp(self.grad(a, xf), self.grad(b, xf - 1.0), u)
    }

    // Fractal (multi-octave) Perlin noise
    fn fractal_noise(&self, x: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += self.noise_1d(x * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= 0.65; // Slightly less falloff for more energy
            frequency *= 1.8; // Not quite doubling for more interesting texture
        }

        total / max_value // Normalize to roughly -1..1
    }
}

impl Generator for Perlin {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Generate fractal Perlin noise
        let s = self.fractal_noise(self.t) * self.gain;
        self.t += self.speed;

        // Wrap t with modulo to maintain precision over very long durations
        self.t = self.t % 256.0;

        for x in out.iter_mut() {
            *x = s;
        }
    }
}
