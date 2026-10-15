# Noise Generator

A high-performance Rust CLI application for generating various types of audio noise with real-time playback and file export capabilities.

## Features

- 🎵 **14 Sounds**: Eight noise types plus Ocean, Wind, Rain, Fire, Fan, and Drone
- 🎧 **Headphone Modes**: Wide stereo, binaural panning, three-source movement, and drift
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

## Headphone spatial modes

All modes below produce ordinary stereo, compatible with headphones, WAV, and MP3.
These are spatial noise effects, not binaural-beat tones or HRTF rendering.

```bash
# Spacious but stationary: independent left and right noise
noisegenerator pink --mode wide --width 0.8
# Center bed with two slowly moving sources
noisegenerator brown --mode trinaural --motion-rate 0.04 --motion-depth 0.7
# Multi-period wandering instead of a single repeating pendulum
noisegenerator perlin --mode drift --save --format wav --seconds 60
```

- `--mode mono|wide|binaural|trinaural|drift` defaults to `mono` (the original sound).
- `--binaural` remains supported with its original default mix and motion. It conflicts with `--mode`.
- `--width 0..1` controls wide mode: shared mono at zero, independent channels at one.
- `--motion-rate 0..1` sets the base movement frequency in Hz (default `0.1`). Zero freezes placement.
- `--motion-depth 0..1` sets movement extent (default `1`). Zero centers moving sources.
- Trinaural and drift use conservative fixed mix headroom; they can be quieter than the legacy binaural mix.
- Mono devices receive a stereo downmix. Additional device channels repeat left/right, as before; these are not surround modes.

Library callers can use `GeneratorConfig` and `create_configured_generator`, or
`audio::{play_audio_with_config, save_to_file_with_config}`. The original APIs remain available.

## Evolving textures and effects

`ocean` and `wind` are procedural textures built from filtered pink noise:
ocean has overlapping-timescale swells with brighter crests; wind has slow gusts
and an evolving low-pass cutoff. These are synthesized approximations, not recordings.
They work with all headphone modes and both export formats.

```bash
noisegenerator ocean --mode wide --lowpass 4000
noisegenerator wind --mode drift --highpass 60 --lowpass 2500
noisegenerator pink --breath-rate 0.08 --breath-depth 0.3
noisegenerator ocean --save --seconds 300 --fade-in 3 --fade-out 10
```

Effects (disabled by default):

- `--lowpass HZ` / `--highpass HZ`: second-order Butterworth filters. Cutoffs must be at least 10 Hz, below the actual output sample rate's Nyquist frequency, and highpass must be below lowpass.
- `--breath-rate 0..1`: envelope frequency in Hz (default `0.1`).
- `--breath-depth 0..1`: attenuation depth; zero disables breathing, one reaches silence. A zero rate freezes the envelope at full level.
- `--fade-in SECONDS` / `--fade-out SECONDS`: smooth sine-squared fades. Overlapping fades multiply. Fade-out requires a finite playback duration; saving uses the export duration (60 seconds by default). Ctrl-C still stops playback immediately.

Processing order: sound source → spatial mix → highpass → lowpass → breathing → fades.
Filter state is independent per ear. Filters can overshoot on transients, so leave gain headroom.
For library fade-out use `create_configured_generator_for_duration` with a duration matching the output frame count.

## Layers, event sounds, and reproducible sessions

Four more procedural sounds:

| Sound | Character |
|---|---|
| `rain` | Filtered noise bed and short, damped resonant drops |
| `fire` | Low rumble with irregular, enveloped crackling noise bursts |
| `fan` | Filtered noise with a quiet motor fundamental and harmonics |
| `drone` | Filtered noise with slowly detuned sinusoidal layers |

```bash
# Base pink noise has weight 1; additional layers have relative weights and fixed pans
noisegenerator pink --mode wide --layer brown:0.5:-0.4 --layer velvet:0.1:0.7
noisegenerator rain --mode trinaural --density 120 --event-level 0.6
noisegenerator fire --density 8 --lowpass 4000
noisegenerator fan --tone-hz 90 --tone-level 0.15 --variation 0.5
noisegenerator drone --tone-hz 110 --tone-level 0.7
noisegenerator ocean --texture-rate 0.06 --variation 0.8
noisegenerator perlin --perlin-speed 35 --perlin-octaves 5
noisegenerator rain --seed 42 --save --format wav --seconds 60
```

- Repeat `--layer KIND:LEVEL:PAN` up to 16 times. Level is relative to the base (0..10), pan is -1..1. Base placement uses `--mode`; added layers are fixed-position mono sources with equal-power panning.
- Mixing divides by `1 + sum(layer levels)` to reserve fixed headroom. This is not perceived-loudness matching or an automatic leveler. A zero-level layer has no audible effect.
- `--density` sets events/sec for velvet/rain/fire. Defaults: 1000/100/8 respectively. Zero disables events; rain/fire still have a background bed unless `--event-level 1` is set. Rain/fire use a fixed pool of 32 voices and steal voices when saturated, so extreme densities change the texture.
- `--event-level 0..1` blends rain/fire bed and events (default 0.5).
- `--texture-rate 0..1` controls ocean/wind swell frequency (Hz); defaults 0.09/0.045. Actual timing also varies slowly. Zero freezes the swell position.
- `--variation 0..1` controls ocean/wind amplitude variation and fan/drone speed variation (default 1).
- `--tone-hz` sets fan/drone pitch (default 110); harmonics must remain below Nyquist. `--tone-level 0..1` controls their tonal mix (default 0.2).
- `--perlin-speed` sets traversal speed (0..1000, exclusive of zero; default 50); `--perlin-octaves` sets 1..10 layers (default 6).
- Sound controls apply to matching base and layer sounds. Master `--gain` still controls the entire mix. Effects and fades run after layer mixing.
- `--seed INTEGER` makes audio samples reproducible for the same version, configuration, and sample rate. Voices and layers receive independent derived streams. Export timestamps/encoded metadata are not part of that contract. Omit it for fresh randomness.

## Experimental headphone depth

`--mode depth` extends the three-source layout with lateral timing and spectral
cues. The farther ear gets a smoothly interpolated delay (up to 650 microseconds)
and increasing high-frequency attenuation. Both ears retain some signal even at
the extreme positions, unlike hard amplitude panning. Added layers use the same
processing at their fixed pan positions.

```bash
noisegenerator pink --mode depth --motion-rate 0.04 --motion-depth 0.8
noisegenerator rain --mode depth --headphone-strength 0.6 --layer brown:0.2:-0.6
noisegenerator ocean --mode depth --seed 42 --save --seconds 60 --fade-in 2 --fade-out 3
```

- `--headphone-strength 0..1` controls the delay/head-shadow blend (default 1). Zero removes those cues but retains the depth mode's level-panning law. It does not disable `--motion-depth`.
- `--motion-rate` and `--motion-depth` control the moving base sources as in trinaural mode. Centered sources have no added timing difference.
- Buffers are allocated during construction, not per audio frame. Delay scales with the actual sample rate; this mode supports 8–384 kHz.
- This is an artistic approximation, **not measured HRTF convolution**, head tracking, distance simulation, or reliable front/back positioning. Perceived externalization varies; listening review is needed.
- Output remains stereo WAV/MP3. Mono downmixing can change the tone because of the delayed signal; this mode is intended for headphones.
- Existing modes and defaults are unchanged. Filtering, breathing, and fades still follow spatial/layer mixing.
