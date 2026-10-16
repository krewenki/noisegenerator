use super::{depth::HeadphonePanner, spatial::write_stereo, Generator, NoiseKind};
use clap::ValueEnum;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct LayerConfig {
    pub kind: NoiseKind,
    /// Relative to the positional/base sound, which has weight 1.
    pub level: f32,
    /// Fixed pan (-1 = left, 0 = center, 1 = right).
    pub pan: f32,
}
impl FromStr for LayerConfig {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = value.split(':').collect();
        if parts.len() != 3 {
            return Err("layer must be KIND:LEVEL:PAN (e.g. brown:0.5:-0.3)".into());
        }
        let kind = <NoiseKind as ValueEnum>::from_str(parts[0], true)?;
        let level: f32 = parts[1].parse().map_err(|_| "invalid layer level")?;
        let pan: f32 = parts[2].parse().map_err(|_| "invalid layer pan")?;
        if !level.is_finite() || !(0.0..=10.0).contains(&level) {
            return Err("layer level must be 0..=10".into());
        }
        if !pan.is_finite() || !(-1.0..=1.0).contains(&pan) {
            return Err("layer pan must be -1..=1".into());
        }
        Ok(Self { kind, level, pan })
    }
}

struct Voice {
    source: Box<dyn Generator>,
    left: f32,
    right: f32,
    pan: f32,
    level: f32,
    depth: Option<HeadphonePanner>,
}

pub struct Mixer {
    base: Box<dyn Generator>,
    layers: Vec<Voice>,
    scale: f32,
}
impl Mixer {
    pub fn new(base: Box<dyn Generator>, layers: Vec<(Box<dyn Generator>, LayerConfig)>) -> Self {
        Self::build(base, layers, None)
    }
    pub fn with_depth(
        base: Box<dyn Generator>,
        layers: Vec<(Box<dyn Generator>, LayerConfig)>,
        sr: f32,
        strength: f32,
    ) -> Self {
        Self::build(base, layers, Some((sr, strength)))
    }
    fn build(
        base: Box<dyn Generator>,
        layers: Vec<(Box<dyn Generator>, LayerConfig)>,
        depth: Option<(f32, f32)>,
    ) -> Self {
        let scale = 1.0 / (1.0 + layers.iter().map(|(_, c)| c.level).sum::<f32>());
        let layers = layers
            .into_iter()
            .map(|(source, config)| {
                let angle = (config.pan + 1.0) * std::f32::consts::FRAC_PI_4;
                Voice {
                    source,
                    left: config.level * angle.cos(),
                    right: config.level * angle.sin(),
                    pan: config.pan,
                    level: config.level,
                    depth: depth.map(|(sr, strength)| HeadphonePanner::new(sr, strength)),
                }
            })
            .collect();
        Self {
            base,
            layers,
            scale,
        }
    }
}
impl Generator for Mixer {
    fn next_frame(&mut self, out: &mut [f32]) {
        let mut frame = [0.0; 2];
        self.base.next_frame(&mut frame);
        for voice in &mut self.layers {
            let mut mono = [0.0];
            voice.source.next_frame(&mut mono);
            if let Some(panner) = &mut voice.depth {
                let ears = panner.process(mono[0], voice.pan);
                frame[0] += ears[0] * voice.level;
                frame[1] += ears[1] * voice.level;
            } else {
                frame[0] += mono[0] * voice.left;
                frame[1] += mono[0] * voice.right;
            }
        }
        write_stereo(out, frame[0] * self.scale, frame[1] * self.scale);
    }
}
