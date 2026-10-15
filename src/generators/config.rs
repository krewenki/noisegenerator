//! One construction path shared by playback and all exports.
use super::effects::{Effects, EffectsConfig, Fade};
use super::mixer::Mixer;
use super::{
    Binaural, Generator, LayerConfig, NoiseKind, SoundConfig, Spatial, SpatialConfig, SpatialMode,
};
use anyhow::{ensure, Result};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Clone, Debug)]
pub struct GeneratorConfig {
    pub kind: NoiseKind,
    pub gain: f32,
    pub spatial: SpatialConfig,
    pub effects: EffectsConfig,
    pub sound: SoundConfig,
    pub layers: Vec<LayerConfig>,
    pub seed: Option<u64>,
}

impl GeneratorConfig {
    pub fn new(kind: NoiseKind, gain: f32) -> Self {
        Self {
            kind,
            gain,
            spatial: SpatialConfig::default(),
            effects: EffectsConfig::default(),
            sound: SoundConfig::default(),
            layers: Vec::new(),
            seed: None,
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
        if let Some(density) = self.sound.density {
            ensure!(
                density.is_finite() && (0.0..=sample_rate).contains(&density),
                "density must be finite and 0..=sample rate events/sec"
            );
        }
        if let Some(rate) = self.sound.texture_rate {
            ensure!(
                rate.is_finite() && (0.0..=1.0).contains(&rate),
                "texture rate must be 0..=1 Hz"
            );
        }
        for level in [
            self.sound.event_level,
            self.sound.variation,
            self.sound.tone_level,
        ] {
            ensure!(
                level.is_finite() && (0.0..=1.0).contains(&level),
                "event level, variation, and tone level must be 0..=1"
            );
        }
        ensure!(
            self.sound.tone_hz.is_finite() && self.sound.tone_hz >= 10.0,
            "tone frequency must be finite and at least 10 Hz"
        );
        for kind in std::iter::once(self.kind).chain(self.layers.iter().map(|l| l.kind)) {
            let top_ratio = match kind {
                NoiseKind::Fan => 3.0,
                NoiseKind::Drone => 1.5,
                _ => 0.0,
            };
            ensure!(
                top_ratio == 0.0 || self.sound.tone_hz * top_ratio * 1.002 < sample_rate * 0.5,
                "tone harmonics must be below Nyquist"
            );
        }
        ensure!(
            self.sound.perlin_speed.is_finite()
                && self.sound.perlin_speed > 0.0
                && self.sound.perlin_speed <= 1000.0,
            "Perlin speed must be greater than 0 and at most 1000 Hz"
        );
        ensure!(
            (1..=10).contains(&self.sound.perlin_octaves),
            "Perlin octaves must be 1..=10"
        );
        ensure!(
            self.layers.len() <= 16,
            "at most 16 additional layers are supported"
        );
        for layer in &self.layers {
            ensure!(
                layer.level.is_finite() && (0.0..=10.0).contains(&layer.level),
                "layer level must be 0..=10"
            );
            ensure!(
                layer.pan.is_finite() && (-1.0..=1.0).contains(&layer.pan),
                "layer pan must be -1..=1"
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
    let mut streams = config
        .seed
        .map(StdRng::seed_from_u64)
        .unwrap_or_else(StdRng::from_entropy);
    let mut source = |kind| {
        super::sounds::source(
            kind,
            config.gain,
            sample_rate,
            &config.sound,
            Some(streams.gen()),
        )
    };
    let mut generator: Box<dyn Generator> = match config.spatial.mode {
        SpatialMode::Mono => source(config.kind),
        SpatialMode::Binaural => Box::new(Binaural::with_motion(
            source(config.kind),
            source(config.kind),
            sample_rate,
            config.spatial.motion_rate,
            config.spatial.motion_depth,
        )),
        _ => Box::new(Spatial::new(
            [
                source(config.kind),
                source(config.kind),
                source(config.kind),
            ],
            config.spatial.clone(),
            sample_rate,
        )),
    };
    if !config.layers.is_empty() {
        let layers = config
            .layers
            .iter()
            .map(|layer| (source(layer.kind), layer.clone()))
            .collect();
        generator = Box::new(Mixer::new(generator, layers));
    }
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
