use clap::Parser;
use noisegenerator::generators::NoiseKind;
use noisegenerator::audio::{play_audio, save_to_file, OutputFormat};

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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    assert!((0.0..=1.0).contains(&args.gain), "gain must be 0..=1");

    if args.save {
        // Save mode: generate audio to file
        let duration = args.seconds.unwrap_or(60);
        save_to_file(args.kind, args.gain, args.binaural, duration, args.format)?;
    } else {
        // Playback mode: stream audio to output device
        play_audio(args.kind, args.gain, args.binaural, args.seconds)?;
    }
    Ok(())
}
