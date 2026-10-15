use noisegenerator::generators::{
    create_configured_generator, Binaural, Generator, GeneratorConfig, NoiseKind, Spatial,
    SpatialConfig, SpatialMode,
};

struct Constant(f32);
impl Generator for Constant {
    fn next_frame(&mut self, out: &mut [f32]) {
        out.fill(self.0);
    }
}
fn fixture(config: SpatialConfig) -> Spatial {
    Spatial::new(
        [
            Box::new(Constant(0.2)),
            Box::new(Constant(0.7)),
            Box::new(Constant(-0.4)),
        ],
        config,
        48000.0,
    )
}

#[test]
fn width_endpoints_and_mono_downmix() {
    for (width, expected) in [(0.0, [0.2, 0.2]), (1.0, [0.7, -0.4])] {
        let mut generator = fixture(SpatialConfig {
            mode: SpatialMode::Wide,
            width,
            ..Default::default()
        });
        let mut frame = [0.0; 2];
        generator.next_frame(&mut frame);
        assert_eq!(frame, expected);
        let mut mono = [0.0];
        generator.next_frame(&mut mono);
        assert!((mono[0] - (expected[0] + expected[1]) * 0.5).abs() < 1e-6);
        generator.next_frame(&mut []);
    }
}

#[test]
fn centered_motion_and_frozen_motion() {
    let mut centered = fixture(SpatialConfig {
        mode: SpatialMode::Trinaural,
        motion_depth: 0.0,
        ..Default::default()
    });
    let mut frozen = fixture(SpatialConfig {
        mode: SpatialMode::Drift,
        motion_rate: 0.0,
        ..Default::default()
    });
    let mut previous = [0.0; 2];
    frozen.next_frame(&mut previous);
    for _ in 0..1000 {
        let mut frame = [0.0; 2];
        centered.next_frame(&mut frame);
        assert_eq!(frame[0], frame[1]);
        frozen.next_frame(&mut frame);
        assert_eq!(previous, frame);
    }
}

#[test]
fn new_spatial_modes_are_peak_bounded_and_move() {
    for mode in [SpatialMode::Trinaural, SpatialMode::Drift] {
        let mut generator = fixture(SpatialConfig {
            mode,
            ..Default::default()
        });
        let mut initial = [0.0; 2];
        generator.next_frame(&mut initial);
        let mut frame = [0.0; 2];
        for _ in 0..480_000 {
            generator.next_frame(&mut frame);
            assert!(frame.iter().all(|s| s.is_finite() && s.abs() <= 0.7));
        }
        assert_ne!(initial, frame);
    }
}

#[test]
fn default_binaural_constructor_is_unchanged() {
    let make = || Box::new(Constant(0.2)) as Box<dyn Generator>;
    let mut old = Binaural::new(make(), make(), 48000.0);
    let mut explicit = Binaural::with_motion(make(), make(), 48000.0, 0.1, 1.0);
    for _ in 0..48000 {
        let mut a = [0.0; 2];
        let mut b = a;
        old.next_frame(&mut a);
        explicit.next_frame(&mut b);
        assert_eq!(a, b);
    }
}

#[test]
fn factory_rejects_invalid_controls() {
    let mut config = GeneratorConfig::new(NoiseKind::Pink, 0.2);
    for rate in [f32::NAN, -1.0, f32::INFINITY] {
        config.spatial.motion_rate = rate;
        assert!(create_configured_generator(&config, 48000.0).is_err());
    }
    config.spatial.motion_rate = 0.1;
    config.spatial.width = 1.1;
    assert!(config.validate(48000.0).is_err());
    config.spatial.width = 1.0;
    assert!(config.validate(0.0).is_err());
}
