//! Fan and drone textures: noise plus quiet, slowly detuned sinusoidal components.
use super::{Generator, NoiseKind, Pink, SoundConfig};
use rand::{rngs::StdRng, Rng, SeedableRng};
pub struct Tonal {
    noise: Pink,
    kind: NoiseKind,
    gain: f32,
    phases: [f64; 3],
    drift: f64,
    sr: f32,
    hz: f32,
    level: f32,
    variation: f32,
    low: f32,
}
impl Tonal {
    pub fn new(kind: NoiseKind, gain: f32, sr: f32, config: &SoundConfig, mut rng: StdRng) -> Self {
        let phases = std::array::from_fn(|_| rng.gen::<f64>() * std::f64::consts::TAU);
        Self {
            noise: Pink::with_rng(1.0, StdRng::seed_from_u64(rng.gen())),
            kind,
            gain,
            phases,
            drift: 0.0,
            sr,
            hz: config.tone_hz,
            level: config.tone_level,
            variation: config.variation,
            low: 0.0,
        }
    }
}
impl Generator for Tonal {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut noise = [0.0];
        self.noise.next_frame(&mut noise);
        let cutoff: f32 = if matches!(self.kind, NoiseKind::Fan) {
            1200.0
        } else {
            600.0
        };
        let alpha = 1.0 - (-std::f32::consts::TAU * cutoff.min(self.sr * 0.4) / self.sr).exp();
        self.low += alpha * (noise[0] - self.low);
        let ratios = if matches!(self.kind, NoiseKind::Fan) {
            [1.0, 2.0, 3.0]
        } else {
            [1.0, 1.003, 1.5]
        };
        let mut tone = 0.0;
        for (i, phase) in self.phases.iter_mut().enumerate() {
            tone += phase.sin() as f32 / 3.0;
            let hz = self.hz as f64
                * ratios[i]
                * (1.0 + 0.002 * self.variation as f64 * (self.drift + i as f64).sin());
            *phase = (*phase + std::f64::consts::TAU * hz / self.sr as f64) % std::f64::consts::TAU;
        }
        let flutter = 1.0 - 0.08 * self.variation * (0.5 + 0.5 * self.drift.sin() as f32);
        out.fill(((1.0 - self.level) * self.low + self.level * tone) * flutter * self.gain);
        self.drift =
            (self.drift + std::f64::consts::TAU * 0.07 / self.sr as f64) % std::f64::consts::TAU;
    }
}
