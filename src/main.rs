use clap::Parser;
use noisegenerator::audio::{play_audio_with_config, save_to_file_with_config, OutputFormat};
use noisegenerator::generators::{GeneratorConfig, LayerConfig, NoiseKind, SpatialMode};

/// Simple CLI noise generator for macOS (CoreAudio via cpal).
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Noise type
    #[arg(value_enum, default_value_t = NoiseKind::White)]
    kind: NoiseKind,
    /// Linear gain (0.0..1.0). 0.1–0.3 is comfortable.
    #[arg(long, short, default_value_t = 0.2)]
    gain: f32,
    /// Target duration in seconds (omit to run until Ctrl-C).
    #[arg(long)]
    seconds: Option<u64>,
    /// Save to file instead of playing. Defaults to 60s if --seconds not specified.
    #[arg(long)]
    save: bool,
    /// Output format when saving (mp3 for long recordings, wav for lossless)
    #[arg(long, value_enum, default_value_t = OutputFormat::Mp3)]
    format: OutputFormat,
    /// Binaural mode: dual generators with slow stereo panning for immersive soundscapes
    #[arg(long)]
    binaural: bool,
    /// Stereo placement (binaural is panning, not beat tones).
    #[arg(long, value_enum, conflicts_with = "binaural")]
    mode: Option<SpatialMode>,
    /// Independent stereo mix for wide mode (0..=1).
    #[arg(long, default_value_t = 1.0)]
    width: f32,
    /// Pan cycles per second (0..=1; zero freezes movement).
    #[arg(long, default_value_t = 0.1)]
    motion_rate: f32,
    /// Pan excursion (0..=1).
    #[arg(long, default_value_t = 1.0)]
    motion_depth: f32,
    /// Low-pass cutoff in Hz (at least 10, below Nyquist).
    #[arg(long)]
    lowpass: Option<f32>,
    /// High-pass cutoff in Hz (at least 10, below Nyquist).
    #[arg(long)]
    highpass: Option<f32>,
    /// Breathing envelope rate in Hz (0..=1).
    #[arg(long, default_value_t = 0.1)]
    breath_rate: f32,
    /// Breathing depth (0 disables, 1 reaches silence).
    #[arg(long, default_value_t = 0.0)]
    breath_depth: f32,
    /// Smooth fade-in duration in seconds.
    #[arg(long, default_value_t = 0.0)]
    fade_in: f64,
    /// Smooth fade-out duration in seconds (requires a timed session).
    #[arg(long, default_value_t = 0.0)]
    fade_out: f64,
    /// Add a fixed-position layer, KIND:LEVEL:PAN (repeatable, max 16).
    #[arg(long)]
    layer: Vec<LayerConfig>,
    /// Reproduce the same sample sequence (same version/config/sample rate).
    #[arg(long)]
    seed: Option<u64>,
    /// Events per second for velvet, rain, and fire (sound-specific default).
    #[arg(long)]
    density: Option<f32>,
    /// Rain/fire event prominence (0..=1).
    #[arg(long, default_value_t = 0.5)]
    event_level: f32,
    /// Ocean/wind base swell rate in Hz (0..=1).
    #[arg(long)]
    texture_rate: Option<f32>,
    /// Ocean/wind amplitude variation and fan/drone speed variation (0..=1).
    #[arg(long, default_value_t = 1.0)]
    variation: f32,
    /// Fan/drone fundamental frequency in Hz.
    #[arg(long, default_value_t = 110.0)]
    tone_hz: f32,
    /// Fan/drone tonal mix (0..=1).
    #[arg(long, default_value_t = 0.2)]
    tone_level: f32,
    /// Perlin noise-space traversal speed (greater than 0, at most 1000 Hz).
    #[arg(long, default_value_t = 50.0)]
    perlin_speed: f32,
    /// Perlin octave count (1..=10).
    #[arg(long, default_value_t = 6)]
    perlin_octaves: usize,
    /// Interaural timing/head-shadow strength for depth mode (0..=1).
    #[arg(long, default_value_t = 1.0)]
    headphone_strength: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = GeneratorConfig::new(args.kind, args.gain);
    config.spatial.mode = args.mode.unwrap_or(if args.binaural {
        SpatialMode::Binaural
    } else {
        SpatialMode::Mono
    });
    config.spatial.width = args.width;
    config.spatial.headphone_strength = args.headphone_strength;
    config.spatial.motion_rate = args.motion_rate;
    config.spatial.motion_depth = args.motion_depth;
    config.effects.lowpass = args.lowpass;
    config.effects.highpass = args.highpass;
    config.effects.breath_rate = args.breath_rate;
    config.effects.breath_depth = args.breath_depth;
    config.effects.fade_in = args.fade_in;
    config.effects.fade_out = args.fade_out;
    config.layers = args.layer;
    config.seed = args.seed;
    config.sound.density = args.density;
    config.sound.event_level = args.event_level;
    config.sound.texture_rate = args.texture_rate;
    config.sound.variation = args.variation;
    config.sound.tone_hz = args.tone_hz;
    config.sound.tone_level = args.tone_level;
    config.sound.perlin_speed = args.perlin_speed;
    config.sound.perlin_octaves = args.perlin_octaves;
    anyhow::ensure!(
        args.fade_out == 0.0 || args.save || args.seconds.is_some(),
        "fade-out requires --seconds for playback"
    );

    if args.save {
        // Save mode: generate audio to file
        let duration = args.seconds.unwrap_or(60);
        save_to_file_with_config(&config, duration, args.format)?;
    } else {
        // Playback mode: stream audio to output device
        play_audio_with_config(&config, args.seconds)?;
    }
    Ok(())
}
