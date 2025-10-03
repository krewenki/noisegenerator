# Noise Generator

A high-performance Rust CLI application for generating various types of audio noise with real-time playback and file export capabilities.

## Features

- 🎵 **8 Noise Types**: White, Pink, Brown, Blue, Violet, Gray, Velvet, and Perlin
- 🎧 **Binaural Mode**: Dual generators with smooth stereo panning for immersive soundscapes
- 🔊 **Real-time Playback**: Stream directly to your audio device via CoreAudio (macOS)
- 💾 **File Export**: Save as high-quality MP3 (320kbps) or lossless WAV
- ⚡ **High Performance**: Generate audio at 23-45x realtime speed
- 🌙 **Long Duration Support**: Perfect for 8-24 hour sleep/focus sessions

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- macOS (CoreAudio support)

### Build from Source

```bash
git clone https://github.com/yourusername/noisegenerator.git
cd noisegenerator
cargo build --release
```

The binary will be at `./target/release/noisegenerator`

## Quick Start

```bash
# Real-time playback of white noise (Ctrl-C to stop)
./target/release/noisegenerator white

# 30 seconds of pink noise with binaural panning
./target/release/noisegenerator pink --binaural --seconds 30

# Export 8 hours of brown noise as MP3
./target/release/noisegenerator brown --save --seconds 28800

# Save 5 minutes of Perlin noise as lossless WAV
./target/release/noisegenerator perlin --save --format wav --seconds 300 --gain 0.15
```

## Noise Types

| Type | Description | Use Case |
|------|-------------|----------|
| **White** | Equal energy across all frequencies | Masking, focus |
| **Pink** | 1/f falloff, balanced frequency spectrum | Sleep, relaxation |
| **Brown** | Low-frequency emphasis, deep rumble | Deep sleep, meditation |
| **Blue** | High-frequency emphasis, crisp | Productivity, alertness |
| **Violet** | Very high-frequency, sharp | Tinnitus masking |
| **Gray** | Psychoacoustically flat (equal loudness) | Natural masking |
| **Velvet** | Sparse impulses, crackling texture | Light rain ambience |
| **Perlin** | Smooth organic waves, natural | Ambient soundscapes |

## Usage

```
noisegenerator [OPTIONS] [KIND]

Arguments:
  [KIND]  Noise type [default: white]
          [possible values: white, pink, brown, blue, violet, gray, velvet, perlin]

Options:
  -g, --gain <GAIN>
          Linear gain (0.0..1.0). 0.1–0.3 is comfortable [default: 0.2]
          
  --seconds <SECONDS>
          Target duration in seconds (omit to run until Ctrl-C)
          
  --save
          Save to file instead of playing. Defaults to 60s if --seconds not specified
          
  --format <FORMAT>
          Output format when saving [default: mp3]
          [possible values: wav, mp3]
          
  --binaural
          Binaural mode: dual generators with slow stereo panning
          Creates an immersive spatial audio experience
          
  -h, --help
          Print help
          
  -V, --version
          Print version
```

## Examples

### Real-time Playback

```bash
# Basic white noise
noisegenerator white

# Pink noise at 15% volume
noisegenerator pink --gain 0.15

# Binaural brown noise for 1 hour
noisegenerator brown --binaural --seconds 3600
```

### File Export

```bash
# 60-second MP3 (default)
noisegenerator white --save

# 2-hour sleep noise as MP3
noisegenerator pink --binaural --save --seconds 7200

# 5-minute lossless WAV
noisegenerator perlin --save --format wav --seconds 300

# Full 8-hour sleep session
noisegenerator brown --binaural --save --seconds 28800
```

### Binaural Mode

Binaural mode creates an immersive spatial audio experience by:
- Running two independent noise generators simultaneously
- Slowly panning one generator across the stereo field (0.1 Hz = 10-second cycle)
- Using equal-power panning law for smooth transitions
- Ideal for headphone listening and deep focus/sleep sessions

```bash
# Binaural pink noise (recommended for sleep)
noisegenerator pink --binaural --seconds 28800 --save
```

## Performance

Performance benchmarks on Apple M1 (release build):

| Generator | Mode | Export Speed |
|-----------|------|--------------|
| White | Mono | **45x realtime** |
| Pink | Mono | **42x realtime** |
| Perlin | Binaural | **23x realtime** |

This means:
- 8-hour audio file generates in ~10-12 minutes
- 24-hour audio file generates in ~30-40 minutes

**Important**: Always use the release build (`cargo build --release`) for maximum performance. Debug builds are ~5-6x slower.

## Project Structure

```
src/
├── lib.rs              # Library entry point
├── main.rs             # CLI wrapper (42 lines)
├── generators/         # Noise generation algorithms
│   ├── mod.rs         # Generator trait and factory
│   ├── white.rs       # White noise
│   ├── pink.rs        # Pink noise (Voss-McCartney)
│   ├── brown.rs       # Brown noise (integrated)
│   ├── blue.rs        # Blue noise (differentiated)
│   ├── violet.rs      # Violet noise (double-differentiated)
│   ├── gray.rs        # Gray noise (equal-loudness weighted)
│   ├── velvet.rs      # Velvet noise (sparse impulses)
│   ├── perlin.rs      # Perlin noise (smooth organic)
│   └── binaural.rs    # Binaural wrapper
└── audio/             # Audio I/O
    ├── mod.rs         # Module exports
    ├── playback.rs    # Real-time streaming
    └── export.rs      # File export (WAV/MP3)
```

## Technical Details

- **Sample Rate**: 48kHz
- **Channels**: Stereo (2 channels)
- **Bit Depth**: 16-bit (exports)
- **MP3 Encoding**: 320kbps, LAME "Best" quality
- **Architecture**: Zero-copy generator design, trait-based polymorphism

## Use Cases

### Sleep & Relaxation
```bash
# Classic pink noise for 8 hours
noisegenerator pink --save --seconds 28800

# Deep brown noise with binaural panning
noisegenerator brown --binaural --save --seconds 28800
```

### Focus & Productivity
```bash
# White noise for distraction masking
noisegenerator white --seconds 7200

# Blue noise for alertness
noisegenerator blue --gain 0.15 --seconds 3600
```

### Tinnitus Masking
```bash
# Violet noise at comfortable volume
noisegenerator violet --gain 0.1
```

### Ambient Soundscapes
```bash
# Organic Perlin waves
noisegenerator perlin --binaural --seconds 3600

# Light rain effect (velvet)
noisegenerator velvet --gain 0.25 --seconds 1800
```

## Troubleshooting

### Clicks/Pops in Long Files

If you hear clicks or pops in long-duration files (8+ hours), ensure you're using the latest version which includes floating-point precision fixes. See [CLICKS_FIX.md](CLICKS_FIX.md) for details.

### Slow Generation

Always use the release build:
```bash
cargo build --release
./target/release/noisegenerator  # Not ./target/debug/
```

### Audio Device Not Found

Ensure your macOS audio output device is properly configured:
```bash
# Check available devices
system_profiler SPAudioDataType
```

## Documentation

- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Detailed architecture documentation
- [PERFORMANCE.md](PERFORMANCE.md) - Performance optimization guide
- [CLICKS_FIX.md](CLICKS_FIX.md) - Audio quality troubleshooting

## Library Usage

The project is structured as a library, so you can use it in your own Rust projects:

```rust
use noisegenerator::generators::{NoiseKind, create_generator};
use noisegenerator::audio::{play_audio, save_to_file, OutputFormat};

// Create a generator
let mut gen = create_generator(NoiseKind::Pink, 0.2, 48000.0);

// Generate samples
let mut frame = vec![0.0f32; 2];
gen.next_frame(&mut frame);

// Or use high-level functions
play_audio(NoiseKind::White, 0.2, false, Some(60))?;
save_to_file(NoiseKind::Pink, 0.2, true, 3600, OutputFormat::Mp3)?;
```

## License

MIT License - See [LICENSE](LICENSE) for details

## Contributing

Contributions welcome! Areas for improvement:

- Additional noise types (Simplex, fractal, bandpass-filtered)
- Cross-platform support (Windows, Linux)
- GUI interface
- Real-time parameter modulation
- Additional export formats (FLAC, OGG)

## Author

Warren Krewenki

## Acknowledgments

- Pink noise implementation based on the Voss-McCartney algorithm
- Perlin noise adapted from Ken Perlin's reference implementation
- MP3 encoding via LAME encoder
- Audio I/O via cpal library
