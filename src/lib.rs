//! Noise generator library for creating various types of audio noise.
//!
//! This library provides multiple noise generation algorithms including:
//! - White, Pink, Brown noise (classic colored noises)
//! - Blue, Violet noise (high-frequency emphasis)
//! - Gray noise (psychoacoustically weighted)
//! - Velvet noise (sparse impulses)
//! - Perlin noise (smooth, organic evolution)
//!
//! It also supports binaural/spatial audio mode for immersive soundscapes.

pub mod generators;
pub mod audio;

pub use generators::{Generator, NoiseKind};
