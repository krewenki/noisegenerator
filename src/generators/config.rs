//! One construction path shared by playback and all exports.
use super::{
    create_generator, Binaural, Generator, NoiseKind, Spatial, SpatialConfig, SpatialMode,
};
use anyhow::{ensure, Result};

#[derive(Clone, Debug)]
pub struct GeneratorConfig {
    pub kind: NoiseKind,
    pub gain: f32,
    pub spatial: SpatialConfig,
}

impl GeneratorConfig {
    pub fn new(kind: NoiseKind, gain: f32) -> Self {
        Self {
            kind,
            gain,
            spatial: SpatialConfig::default(),
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
        Ok(())
    }
}

pub fn create_configured_generator(
    config: &GeneratorConfig,
    sample_rate: f32,
) -> Result<Box<dyn Generator>> {
    config.validate(sample_rate)?;
    let source = || create_generator(config.kind, config.gain, sample_rate);
    Ok(match config.spatial.mode {
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
    })
}
