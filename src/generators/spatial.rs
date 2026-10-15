//! Stereo sound placement. These modes use panning, not binaural-beat tones or HRTFs.
use super::Generator;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum SpatialMode {
    #[default]
    Mono,
    Wide,
    Binaural,
    Trinaural,
    Drift,
}

#[derive(Clone, Debug)]
pub struct SpatialConfig {
    pub mode: SpatialMode,
    /// Independent-channel mix: 0 = shared mono, 1 = independent left/right.
    pub width: f32,
    /// Cycles per second; zero freezes the placement.
    pub motion_rate: f32,
    /// Pan excursion, from centered (0) to full left/right (1).
    pub motion_depth: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            mode: SpatialMode::Mono,
            width: 1.0,
            motion_rate: 0.1,
            motion_depth: 1.0,
        }
    }
}

pub struct Spatial {
    sources: [Box<dyn Generator>; 3],
    config: SpatialConfig,
    phase: f64,
    step: f64,
}

impl Spatial {
    pub fn new(sources: [Box<dyn Generator>; 3], config: SpatialConfig, sample_rate: f32) -> Self {
        let step = std::f64::consts::TAU * config.motion_rate as f64 / sample_rate as f64;
        Self {
            sources,
            config,
            phase: 0.0,
            step,
        }
    }
}

pub(crate) fn write_stereo(out: &mut [f32], left: f32, right: f32) {
    if out.len() == 1 {
        out[0] = (left + right) * 0.5;
    } else {
        for (i, sample) in out.iter_mut().enumerate() {
            *sample = if i % 2 == 0 { left } else { right };
        }
    }
}

impl Generator for Spatial {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut samples = [0.0; 3];
        for (source, sample) in self.sources.iter_mut().zip(&mut samples) {
            source.next_frame(std::slice::from_mut(sample));
        }
        let [a, b, c] = samples;
        let (left, right) = if self.config.mode == SpatialMode::Wide {
            let width = self.config.width;
            ((1.0 - width) * a + width * b, (1.0 - width) * a + width * c)
        } else {
            let pan_a = self.phase.sin() as f32 * self.config.motion_depth;
            // Different long periods and multiple smooth components avoid a simple pendulum.
            let pan_b = if self.config.mode == SpatialMode::Drift {
                ((self.phase * 0.73 + 1.4).sin() * 0.6 + (self.phase * 0.31).sin() * 0.4) as f32
            } else {
                (self.phase * 0.73 + std::f64::consts::PI).sin() as f32
            } * self.config.motion_depth;
            let angle_a = (pan_a + 1.0) * std::f32::consts::FRAC_PI_4;
            let angle_b = (pan_b + 1.0) * std::f32::consts::FRAC_PI_4;
            // Conservative, fixed peak headroom (not a modulating auto-leveler).
            let scale = 1.0 / (std::f32::consts::FRAC_1_SQRT_2 + 2.0);
            (
                (a * std::f32::consts::FRAC_1_SQRT_2 + b * angle_a.cos() + c * angle_b.cos())
                    * scale,
                (a * std::f32::consts::FRAC_1_SQRT_2 + b * angle_a.sin() + c * angle_b.sin())
                    * scale,
            )
        };
        write_stereo(out, left, right);
        // Keep the slower oscillators continuous too; avoid wrapping an incommensurate phase.
        self.phase += self.step;
    }
}
