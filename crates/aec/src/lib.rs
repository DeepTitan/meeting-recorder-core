mod error;
pub use error::*;

#[cfg(feature = "onnx")]
mod onnx;
#[cfg(feature = "onnx")]
pub use onnx::AEC;
#[cfg(feature = "onnx")]
pub use onnx::model::{BLOCK_SHIFT, BLOCK_SIZE};

/// Passthrough acoustic echo cancellation.
///
/// Upstream Hyprnote provides a neural AEC gated behind the `onnx` feature.
/// We ship this stub when that feature is off so callers can keep the same
/// API surface — microphone samples pass through untouched.
#[cfg(not(feature = "onnx"))]
pub struct AEC;

#[cfg(not(feature = "onnx"))]
impl AEC {
    pub fn new() -> Result<Self, Error> {
        Ok(AEC)
    }

    /// Returns the mic buffer unchanged. A real AEC would subtract the
    /// loopback signal here using the speaker reference.
    pub fn process_streaming(
        &mut self,
        mic: &[f32],
        _speaker: &[f32],
    ) -> Result<Vec<f32>, Error> {
        Ok(mic.to_vec())
    }
}

pub(crate) struct CircularBuffer {
    buffer: Vec<f32>,
    block_len: usize,
    block_shift: usize,
}

impl CircularBuffer {
    fn new(block_len: usize, block_shift: usize) -> Self {
        Self {
            buffer: vec![0.0f32; block_len],
            block_len,
            block_shift,
        }
    }

    fn push_chunk(&mut self, chunk: &[f32]) {
        let keep = self.block_len - self.block_shift;
        self.buffer.copy_within(self.block_shift.., 0);
        let copy_len = chunk.len().min(self.block_shift);
        self.buffer[keep..keep + copy_len].copy_from_slice(&chunk[..copy_len]);

        if copy_len < self.block_shift {
            self.buffer[keep + copy_len..].fill(0.0);
        }
    }

    fn shift_and_accumulate(&mut self, data: &[f32]) {
        let keep = self.block_len - self.block_shift;
        self.buffer.copy_within(self.block_shift.., 0);
        self.buffer[keep..].fill(0.0);

        for (d, &val) in self.buffer.iter_mut().zip(data.iter()) {
            *d += val;
        }
    }

    fn data(&self) -> &[f32] {
        &self.buffer
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}
