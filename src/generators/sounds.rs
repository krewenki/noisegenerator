//! Parameterized sound construction and deterministic random streams.
use super::*;
use rand::{rngs::StdRng, SeedableRng};

#[derive(Clone, Debug)]
pub struct SoundConfig {
    /// Events per second for velvet, rain, and fire; None uses each sound's default.
    pub density: Option<f32>,
    pub event_level: f32,
    pub texture_rate: Option<f32>,
    pub variation: f32,
    pub tone_hz: f32,
    pub tone_level: f32,
    pub perlin_speed: f32,
    pub perlin_octaves: usize,
}
impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            density: None,
            event_level: 0.5,
            texture_rate: None,
            variation: 1.0,
            tone_hz: 110.0,
            tone_level: 0.2,
            perlin_speed: 50.0,
            perlin_octaves: 6,
        }
    }
}

pub(crate) fn source(
    kind: NoiseKind,
    gain: f32,
    sr: f32,
    config: &SoundConfig,
    seed: Option<u64>,
) -> Box<dyn Generator> {
    let rng = seed
        .map(StdRng::seed_from_u64)
        .unwrap_or_else(StdRng::from_entropy);
    match kind {
        NoiseKind::White => Box::new(White::with_rng(gain, rng)),
        NoiseKind::Pink => Box::new(Pink::with_rng(gain, rng)),
        NoiseKind::Brown => Box::new(Brown::with_rng(gain, sr, rng)),
        NoiseKind::Blue => Box::new(Blue::with_rng(gain, sr, rng)),
        NoiseKind::Violet => Box::new(Violet::with_rng(gain, sr, rng)),
        NoiseKind::Gray => Box::new(Gray::with_rng(gain, sr, rng)),
        NoiseKind::Velvet => {
            let mut generator = Velvet::with_rng(gain, sr, rng);
            generator.set_density(config.density.unwrap_or(1000.0), sr);
            Box::new(generator)
        }
        NoiseKind::Perlin => {
            let mut generator = Perlin::with_rng(gain, sr, rng);
            generator.set_texture(config.perlin_speed, config.perlin_octaves, sr);
            Box::new(generator)
        }
        NoiseKind::Ocean | NoiseKind::Wind => Box::new(textures::Texture::with_config(
            kind,
            gain,
            sr,
            Box::new(Pink::with_rng(1.0, rng)),
            config,
        )),
        NoiseKind::Rain | NoiseKind::Fire => {
            Box::new(events::Events::new(kind, gain, sr, config, rng))
        }
        NoiseKind::Fan | NoiseKind::Drone => {
            Box::new(tones::Tonal::new(kind, gain, sr, config, rng))
        }
    }
}
