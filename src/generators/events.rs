//! A fixed-size voice pool: no callback-time allocation, even at high event densities.
use super::{Brown, Generator, NoiseKind, SoundConfig};
use rand::{prelude::*, rngs::StdRng};
#[derive(Clone, Copy, Default)]
struct Voice {
    age: u32,
    length: u32,
    phase: f64,
    step: f64,
    amplitude: f32,
}
pub struct Events {
    kind: NoiseKind,
    gain: f32,
    sr: f32,
    probability: f32,
    level: f32,
    rng: StdRng,
    bed: Brown,
    low: f32,
    voices: [Voice; 32],
    next: usize,
}
impl Events {
    pub fn new(kind: NoiseKind, gain: f32, sr: f32, config: &SoundConfig, mut rng: StdRng) -> Self {
        let default_density = if matches!(kind, NoiseKind::Rain) {
            100.0
        } else {
            8.0
        };
        let bed = Brown::with_rng(1.0, sr, StdRng::seed_from_u64(rng.gen()));
        Self {
            kind,
            gain,
            sr,
            probability: config.density.unwrap_or(default_density) / sr,
            level: config.event_level,
            rng,
            bed,
            low: 0.0,
            voices: [Voice::default(); 32],
            next: 0,
        }
    }
}
impl Generator for Events {
    fn next_frame(&mut self, out: &mut [f32]) {
        if self.rng.gen::<f32>() < self.probability {
            // Prefer an inactive voice; steal round-robin only when the bounded pool is full.
            let slot = self
                .voices
                .iter()
                .position(|v| v.age >= v.length)
                .unwrap_or(self.next);
            let rain = matches!(self.kind, NoiseKind::Rain);
            let hz: f32 = if rain {
                self.rng.gen_range(500.0..3000.0)
            } else {
                self.rng.gen_range(100.0..1200.0)
            };
            self.voices[slot] = Voice {
                age: 0,
                length: (self.rng.gen_range(0.008..0.06) * self.sr).max(2.0) as u32,
                phase: 0.0,
                step: std::f64::consts::TAU * hz.min(self.sr * 0.4) as f64 / self.sr as f64,
                amplitude: self.rng.gen_range(0.2..1.0),
            };
            self.next = (slot + 1) % self.voices.len();
        }
        let white = self.rng.gen_range(-1.0..1.0);
        let cutoff: f32 = if matches!(self.kind, NoiseKind::Rain) {
            6000.0
        } else {
            900.0
        };
        let alpha = 1.0 - (-std::f32::consts::TAU * cutoff.min(self.sr * 0.4) / self.sr).exp();
        self.low += alpha * (white - self.low);
        let mut events = 0.0;
        for voice in &mut self.voices {
            if voice.age < voice.length {
                let t = voice.age as f32 / (voice.length - 1) as f32;
                let envelope = (std::f32::consts::PI * t).sin() * (-5.0 * t).exp();
                let carrier = if matches!(self.kind, NoiseKind::Rain) {
                    voice.phase.sin() as f32
                } else {
                    white
                };
                events += carrier * voice.amplitude * envelope;
                voice.phase = (voice.phase + voice.step) % std::f64::consts::TAU;
                voice.age += 1;
            }
        }
        let bed = if matches!(self.kind, NoiseKind::Rain) {
            self.low
        } else {
            let mut s = [0.0];
            self.bed.next_frame(&mut s);
            s[0] * 0.6 + self.low * 0.4
        };
        // Smoothly bound dense overlapping events; fixed dry/wet balance.
        out.fill(((1.0 - self.level) * bed + self.level * events.tanh()) * self.gain);
    }
}
