//! Software epochs: the CPU-side tail of a network, running `embedded-nn` kernels.
//!
//! ST Edge AI does not emit a purely hardware network. The tail of a compiled
//! network is normally a run of *software* epochs — softmax, dequantization,
//! argmax — that the Cortex-M55 executes once the NPU's hardware epochs have
//! finished. [`EpochBlock`](super::EpochBlock) models that shape (`blob: None`
//! is a pure software epoch), but its `start`/`end` are bare `fn()` pointers and
//! so cannot carry the buffers a kernel needs. These types are therefore used
//! directly, alongside the hardware epochs rather than inside an `EpochBlock`.
//!
//! Each one is a thin wrapper over the equivalent [`embedded-nn`] kernel, so the
//! CPU half of a network runs the same integer arithmetic as the rest of that
//! runtime.
//!
//! ```rust,ignore
//! use embassy_stm32::npu::epoch::{ArgMaxEpoch, SoftmaxEpoch, SoftwareKernel};
//!
//! // Logits written by the NPU's hardware epoch.
//! let logits: &[i8] = /* ... */;
//!
//! let mut probabilities = [0i8; 5];
//! SoftmaxEpoch::new_1d(logits, &mut probabilities, mult, shift, diff_min).run().unwrap();
//!
//! let (mut class, mut score) = (0usize, 0i8);
//! ArgMaxEpoch::new(logits, &mut class, &mut score).run().unwrap();
//! ```
//!
//! Enabled by the `npu-nn` feature, which brings in the `embedded-nn` dependency.

use embedded_nn::softmax::softmax_s8;
use embedded_nn::support::dequantize_s8_to_f32;

/// Error returned by a software epoch.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum EpochError {
    /// The output buffer does not match the input, or is empty.
    DimensionMismatch,
    /// The underlying `embedded-nn` kernel rejected the call.
    Kernel,
}

/// A step of a network that runs on the Cortex-M55 rather than on the NPU.
pub trait SoftwareKernel {
    /// Runs the epoch.
    fn run(&mut self) -> Result<(), EpochError>;
}

/// Applies [`embedded-nn`]'s fixed-point softmax over a row of logits.
pub struct SoftmaxEpoch<'a> {
    input: &'a [i8],
    output: &'a mut [i8],
    mult: i32,
    shift: i32,
    diff_min: i32,
}

impl<'a> SoftmaxEpoch<'a> {
    /// Softmax over a single row of `input.len()` logits.
    ///
    /// `mult`, `shift` and `diff_min` are the fixed-point parameters ST's
    /// compiler emits next to the network (its `sw_info`); they must correspond
    /// to the quantization scale of `input`.
    pub fn new_1d(input: &'a [i8], output: &'a mut [i8], mult: i32, shift: i32, diff_min: i32) -> Self {
        Self {
            input,
            output,
            mult,
            shift,
            diff_min,
        }
    }
}

impl SoftwareKernel for SoftmaxEpoch<'_> {
    fn run(&mut self) -> Result<(), EpochError> {
        if self.input.len() != self.output.len() {
            return Err(EpochError::DimensionMismatch);
        }
        softmax_s8(
            self.input,
            1,
            self.input.len(),
            self.mult,
            self.shift,
            self.diff_min,
            self.output,
        )
        .map_err(|_| EpochError::Kernel)
    }
}

/// Dequantizes `int8` NPU output to `f32` using a scale and zero-point.
pub struct DequantizeEpoch<'a> {
    input: &'a [i8],
    output: &'a mut [f32],
    scale: f32,
    zero_point: i32,
}

impl<'a> DequantizeEpoch<'a> {
    /// Dequantizes each element of `input` into `output`.
    pub fn new(input: &'a [i8], output: &'a mut [f32], scale: f32, zero_point: i32) -> Self {
        Self {
            input,
            output,
            scale,
            zero_point,
        }
    }
}

impl SoftwareKernel for DequantizeEpoch<'_> {
    fn run(&mut self) -> Result<(), EpochError> {
        if self.input.len() != self.output.len() {
            return Err(EpochError::DimensionMismatch);
        }
        for (out, &value) in self.output.iter_mut().zip(self.input) {
            *out = dequantize_s8_to_f32(value, self.scale, self.zero_point);
        }
        Ok(())
    }
}

/// Finds the index and value of the largest element of a logit slice.
pub struct ArgMaxEpoch<'a> {
    logits: &'a [i8],
    best_class: &'a mut usize,
    best_score: &'a mut i8,
}

impl<'a> ArgMaxEpoch<'a> {
    /// Writes the index and value of the largest logit into `best_class` and
    /// `best_score`.
    pub fn new(logits: &'a [i8], best_class: &'a mut usize, best_score: &'a mut i8) -> Self {
        Self {
            logits,
            best_class,
            best_score,
        }
    }
}

impl SoftwareKernel for ArgMaxEpoch<'_> {
    fn run(&mut self) -> Result<(), EpochError> {
        let Some((&first, rest)) = self.logits.split_first() else {
            return Err(EpochError::DimensionMismatch);
        };
        let mut max_idx = 0;
        let mut max_val = first;
        for (i, &candidate) in rest.iter().enumerate() {
            if candidate > max_val {
                max_val = candidate;
                max_idx = i + 1;
            }
        }
        *self.best_class = max_idx;
        *self.best_score = max_val;
        Ok(())
    }
}
