use clap::ValueEnum;
use noisegenerator::generators::{
    create_configured_generator, GeneratorConfig, NoiseKind, SpatialMode,
};

#[test]
fn depth_is_reproducible_and_bounded_for_every_sound() {
    for &kind in NoiseKind::value_variants() {
        for sr in [44100.0, 48000.0, 96000.0] {
            let mut config = GeneratorConfig::new(kind, 0.2);
            config.seed = Some(123);
            config.spatial.mode = SpatialMode::Depth;
            config.layers = vec![
                "pink:0.3:-0.7".parse().unwrap(),
                "drone:0.1:0.6".parse().unwrap(),
            ];
            let mut a = create_configured_generator(&config, sr).unwrap();
            let mut b = create_configured_generator(&config, sr).unwrap();
            for _ in 0..4096 {
                let mut fa = [0.0; 2];
                let mut fb = fa;
                a.next_frame(&mut fa);
                b.next_frame(&mut fb);
                assert_eq!(fa, fb);
                assert!(fa.iter().all(|s| s.is_finite() && s.abs() <= 0.2));
            }
        }
    }
}
#[test]
fn centered_depth_matches_centered_three_source_mix_and_mono_downmix() {
    let mut config = GeneratorConfig::new(NoiseKind::Pink, 0.2);
    config.seed = Some(1);
    config.spatial.mode = SpatialMode::Depth;
    config.spatial.motion_depth = 0.0;
    let mut depth = create_configured_generator(&config, 48000.0).unwrap();
    let mut mono = create_configured_generator(&config, 48000.0).unwrap();
    config.spatial.mode = SpatialMode::Trinaural;
    let mut pan = create_configured_generator(&config, 48000.0).unwrap();
    for _ in 0..48000 {
        let mut a = [0.0; 2];
        let mut b = a;
        let mut m = [0.0];
        depth.next_frame(&mut a);
        pan.next_frame(&mut b);
        mono.next_frame(&mut m);
        for i in 0..2 {
            assert!((a[i] - b[i]).abs() < 1e-7);
        }
        assert!((m[0] - (a[0] + a[1]) * 0.5).abs() < 1e-7);
    }
}
#[test]
fn invalid_depth_parameters_are_rejected() {
    let mut config = GeneratorConfig::new(NoiseKind::White, 0.2);
    config.spatial.mode = SpatialMode::Depth;
    assert!(create_configured_generator(&config, 1000.0).is_err());
    config.spatial.headphone_strength = f32::NAN;
    assert!(create_configured_generator(&config, 48000.0).is_err());
}
