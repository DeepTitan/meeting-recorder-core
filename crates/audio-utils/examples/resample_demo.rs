//! End-to-end proof that the foundational crates work.
//!
//! 1. Synthesizes 2 seconds of a 440 Hz sine wave at 48 kHz.
//! 2. Writes it to `/tmp/mrc_demo_48k.wav` via `hound`.
//! 3. Reads it back through `hypr_audio_utils::source_from_path` (uses rodio).
//! 4. Downsamples to 16 kHz with `hypr_audio_utils::resample_audio`
//!    (which internally drives rubato via the types our `resampler` crate
//!    re-exports).
//! 5. Writes the 16 kHz output to `/tmp/mrc_demo_16k.wav`.
//! 6. Prints sample counts + durations + file sizes so you can verify.
//!
//! Run: `cargo run --example resample_demo -p audio-utils`

use std::f32::consts::TAU;
use std::fs;
use std::path::Path;

use hound::{SampleFormat, WavSpec, WavWriter};
// The crate is aliased as hypr-audio-utils for downstream consumers,
// but from inside its own examples directory we use the raw name.
use audio_utils::{resample_audio, source_from_path};

const INPUT_RATE: u32 = 48_000;
const OUTPUT_RATE: u32 = 16_000;
const DURATION_SECS: u32 = 2;
const TONE_HZ: f32 = 440.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new("/tmp/mrc_demo_48k.wav");
    let output_path = Path::new("/tmp/mrc_demo_16k.wav");

    // --- 1. Synthesize + write the 48 kHz source --------------------------
    let frames_48k = (INPUT_RATE * DURATION_SECS) as usize;
    let samples_48k: Vec<f32> = (0..frames_48k)
        .map(|i| {
            let t = i as f32 / INPUT_RATE as f32;
            (TAU * TONE_HZ * t).sin() * 0.6
        })
        .collect();

    write_wav_f32(input_path, INPUT_RATE, &samples_48k)?;

    // --- 2. Read it back via our copied pipeline --------------------------
    let source = source_from_path(input_path)?;

    // --- 3. Resample 48 kHz → 16 kHz --------------------------------------
    let samples_16k: Vec<f32> = resample_audio(source, OUTPUT_RATE)?;

    // --- 4. Write the resampled output ------------------------------------
    write_wav_f32(output_path, OUTPUT_RATE, &samples_16k)?;

    // --- 5. Report --------------------------------------------------------
    let expected_16k_frames = (OUTPUT_RATE * DURATION_SECS) as usize;
    let actual_16k_frames = samples_16k.len();
    let ratio = actual_16k_frames as f64 / frames_48k as f64;

    println!("meeting-recorder-core / resample demo");
    println!("-------------------------------------");
    println!("source  : {:>10} frames @ {} Hz  ({}s tone @ {} Hz)",
             frames_48k, INPUT_RATE, DURATION_SECS, TONE_HZ as u32);
    println!("resampled: {:>10} frames @ {} Hz  (expected ~{} frames)",
             actual_16k_frames, OUTPUT_RATE, expected_16k_frames);
    println!("ratio   :  {:.6}  (should be ~{:.6})",
             ratio, OUTPUT_RATE as f64 / INPUT_RATE as f64);

    let meta_48k = fs::metadata(input_path)?;
    let meta_16k = fs::metadata(output_path)?;
    println!();
    println!("48k wav : {} ({} bytes)", input_path.display(), meta_48k.len());
    println!("16k wav : {} ({} bytes)", output_path.display(), meta_16k.len());
    println!();
    println!("listen  : afplay {}", output_path.display());
    println!();

    // --- 6. Hard assertion so the exit code reflects correctness ---------
    let tolerance = 0.01; // within 1% of expected
    let low = (expected_16k_frames as f64) * (1.0 - tolerance);
    let high = (expected_16k_frames as f64) * (1.0 + tolerance);
    assert!(
        (low..=high).contains(&(actual_16k_frames as f64)),
        "resample frame count {} outside expected range {:.0}..{:.0}",
        actual_16k_frames, low, high,
    );
    let peak = samples_16k
        .iter()
        .fold(0.0_f32, |acc, &s| acc.max(s.abs()));
    assert!(peak > 0.4 && peak < 1.0, "peak amplitude {} looks wrong", peak);

    println!("OK  — frame-count within 1% of expected, peak amplitude {:.3}", peak);
    Ok(())
}

fn write_wav_f32(
    path: &Path,
    sample_rate: u32,
    samples: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}
