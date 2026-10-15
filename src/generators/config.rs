//! One construction path shared by playback and all exports.
use super::effects::{Effects, EffectsConfig, Fade};
use super::{
    create_generator, Binaural, Generator, NoiseKind, Spatial, SpatialConfig, SpatialMode,
};
use anyhow::{ensure, Result};

#[derive(Clone, Debug)]
pub struct GeneratorConfig {
    pub kind: NoiseKind,
    pub gain: f32,
    pub spatial: SpatialConfig,
    pub effects: EffectsConfig,
}

impl GeneratorConfig {
    pub fn new(kind: NoiseKind, gain: f32) -> Self {
        Self {
            kind,
            gain,
            spatial: SpatialConfig::default(),
            effects: EffectsConfig::default(),
        }
    }

    pub fn validate(&self, sample_rate: f32) -> Result<()> {
        ensure!(
            sample_rate.is_finite() && sample_rate > 0.0,
            "sample rate must be positive and finite"
        );
        ensure!(
            self.gain.is_finite() && (0.0..=1.0).contains(&self.gain),
            "gain must be 0..=1"
        );
        ensure!(
            self.spatial.width.is_finite() && (0.0..=1.0).contains(&self.spatial.width),
            "width must be 0..=1"
        );
        ensure!(
            self.spatial.motion_depth.is_finite()
                && (0.0..=1.0).contains(&self.spatial.motion_depth),
            "motion depth must be 0..=1"
        );
        ensure!(
            self.spatial.motion_rate.is_finite() && (0.0..=1.0).contains(&self.spatial.motion_rate),
            "motion rate must be 0..=1 Hz"
        );
        for hz in [self.effects.lowpass, self.effects.highpass]
            .into_iter()
            .flatten()
        {
            ensure!(
                hz.is_finite() && hz >= 10.0 && hz < sample_rate * 0.5,
                "filter cutoff must be at least 10 Hz and below Nyquist ({} Hz)",
                sample_rate * 0.5
            );
        }
        if let (Some(low), Some(high)) = (self.effects.highpass, self.effects.lowpass) {
            ensure!(low < high, "highpass cutoff must be below lowpass cutoff");
        }
        ensure!(
            self.effects.breath_rate.is_finite() && (0.0..=1.0).contains(&self.effects.breath_rate),
            "breath rate must be 0..=1 Hz"
        );
        ensure!(
            self.effects.breath_depth.is_finite()
                && (0.0..=1.0).contains(&self.effects.breath_depth),
            "breath depth must be 0..=1"
        );
        for seconds in [self.effects.fade_in, self.effects.fade_out] {
            ensure!(
                seconds.is_finite() && seconds >= 0.0,
                "fade durations must be finite and nonnegative"
            );
        }
        Ok(())
    }
}

pub fn create_configured_generator(
    config: &GeneratorConfig,
    sample_rate: f32,
) -> Result<Box<dyn Generator>> {
    create_configured_generator_for_duration(config, sample_rate, None)
}

pub fn create_configured_generator_for_duration(
    config: &GeneratorConfig,
    sample_rate: f32,
    duration: Option<u64>,
) -> Result<Box<dyn Generator>> {
    config.validate(sample_rate)?;
    ensure!(
        config.effects.fade_out == 0.0 || duration.is_some(),
        "fade-out requires a finite --seconds duration"
    );
    let source = || create_generator(config.kind, config.gain, sample_rate);
    let mut generator: Box<dyn Generator> = match config.spatial.mode {
        SpatialMode::Mono => source(),
        SpatialMode::Binaural => Box::new(Binaural::with_motion(
            source(),
            source(),
            sample_rate,
            config.spatial.motion_rate,
            config.spatial.motion_depth,
        )),
        _ => Box::new(Spatial::new(
            [source(), source(), source()],
            config.spatial.clone(),
            sample_rate,
        )),
    };
    if config.effects.lowpass.is_some()
        || config.effects.highpass.is_some()
        || config.effects.breath_depth > 0.0
    {
        generator = Box::new(Effects::new(generator, &config.effects, sample_rate));
    }
    if config.effects.fade_in > 0.0 || config.effects.fade_out > 0.0 {
        generator = Box::new(Fade::new(generator, &config.effects, sample_rate, duration));
    }
    Ok(generator)
}
