//! Audio playback functionality.

use crate::generators::{
    create_configured_generator, Generator, GeneratorConfig, NoiseKind, SpatialMode,
};
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Play audio through the default output device.
pub fn play_audio(kind: NoiseKind, gain: f32, binaural: bool, duration: Option<u64>) -> Result<()> {
    let mut config = GeneratorConfig::new(kind, gain);
    if binaural {
        config.spatial.mode = SpatialMode::Binaural;
    }
    play_audio_with_config(&config, duration)
}

pub fn play_audio_with_config(settings: &GeneratorConfig, duration: Option<u64>) -> Result<()> {
    settings.validate(48000.0)?;
    // Select default host/device/config
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No default output device");
    let config = device
        .default_output_config()
        .expect("Failed to get default output config");

    eprintln!("→ Device: {}", device.name()?);
    eprintln!(
        "→ Config: {:?} {}ch @ {} Hz",
        config.sample_format(),
        config.channels(),
        config.sample_rate().0
    );
    eprintln!("→ Noise: {:?}, gain: {}", settings.kind, settings.gain);

    // Build the generator state matched to channels & sample rate
    let sample_rate = config.sample_rate().0 as f32;
    let generator = create_configured_generator(settings, sample_rate)?;

    // Build the stream for the negotiated sample format
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), generator)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), generator)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), generator)?,
        _ => panic!("Unsupported sample format"),
    };

    stream.play()?;

    // Block until duration elapses or Ctrl-C
    if let Some(secs) = duration {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    } else {
        ctrlc::set_handler(|| std::process::exit(0)).ok();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut generator: Box<dyn Generator>,
) -> Result<cpal::Stream>
where
    T: cpal::Sample + cpal::FromSample<f32> + cpal::SizedSample,
{
    let channels = config.channels as usize;

    let err_fn = |e| eprintln!("Stream error: {e}");
    let mut frame = vec![0.0f32; channels];

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            // Fill interleaved buffer
            for frame_samples in data.chunks_mut(channels) {
                generator.next_frame(&mut frame);
                for (i, s) in frame_samples.iter_mut().enumerate() {
                    *s = T::from_sample(frame[i]);
                }
            }
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}
