# meeting-recorder-core

Rust library for real-time meeting audio capture and streaming
speech-to-text on macOS. Forked from [Hyprnote / char](https://github.com/fastrepl/char).

Designed to be consumed as a Cargo dependency by desktop apps that need:

- **Mic + system-audio capture** on macOS via Core Audio process taps
  (separate channels, never mixed).
- **Low-latency streaming transcription** (Cactus on Apple Silicon,
  whisper.cpp fallback).
- **Delta-merged transcript events** (`interim` → `final`) suitable for
  driving a live transcript UI.

## Status

Early fork. The scaffold and base audio crates are in place. Audio
capture, streaming STT, and the listener pipeline land in subsequent
commits.

## Layout

```
crates/
├── audio                 # core traits, error types, mic/speaker input enum
├── audio-interface       # IO traits + source abstractions (rodio adapters)
├── audio-mime            # mime/format helpers
├── audio-sync            # cross-channel drift correction via FFT
├── audio-utils           # resample + chunk helpers
├── resampler             # rubato-based sample-rate conversion
│ …  (more crates land per commit)
```

## License

GPL-3.0-or-later. See `LICENSE` and `NOTICE` for attribution.
