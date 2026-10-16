//! Experimental lateral headphone cues, not measured HRTFs or front/back positioning.
use super::{spatial::write_stereo, Generator, SpatialConfig};

pub(crate) struct HeadphonePanner {
    history: Vec<f32>,
    cursor: usize,
    max_delay: f64,
    low: [f32; 2],
    alpha: f32,
    strength: f32,
}
impl HeadphonePanner {
    pub(crate) fn new(sr: f32, strength: f32) -> Self {
        // A bounded artistic approximation: up to 650 microseconds of interaural delay.
        let max_delay = sr as f64 * 0.00065;
        Self {
            history: vec![0.0; max_delay.ceil() as usize + 2],
            cursor: 0,
            max_delay,
            low: [0.0; 2],
            alpha: 1.0 - (-std::f32::consts::TAU * 2200.0 / sr).exp(),
            strength,
        }
    }
    fn read(&self, delay: f64) -> f32 {
        let position = (self.cursor as f64 - delay).rem_euclid(self.history.len() as f64);
        let index = position.floor() as usize;
        let fraction = (position - index as f64) as f32;
        self.history[index] * (1.0 - fraction)
            + self.history[(index + 1) % self.history.len()] * fraction
    }
    pub(crate) fn process(&mut self, sample: f32, pan: f32) -> [f32; 2] {
        self.history[self.cursor] = sample;
        let far = [pan.max(0.0), (-pan).max(0.0)];
        let angle = std::f32::consts::FRAC_PI_4 + pan * std::f32::consts::FRAC_PI_6;
        let gains = [angle.cos(), angle.sin()];
        let mut output = [0.0; 2];
        for ear in 0..2 {
            let shadow = far[ear] * self.strength;
            let delayed = self.read(self.max_delay * shadow as f64);
            // Run filter state continuously; smoothly mix it in as the ear becomes occluded.
            self.low[ear] += self.alpha * (delayed - self.low[ear]);
            output[ear] = ((1.0 - shadow) * delayed + shadow * self.low[ear]) * gains[ear];
        }
        self.cursor = (self.cursor + 1) % self.history.len();
        output
    }
}

pub(crate) struct Depth {
    sources: [Box<dyn Generator>; 3],
    panners: [HeadphonePanner; 2],
    phase: f64,
    step: f64,
    depth: f32,
}
impl Depth {
    pub(crate) fn new(sources: [Box<dyn Generator>; 3], config: &SpatialConfig, sr: f32) -> Self {
        Self {
            sources,
            panners: std::array::from_fn(|_| HeadphonePanner::new(sr, config.headphone_strength)),
            phase: 0.0,
            step: std::f64::consts::TAU * config.motion_rate as f64 / sr as f64,
            depth: config.motion_depth,
        }
    }
}
impl Generator for Depth {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut samples = [0.0; 3];
        for (source, sample) in self.sources.iter_mut().zip(&mut samples) {
            source.next_frame(std::slice::from_mut(sample));
        }
        let a = self.panners[0].process(samples[1], self.phase.sin() as f32 * self.depth);
        let b = self.panners[1].process(
            samples[2],
            (self.phase * 0.73 + std::f64::consts::PI).sin() as f32 * self.depth,
        );
        let center = samples[0] * std::f32::consts::FRAC_1_SQRT_2;
        let scale = 1.0 / (std::f32::consts::FRAC_1_SQRT_2 + 2.0);
        write_stereo(
            out,
            (center + a[0] + b[0]) * scale,
            (center + a[1] + b[1]) * scale,
        );
        self.phase += self.step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn center_is_symmetric_and_immediate() {
        let mut panner = HeadphonePanner::new(48000.0, 1.0);
        let frame = panner.process(1.0, 0.0);
        assert!((frame[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((frame[0] - frame[1]).abs() < 1e-6);
    }
    #[test]
    fn impulse_arrives_at_near_ear_first_at_multiple_sample_rates() {
        for sr in [44100.0, 48000.0, 96000.0] {
            for pan in [-1.0, 1.0] {
                let mut panner = HeadphonePanner::new(sr, 1.0);
                let near = if pan > 0.0 { 1 } else { 0 };
                let far = 1 - near;
                let mut first_far = None;
                for i in 0..100 {
                    let frame = panner.process(if i == 0 { 1.0 } else { 0.0 }, pan);
                    if i == 0 {
                        assert!(frame[near] > 0.9);
                        assert_eq!(frame[far], 0.0);
                    }
                    if frame[far].abs() > 1e-7 && first_far.is_none() {
                        first_far = Some(i);
                    }
                }
                let expected = (sr * 0.00065).floor() as usize;
                assert_eq!(first_far, Some(expected));
            }
        }
    }
    #[test]
    fn zero_strength_removes_delay_and_head_shadow() {
        let mut panner = HeadphonePanner::new(48000.0, 0.0);
        let frame = panner.process(1.0, 1.0);
        assert!(frame[0] > 0.25 && frame[1] > 0.9);
        assert_eq!(panner.process(0.0, 1.0), [0.0; 2]);
    }
    #[test]
    fn far_ear_shadow_attenuates_high_frequencies_more_than_low_frequencies() {
        let ratio = |hz: f32| {
            let mut panner = HeadphonePanner::new(48000.0, 1.0);
            let mut power = [0.0; 2];
            for i in 0..48000 {
                let frame =
                    panner.process((std::f32::consts::TAU * hz * i as f32 / 48000.0).sin(), 1.0);
                if i > 24000 {
                    for ear in 0..2 {
                        power[ear] += frame[ear] * frame[ear];
                    }
                }
            }
            (power[0] / power[1]).sqrt()
        };
        assert!(ratio(10000.0) < ratio(100.0) * 0.4);
    }
    #[test]
    fn moving_delay_is_finite_bounded_and_keeps_preallocated_storage() {
        let mut panner = HeadphonePanner::new(48000.0, 1.0);
        let allocation = panner.history.as_ptr();
        for i in 0..480000 {
            let pan = (i as f32 / 7000.0).sin();
            let frame = panner.process((i as f32 / 7.0).sin(), pan);
            assert!(frame.iter().all(|s| s.is_finite() && s.abs() <= 1.00001));
        }
        assert_eq!(allocation, panner.history.as_ptr());
    }
}
