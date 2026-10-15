use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "noisegenerator-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn all_spatial_modes_export_wav_and_mp3() {
    for mode in ["mono", "wide", "binaural", "trinaural", "drift"] {
        for format in ["wav", "mp3"] {
            let dir = Temp::new();
            let output = Command::new(env!("CARGO_BIN_EXE_noisegenerator"))
                .current_dir(&dir.0)
                .args([
                    "pink",
                    "--mode",
                    mode,
                    "--save",
                    "--seconds",
                    "1",
                    "--format",
                    format,
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let path = fs::read_dir(&dir.0)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert!(fs::metadata(&path).unwrap().len() > 1000);
            if format == "wav" {
                let mut reader = hound::WavReader::open(path).unwrap();
                assert_eq!(reader.spec().channels, 2);
                assert_eq!(reader.spec().sample_rate, 48000);
                assert_eq!(reader.duration(), 48000);
                let samples: Vec<_> = reader.samples::<i16>().map(Result::unwrap).collect();
                assert!(samples.iter().any(|s| *s != 0));
                if mode == "mono" {
                    assert!(samples.chunks_exact(2).all(|s| s[0] == s[1]));
                } else {
                    assert!(samples.chunks_exact(2).any(|s| s[0] != s[1]));
                }
                assert!(samples.iter().all(|s| s.abs() < i16::MAX));
            }
        }
    }
}

#[test]
fn invalid_options_fail_before_creating_files() {
    for args in [
        vec!["--mode", "wide", "--binaural"],
        vec!["--width", "2"],
        vec!["--motion-rate", "NaN"],
    ] {
        let dir = Temp::new();
        let output = Command::new(env!("CARGO_BIN_EXE_noisegenerator"))
            .current_dir(&dir.0)
            .args(["--save", "--seconds", "1"])
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
    }
}

#[test]
fn textured_exports_include_effects_and_timed_fades() {
    for kind in ["ocean", "wind"] {
        for format in ["wav", "mp3"] {
            let dir = Temp::new();
            let output = Command::new(env!("CARGO_BIN_EXE_noisegenerator"))
                .current_dir(&dir.0)
                .args([
                    kind,
                    "--mode",
                    "wide",
                    "--lowpass",
                    "3000",
                    "--highpass",
                    "50",
                    "--breath-depth",
                    "0.3",
                    "--fade-in",
                    "0.1",
                    "--fade-out",
                    "0.1",
                    "--save",
                    "--seconds",
                    "1",
                    "--format",
                    format,
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let path = fs::read_dir(&dir.0)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            if format == "wav" {
                let mut reader = hound::WavReader::open(path).unwrap();
                let samples: Vec<_> = reader.samples::<i16>().map(Result::unwrap).collect();
                assert_eq!(&samples[..2], &[0, 0]);
                assert_eq!(&samples[samples.len() - 2..], &[0, 0]);
                assert!(samples.iter().any(|s| *s != 0));
            }
        }
    }
}

#[test]
fn layered_event_and_tonal_sounds_export_in_both_formats() {
    for kind in ["rain", "fire", "fan", "drone"] {
        for format in ["wav", "mp3"] {
            let dir = Temp::new();
            let output = Command::new(env!("CARGO_BIN_EXE_noisegenerator"))
                .current_dir(&dir.0)
                .args([
                    kind,
                    "--mode",
                    "trinaural",
                    "--layer",
                    "brown:0.5:-0.6",
                    "--layer",
                    "velvet:0.1:0.7",
                    "--seed",
                    "42",
                    "--density",
                    "30",
                    "--save",
                    "--seconds",
                    "1",
                    "--format",
                    format,
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let path = fs::read_dir(&dir.0)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert!(fs::metadata(&path).unwrap().len() > 1000);
        }
    }
}

#[test]
fn seeded_wav_samples_are_identical_across_runs() {
    let mut previous = None;
    for _ in 0..2 {
        let dir = Temp::new();
        let output = Command::new(env!("CARGO_BIN_EXE_noisegenerator"))
            .current_dir(&dir.0)
            .args([
                "rain",
                "--mode",
                "drift",
                "--layer",
                "drone:0.1:0.5",
                "--seed",
                "42",
                "--save",
                "--seconds",
                "1",
                "--format",
                "wav",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let path = fs::read_dir(&dir.0)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let bytes = fs::read(path).unwrap();
        if let Some(expected) = &previous {
            assert_eq!(expected, &bytes);
        }
        previous = Some(bytes);
    }
}
