//! Records 5 seconds from the default microphone and writes the result to
//! `/tmp/mrc_capture.wav`. First run will trigger the macOS Microphone TCC
//! prompt — approve it in System Settings → Privacy & Security → Microphone
//! if you miss the popup, then rerun.
//!
//! Run: `cargo run --example capture_demo -p audio-actual`

use std::path::Path;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use hound::{SampleFormat, WavSpec, WavWriter};

use audio_actual::AudioInput;

const SAMPLE_RATE: u32 = 16_000;
const CHUNK_SIZE: usize = 512;
const CAPTURE_SECS: u64 = 5;
const OUTPUT_PATH: &str = "/tmp/mrc_capture.wav";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("meeting-recorder-core / mic capture demo");
    println!("----------------------------------------");

    let default_name = AudioInput::get_default_device_name();
    println!("device : {default_name}");
    println!("format : {SAMPLE_RATE} Hz, mono, f32");
    println!("length : {CAPTURE_SECS} seconds");
    println!();
    println!("speak now…");

    let mut stream = AudioInput::from_mic_capture(None, SAMPLE_RATE, CHUNK_SIZE)?;

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(OUTPUT_PATH, spec)?;

    let deadline = Instant::now() + Duration::from_secs(CAPTURE_SECS);
    let mut frames_written: usize = 0;
    let mut peak: f32 = 0.0;
    let mut last_report = Instant::now();

    while Instant::now() < deadline {
        let timeout = deadline.saturating_duration_since(Instant::now());
        let next = tokio::time::timeout(timeout, stream.next()).await;
        match next {
            Ok(Some(Ok(frame))) => {
                for &s in frame.raw_mic.iter() {
                    writer.write_sample(s)?;
                    peak = peak.max(s.abs());
                    frames_written += 1;
                }
                if last_report.elapsed() >= Duration::from_millis(500) {
                    let elapsed = CAPTURE_SECS as f32
                        - deadline.saturating_duration_since(Instant::now()).as_secs_f32();
                    println!(
                        "  t={elapsed:>4.1}s  frames={frames_written:>7}  peak={peak:.3}"
                    );
                    last_report = Instant::now();
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("capture error: {e:?}");
                break;
            }
            Ok(None) => {
                eprintln!("stream ended early");
                break;
            }
            Err(_) => break, // timeout = deadline reached
        }
    }

    writer.finalize()?;

    let expected = (SAMPLE_RATE as usize) * (CAPTURE_SECS as usize);
    let captured_secs = frames_written as f32 / SAMPLE_RATE as f32;
    println!();
    println!("file   : {OUTPUT_PATH}");
    println!("frames : {frames_written} (expected ~{expected})");
    println!("seconds: {captured_secs:.2}");
    println!("peak   : {peak:.3}   (needs to be > 0.001 to confirm real signal)");
    println!();

    if peak < 0.001 {
        eprintln!("WARNING: peak amplitude is essentially silent. Either your mic");
        eprintln!("         is muted/denied TCC, or nothing was making sound.");
        eprintln!("         Check System Settings → Privacy & Security → Microphone.");
    } else {
        println!("OK — got real audio. Listen:");
        println!("     afplay {}", Path::new(OUTPUT_PATH).display());
    }

    Ok(())
}
