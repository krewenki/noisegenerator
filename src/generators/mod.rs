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
    Rain,
    Fire,
    Fan,
    Drone,
}

/// Create a generator based on noise kind and parameters.
pub fn create_generator(kind: NoiseKind, gain: f32, sample_rate: f32) -> Box<dyn Generator> {
    sounds::source(kind, gain, sample_rate, &SoundConfig::default(), None)
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

mod events;
mod mixer;
mod sounds;
mod tones;
pub use mixer::LayerConfig;
pub use sounds::SoundConfig;
