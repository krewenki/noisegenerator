//! Noise generator trait and implementations.

use clap::ValueEnum;

mod binaural;
mod blue;
mod brown;
mod gray;
mod perlin;
mod pink;
mod velvet;
mod violet;
mod white;

pub use binaural::Binaural;
pub use blue::Blue;
pub use brown::Brown;
pub use gray::Gray;
pub use perlin::Perlin;
pub use pink::Pink;
pub use velvet::Velvet;
pub use violet::Violet;
pub use white::White;

/// Trait for all noise generators.
pub trait Generator: Send {
    /// Generate one frame of audio (all channels).
    fn next_frame(&mut self, out: &mut [f32]);
}

/// Types of noise that can be generated.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum NoiseKind {
    White,
    Pink,
    Brown,
    Blue,
    Violet,
    Gray,
    Velvet,
    Perlin,
    Ocean,
    Wind,
}

/// Create a generator based on noise kind and parameters.
pub fn create_generator(kind: NoiseKind, gain: f32, sample_rate: f32) -> Box<dyn Generator> {
    match kind {
        NoiseKind::Ocean | NoiseKind::Wind => Box::new(textures::Texture::new(
            kind,
            gain,
            sample_rate,
            Box::new(Pink::new(1.0)),
        )),
        NoiseKind::White => Box::new(White::new(gain)),
        NoiseKind::Pink => Box::new(Pink::new(gain)),
        NoiseKind::Brown => Box::new(Brown::new(gain, sample_rate)),
        NoiseKind::Blue => Box::new(Blue::new(gain, sample_rate)),
        NoiseKind::Violet => Box::new(Violet::new(gain, sample_rate)),
        NoiseKind::Gray => Box::new(Gray::new(gain, sample_rate)),
        NoiseKind::Velvet => Box::new(Velvet::new(gain, sample_rate)),
        NoiseKind::Perlin => Box::new(Perlin::new(gain, sample_rate)),
    }
}

mod config;
mod spatial;
pub use config::{
    create_configured_generator, create_configured_generator_for_duration, GeneratorConfig,
};
pub use spatial::{Spatial, SpatialConfig, SpatialMode};

mod effects;
mod textures;
pub use effects::{Biquad, EffectsConfig};
