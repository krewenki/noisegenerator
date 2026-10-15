//! Binaural spatial audio generator.

use crate::generators::Generator;

/// Binaural generator: wraps two independent generators and applies smooth stereo panning.
/// Creates an immersive spatial soundscape by slowly moving one source across the stereo field.
pub struct Binaural {
    gen_a: Box<dyn Generator>, // First generator (stationary or opposite pan)
    gen_b: Box<dyn Generator>, // Second generator (panning)
    lfo_phase: f32,            // LFO phase for panning (0..2π)
    depth: f32,
    lfo_speed: f32,   // How fast the LFO cycles (radians per sample)
    temp_a: Vec<f32>, // Reusable buffer for gen_a
    temp_b: Vec<f32>, // Reusable buffer for gen_b
}

impl Binaural {
    pub fn new(gen_a: Box<dyn Generator>, gen_b: Box<dyn Generator>, sample_rate: f32) -> Self {
        Self::with_motion(gen_a, gen_b, sample_rate, 0.1, 1.0)
    }

    pub fn with_motion(
        gen_a: Box<dyn Generator>,
        gen_b: Box<dyn Generator>,
        sample_rate: f32,
        lfo_freq: f32,
        depth: f32,
    ) -> Self {
        let lfo_speed = 2.0 * std::f32::consts::PI * lfo_freq / sample_rate;

        Self {
            gen_a,
            gen_b,
            lfo_phase: 0.0,
            lfo_speed,
            depth,
            temp_a: vec![0.0f32; 1],
            temp_b: vec![0.0f32; 1],
        }
    }
}

impl Generator for Binaural {
    fn next_frame(&mut self, out: &mut [f32]) {
        // Generate mono samples from both generators (reuse buffers)
        self.gen_a.next_frame(&mut self.temp_a);
        self.gen_b.next_frame(&mut self.temp_b);

        let sample_a = self.temp_a[0];
        let sample_b = self.temp_b[0];

        // Calculate pan position using sine wave LFO (-1 to 1)
        let pan = self.lfo_phase.sin() * self.depth;

        // Equal power panning law for smooth transitions
        // pan = -1 (full left), 0 (center), 1 (full right)
        let angle = (pan + 1.0) * 0.25 * std::f32::consts::PI; // 0 to π/2
        let left_gain = angle.cos();
        let right_gain = angle.sin();

        // Mix: gen_a stays centered, gen_b pans
        if out.len() >= 2 {
            // Stereo output
            out[0] = sample_a * 0.707 + sample_b * left_gain; // Left channel
            out[1] = sample_a * 0.707 + sample_b * right_gain; // Right channel

            // Copy to additional channels if present
            for i in 2..out.len() {
                out[i] = out[i % 2];
            }
        } else if out.len() == 1 {
            // Mono output (fallback)
            out[0] = (sample_a + sample_b) * 0.5;
        }

        // Advance LFO phase with modulo to prevent precision drift
        self.lfo_phase += self.lfo_speed;
        // Use modulo instead of subtraction to maintain precision over long durations
        self.lfo_phase = self.lfo_phase % (2.0 * std::f32::consts::PI);
    }
}
