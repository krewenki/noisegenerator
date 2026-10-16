use clap::ValueEnum;
use noisegenerator::generators::{
    create_configured_generator, GeneratorConfig, LayerConfig, NoiseKind, SpatialMode,
};

#[test]
fn every_sound_is_reproducible_without_locking_stereo_channels_together() {
    for &kind in NoiseKind::value_variants() {
        let mut config = GeneratorConfig::new(kind, 0.2);
        config.seed = Some(42);
        config.spatial.mode = SpatialMode::Wide;
        let mut a = create_configured_generator(&config, 48000.0).unwrap();
        let mut b = create_configured_generator(&config, 48000.0).unwrap();
        config.seed = Some(43);
        let mut c = create_configured_generator(&config, 48000.0).unwrap();
        let mut channels_differ = false;
        let mut seeds_differ = false;
        for _ in 0..4096 {
            let mut fa = [0.0; 2];
            let mut fb = fa;
            let mut fc = fa;
            a.next_frame(&mut fa);
            b.next_frame(&mut fb);
            c.next_frame(&mut fc);
            assert_eq!(fa, fb, "seed mismatch for {kind:?}");
            assert!(
                fa.iter().all(|s| s.is_finite() && s.abs() <= 0.2),
                "{kind:?}"
            );
            channels_differ |= fa[0] != fa[1];
            seeds_differ |= fa != fc;
        }
        assert!(
            channels_differ && seeds_differ,
            "independence failed for {kind:?}"
        );
    }
}
#[test]
fn layers_are_reproducible_and_hard_pan_is_respected() {
    let mut config = GeneratorConfig::new(NoiseKind::White, 0.5);
    config.seed = Some(17);
    config.layers = vec!["pink:1:-1".parse().unwrap()];
    let mut mixed = create_configured_generator(&config, 48000.0).unwrap();
    config.layers.clear();
    let mut base = create_configured_generator(&config, 48000.0).unwrap();
    let mut changed_left = false;
    for _ in 0..10000 {
        let mut a = [0.0; 2];
        let mut b = a;
        mixed.next_frame(&mut a);
        base.next_frame(&mut b);
        assert!((a[1] - b[1] * 0.5).abs() < 1e-6);
        changed_left |= (a[0] - b[0] * 0.5).abs() > 1e-5;
        assert!(a.iter().all(|s| s.abs() <= 0.5));
    }
    assert!(changed_left);
}
#[test]
fn zero_density_removes_events_and_max_density_stays_bounded() {
    for kind in [NoiseKind::Velvet, NoiseKind::Rain, NoiseKind::Fire] {
        let mut config = GeneratorConfig::new(kind, 0.8);
        config.seed = Some(3);
        config.sound.density = Some(0.0);
        config.sound.event_level = 1.0;
        let mut silent = create_configured_generator(&config, 48000.0).unwrap();
        for _ in 0..1000 {
            let mut frame = [1.0; 2];
            silent.next_frame(&mut frame);
            assert_eq!(frame, [0.0; 2]);
        }
        config.sound.density = Some(48000.0);
        let mut dense = create_configured_generator(&config, 48000.0).unwrap();
        let mut power = 0.0;
        for _ in 0..48000 {
            let mut frame = [0.0; 2];
            dense.next_frame(&mut frame);
            assert!(frame.iter().all(|s| s.is_finite() && s.abs() <= 0.8));
            power += frame[0] * frame[0];
        }
        assert!(power > 0.01);
    }
}
#[test]
fn layer_and_sound_controls_are_validated() {
    for invalid in [
        "pink",
        "pink:1",
        "purple:1:0",
        "pink:NaN:0",
        "pink:1:2",
        "pink:-1:0",
    ] {
        assert!(invalid.parse::<LayerConfig>().is_err(), "{invalid}");
    }
    let mut config = GeneratorConfig::new(NoiseKind::Fan, 0.2);
    config.sound.tone_hz = 10000.0;
    assert!(config.validate(48000.0).is_err());
    config.sound.tone_hz = 110.0;
    config.sound.perlin_octaves = 11;
    assert!(config.validate(48000.0).is_err());
    config.sound.perlin_octaves = 6;
    config.sound.density = Some(f32::NAN);
    assert!(config.validate(48000.0).is_err());
    config.sound.density = None;
    config.layers = vec!["pink:0.5:0".parse().unwrap(); 17];
    assert!(config.validate(48000.0).is_err());
}
