//! Small composable DSP effects. Filter coefficients follow the Audio EQ Cookbook.
use super::{spatial::write_stereo, Generator};

#[derive(Clone, Debug, Default)]
pub struct EffectsConfig {
    pub lowpass: Option<f32>,
    pub highpass: Option<f32>,
    pub breath_rate: f32,
    pub breath_depth: f32,
    pub fade_in: f64,
    pub fade_out: f64,
}

#[derive(Clone, Copy, Default)]
pub struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    x: [f32; 2],
    y: [f32; 2],
}
impl Biquad {
    pub fn lowpass(hz: f32, sample_rate: f32) -> Self {
        Self::new(hz, sample_rate, false)
    }
    pub fn highpass(hz: f32, sample_rate: f32) -> Self {
        Self::new(hz, sample_rate, true)
    }
    fn new(hz: f32, sr: f32, high: bool) -> Self {
        let omega = std::f32::consts::TAU * hz / sr;
        let cos = omega.cos();
        let alpha = omega.sin() / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
        let a0 = 1.0 + alpha;
        let b = if high {
            [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0]
        } else {
            [(1.0 - cos) / 2.0, 1.0 - cos, (1.0 - cos) / 2.0]
        };
        Self {
            b: b.map(|v| v / a0),
            a: [-2.0 * cos / a0, (1.0 - alpha) / a0],
            ..Default::default()
        }
    }
    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.b[0] * input + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [input, self.x[0]];
        self.y = [out, self.y[0]];
        out
    }
}

pub struct Effects {
    source: Box<dyn Generator>,
    low: Option<[Biquad; 2]>,
    high: Option<[Biquad; 2]>,
    depth: f32,
    phase: f64,
    step: f64,
}
impl Effects {
    pub fn new(source: Box<dyn Generator>, config: &EffectsConfig, sr: f32) -> Self {
        Self {
            source,
            low: config.lowpass.map(|f| [Biquad::lowpass(f, sr); 2]),
            high: config.highpass.map(|f| [Biquad::highpass(f, sr); 2]),
            depth: config.breath_depth,
            phase: 0.0,
            step: std::f64::consts::TAU * config.breath_rate as f64 / sr as f64,
        }
    }
}
impl Generator for Effects {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut stereo = [0.0; 2];
        self.source.next_frame(&mut stereo);
        let envelope = 1.0 - self.depth * (0.5 - 0.5 * self.phase.cos() as f32);
        for (channel, sample) in stereo.iter_mut().enumerate() {
            if let Some(filters) = &mut self.high {
                *sample = filters[channel].process(*sample);
            }
            if let Some(filters) = &mut self.low {
                *sample = filters[channel].process(*sample);
            }
            *sample *= envelope;
        }
        write_stereo(out, stereo[0], stereo[1]);
        self.phase = (self.phase + self.step) % std::f64::consts::TAU;
    }
}

pub struct Fade {
    source: Box<dyn Generator>,
    index: u64,
    fade_in: f64,
    fade_out: f64,
    total: Option<f64>,
}
impl Fade {
    pub fn new(
        source: Box<dyn Generator>,
        config: &EffectsConfig,
        sr: f32,
        seconds: Option<u64>,
    ) -> Self {
        Self {
            source,
            index: 0,
            fade_in: config.fade_in * sr as f64,
            fade_out: config.fade_out * sr as f64,
            total: seconds.map(|s| s as f64 * sr as f64),
        }
    }
}
impl Generator for Fade {
    fn next_frame(&mut self, out: &mut [f32]) {
        self.source.next_frame(out);
        let smooth = |x: f64| {
            (x.clamp(0.0, 1.0) * std::f64::consts::FRAC_PI_2)
                .sin()
                .powi(2) as f32
        };
        let mut gain = if self.fade_in > 0.0 {
            smooth(self.index as f64 / self.fade_in)
        } else {
            1.0
        };
        if let Some(total) = self.total {
            if self.fade_out > 0.0 {
                gain *= smooth((total - 1.0 - self.index as f64) / self.fade_out);
            }
        }
        for sample in out {
            *sample *= gain;
        }
        self.index = self.index.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Left;
    impl Generator for Left {
        fn next_frame(&mut self, out: &mut [f32]) {
            write_stereo(out, 1.0, 0.0);
        }
    }
    #[test]
    fn fade_endpoints_and_stereo_state() {
        let config = EffectsConfig {
            fade_in: 0.1,
            fade_out: 0.1,
            lowpass: Some(100.0),
            ..Default::default()
        };
        let mut fade = Fade::new(Box::new(Left), &config, 1000.0, Some(1));
        for i in 0..1000 {
            let mut frame = [0.0; 2];
            fade.next_frame(&mut frame);
            assert_eq!(frame[1], 0.0);
            if i == 0 || i == 999 {
                assert_eq!(frame[0], 0.0);
            }
            if i == 500 {
                assert_eq!(frame[0], 1.0);
            }
        }
        let mut effect = Effects::new(Box::new(Left), &config, 1000.0);
        for _ in 0..10000 {
            let mut frame = [0.0; 2];
            effect.next_frame(&mut frame);
            assert_eq!(frame[1], 0.0);
            assert!(frame[0].is_finite());
        }
    }
    #[test]
    fn breathing_depth_is_bounded_and_reaches_trough() {
        let config = EffectsConfig {
            breath_rate: 1.0,
            breath_depth: 0.8,
            ..Default::default()
        };
        let mut effect = Effects::new(Box::new(Left), &config, 1000.0);
        let mut min = 1.0f32;
        for _ in 0..1000 {
            let mut frame = [0.0; 2];
            effect.next_frame(&mut frame);
            min = min.min(frame[0]);
            assert!((0.1999..=1.0).contains(&frame[0]));
        }
        assert!((min - 0.2).abs() < 1e-5);
    }
}
