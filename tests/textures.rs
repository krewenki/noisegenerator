use noisegenerator::generators::{
    create_configured_generator, create_configured_generator_for_duration, Biquad, GeneratorConfig,
    NoiseKind, SpatialMode,
};

fn response(mut filter: Biquad, hz: f32, sr: f32) -> f32 {
    let mut power = 0.0;
    for i in 0..48000 {
        let sample = filter.process((std::f32::consts::TAU * hz * i as f32 / sr).sin());
        assert!(sample.is_finite());
        if i >= 24000 {
            power += sample * sample;
        }
    }
    (power / 24000.0).sqrt()
}
#[test]
fn filters_attenuate_the_correct_bands_at_device_and_export_rates() {
    for sr in [44100.0, 48000.0, 96000.0] {
        assert!(response(Biquad::lowpass(1000.0, sr), 100.0, sr) > 0.65);
        assert!(response(Biquad::lowpass(1000.0, sr), 10000.0, sr) < 0.02);
        assert!(response(Biquad::highpass(1000.0, sr), 100.0, sr) < 0.02);
        assert!(response(Biquad::highpass(1000.0, sr), 10000.0, sr) > 0.65);
    }
}
#[test]
fn textures_stay_finite_and_non_silent_in_spatial_modes() {
    for kind in [NoiseKind::Ocean, NoiseKind::Wind] {
        for mode in [SpatialMode::Mono, SpatialMode::Wide, SpatialMode::Trinaural] {
            let mut config = GeneratorConfig::new(kind, 0.2);
            config.spatial.mode = mode;
            let mut generator = create_configured_generator(&config, 48000.0).unwrap();
            let mut power = 0.0;
            for _ in 0..48000 {
                let mut frame = [0.0; 2];
                generator.next_frame(&mut frame);
                for s in frame {
                    assert!(s.is_finite() && s.abs() <= 0.2);
                    power += s * s;
                }
            }
            assert!(power > 0.001);
        }
    }
}
#[test]
fn effect_controls_validate_before_generation() {
    let mut config = GeneratorConfig::new(NoiseKind::Pink, 0.2);
    config.effects.lowpass = Some(23000.0);
    assert!(config.validate(44100.0).is_err());
    assert!(config.validate(48000.0).is_ok());
    config.effects.lowpass = Some(100.0);
    config.effects.highpass = Some(200.0);
    assert!(config.validate(48000.0).is_err());
    config.effects.highpass = None;
    config.effects.fade_out = 1.0;
    assert!(create_configured_generator(&config, 48000.0).is_err());
    assert!(create_configured_generator_for_duration(&config, 48000.0, Some(2)).is_ok());
    config.effects.fade_in = f64::NAN;
    assert!(config.validate(48000.0).is_err());
}
