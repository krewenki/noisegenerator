//! Noise generator trait and implementations.

use clap::ValueEnum;

mod white;
mod pink;
mod brown;
mod blue;
mod violet;
mod gray;
mod velvet;
mod perlin;
mod binaural;

pub use white::White;
pub use pink::Pink;
pub use brown::Brown;
pub use blue::Blue;
pub use violet::Violet;
pub use gray::Gray;
pub use velvet::Velvet;
pub use perlin::Perlin;
pub use binaural::Binaural;

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
}

/// Create a generator based on noise kind and parameters.
pub fn create_generator(kind: NoiseKind, gain: f32, sample_rate: f32) -> Box<dyn Generator> {
    match kind {
        NoiseKind::White  => Box::new(White::new(gain)),
        NoiseKind::Pink   => Box::new(Pink::new(gain)),
        NoiseKind::Brown  => Box::new(Brown::new(gain, sample_rate)),
        NoiseKind::Blue   => Box::new(Blue::new(gain, sample_rate)),
        NoiseKind::Violet => Box::new(Violet::new(gain, sample_rate)),
        NoiseKind::Gray   => Box::new(Gray::new(gain, sample_rate)),
        NoiseKind::Velvet => Box::new(Velvet::new(gain, sample_rate)),
        NoiseKind::Perlin => Box::new(Perlin::new(gain, sample_rate)),
    }
}
