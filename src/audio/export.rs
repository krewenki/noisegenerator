//! Audio file export functionality (WAV and MP3).

use crate::generators::{
    create_configured_generator_for_duration, GeneratorConfig, NoiseKind, SpatialMode,
};
use anyhow::Result;
use chrono::Local;
use clap::ValueEnum;
use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm};
use std::mem::MaybeUninit;

/// Output file format.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Wav,
    Mp3,
}

/// Save audio to a file in the specified format.
pub fn save_to_file(
    kind: NoiseKind,
    gain: f32,
    binaural: bool,
    duration: u64,
    format: OutputFormat,
) -> Result<()> {
    let mut config = GeneratorConfig::new(kind, gain);
    if binaural {
        config.spatial.mode = SpatialMode::Binaural;
    }
    save_to_file_with_config(&config, duration, format)
}

pub fn save_to_file_with_config(
    config: &GeneratorConfig,
    duration: u64,
    format: OutputFormat,
) -> Result<()> {
    config.validate(48000.0)?;
    match format {
        OutputFormat::Wav => save_to_wav(config, duration),
        OutputFormat::Mp3 => save_to_mp3(config, duration),
    }
}

fn save_to_wav(config: &GeneratorConfig, duration: u64) -> Result<()> {
    let kind = config.kind;
    let gain = config.gain;
    // Fixed sample rate and channels for file output
    let sample_rate = 48000u32;
    let channels = 2u16;
    let sr = sample_rate as f32;

    // Generate filename with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{:?}-{}.wav", kind, timestamp);

    eprintln!("→ Generating {} seconds of {:?} noise", duration, kind);
    eprintln!(
        "→ Output: {} ({}ch @ {} Hz, WAV)",
        filename, channels, sample_rate
    );
    eprintln!("→ Gain: {}", gain);

    // Create WAV writer
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&filename, spec)?;

    // Create generator
    let mut generator = create_configured_generator_for_duration(config, sr, Some(duration))?;

    // Generate samples
    let total_samples = sample_rate as u64 * duration;
    let mut frame = vec![0.0f32; channels as usize];

    for i in 0..total_samples {
        generator.next_frame(&mut frame);

        // Write all channels
        for &sample in &frame {
            // Sanitize sample: replace NaN/Inf with silence to prevent clicks
            let safe_sample = if sample.is_finite() { sample } else { 0.0 };

            // Convert f32 to i16
            let amplitude = 32767.0;
            let value = (safe_sample * amplitude).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(value)?;
        }

        // Progress indicator every second
        if i % sample_rate as u64 == 0 && i > 0 {
            eprint!(
                "\r→ Progress: {}/{} seconds",
                i / sample_rate as u64,
                duration
            );
        }
    }

    eprintln!("\r→ Progress: {}/{} seconds", duration, duration);
    writer.finalize()?;
    eprintln!("✓ Saved to {}", filename);

    Ok(())
}

fn save_to_mp3(config: &GeneratorConfig, duration: u64) -> Result<()> {
    let kind = config.kind;
    let gain = config.gain;
    // Fixed sample rate and channels for file output
    let sample_rate = 48000u32;
    let channels = 2u16;
    let sr = sample_rate as f32;

    // Generate filename with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{:?}-{}.mp3", kind, timestamp);

    eprintln!("→ Generating {} seconds of {:?} noise", duration, kind);
    eprintln!(
        "→ Output: {} ({}ch @ {} Hz, MP3 320kbps)",
        filename, channels, sample_rate
    );
    eprintln!("→ Gain: {}", gain);

    // Create MP3 encoder with high quality settings
    let mut encoder = Builder::new().expect("Create LAME builder");
    encoder
        .set_sample_rate(sample_rate)
        .expect("set sample rate");
    encoder
        .set_num_channels(channels as u8)
        .expect("set channels");
    encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .expect("set quality");
    encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps320)
        .expect("set bitrate");
    let mut encoder = encoder.build().expect("build encoder");

    // Create output file
    let mut output = std::fs::File::create(&filename)?;

    // Create generator
    let mut generator = create_configured_generator_for_duration(config, sr, Some(duration))?;

    // Generate samples in chunks for efficient MP3 encoding
    let total_samples = sample_rate as u64 * duration;
    let mut frame = vec![0.0f32; channels as usize];
    let chunk_size = 1152; // MP3 frame size
    let mut pcm_buffer = Vec::with_capacity(chunk_size * channels as usize);

    for i in 0..total_samples {
        generator.next_frame(&mut frame);

        // Convert f32 samples to i16 and add to buffer
        for &sample in &frame {
            // Sanitize sample: replace NaN/Inf with silence to prevent clicks
            let safe_sample = if sample.is_finite() { sample } else { 0.0 };

            let amplitude = 32767.0;
            let value = (safe_sample * amplitude).clamp(-32768.0, 32767.0) as i16;
            pcm_buffer.push(value);
        }

        // Encode when buffer is full
        if pcm_buffer.len() >= chunk_size * channels as usize {
            let input = InterleavedPcm(&pcm_buffer[..chunk_size * channels as usize]);
            let mut mp3_buffer: Vec<MaybeUninit<u8>> =
                vec![MaybeUninit::uninit(); chunk_size * 5 / 4 + 7200];
            let encoded_size = encoder.encode(input, &mut mp3_buffer).expect("encode");
            let encoded_data: &[u8] = unsafe {
                std::slice::from_raw_parts(mp3_buffer.as_ptr() as *const u8, encoded_size)
            };
            std::io::Write::write_all(&mut output, encoded_data)?;
            pcm_buffer.drain(..chunk_size * channels as usize);
        }

        // Progress indicator every second
        if i % sample_rate as u64 == 0 && i > 0 {
            eprint!(
                "\r→ Progress: {}/{} seconds",
                i / sample_rate as u64,
                duration
            );
        }
    }

    // Encode remaining samples
    if !pcm_buffer.is_empty() {
        let input = InterleavedPcm(&pcm_buffer);
        let mut mp3_buffer: Vec<MaybeUninit<u8>> =
            vec![MaybeUninit::uninit(); pcm_buffer.len() * 5 / 4 + 7200];
        let encoded_size = encoder.encode(input, &mut mp3_buffer).expect("encode");
        let encoded_data: &[u8] =
            unsafe { std::slice::from_raw_parts(mp3_buffer.as_ptr() as *const u8, encoded_size) };
        std::io::Write::write_all(&mut output, encoded_data)?;
    }

    // Flush encoder
    let mut mp3_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); 7200];
    let encoded_size = encoder.flush::<FlushNoGap>(&mut mp3_buffer).expect("flush");
    let encoded_data: &[u8] =
        unsafe { std::slice::from_raw_parts(mp3_buffer.as_ptr() as *const u8, encoded_size) };
    std::io::Write::write_all(&mut output, encoded_data)?;

    eprintln!("\r→ Progress: {}/{} seconds", duration, duration);
    eprintln!("✓ Saved to {}", filename);

    Ok(())
}
