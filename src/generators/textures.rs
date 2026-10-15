//! Procedural noise textures, not field recordings.
use super::{Generator, NoiseKind};

pub struct Texture {
    source: Box<dyn Generator>,
    kind: NoiseKind,
    gain: f32,
    phase: f64,
    slow_phase: f64,
    step: f64,
    slow_step: f64,
    sr: f32,
    filtered: f32,
}
impl Texture {
    pub fn new(kind: NoiseKind, gain: f32, sample_rate: f32, source: Box<dyn Generator>) -> Self {
        let rate = if matches!(kind, NoiseKind::Ocean) {
            0.09
        } else {
            0.045
        };
        Self {
            source,
            kind,
            gain,
            phase: 0.0,
            slow_phase: 1.2,
            step: std::f64::consts::TAU * rate / sample_rate as f64,
            slow_step: std::f64::consts::TAU * 0.017 / sample_rate as f64,
            sr: sample_rate,
            filtered: 0.0,
        }
    }
}
impl Generator for Texture {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut sample = [0.0];
        self.source.next_frame(&mut sample);
        let swell = (0.5 - 0.5 * self.phase.cos()) as f32;
        let slow = (0.5 + 0.5 * self.slow_phase.sin()) as f32;
        let (envelope, cutoff) = if matches!(self.kind, NoiseKind::Ocean) {
            let crest = swell * swell;
            (0.15 + 0.85 * crest, 350.0 + 2800.0 * crest)
        } else {
            (
                0.2 + 0.8 * swell * (0.5 + 0.5 * slow),
                180.0 + 1800.0 * swell * slow,
            )
        };
        let coefficient =
            1.0 - (-std::f32::consts::TAU * cutoff.min(self.sr * 0.4) / self.sr).exp();
        self.filtered += coefficient * (sample[0] - self.filtered);
        out.fill(self.filtered * envelope * self.gain);
        self.phase = (self.phase + self.step * (0.8 + 0.4 * slow as f64)) % std::f64::consts::TAU;
        self.slow_phase = (self.slow_phase + self.slow_step) % std::f64::consts::TAU;
    }
}
