//! USB Audio Class 1.0 - Speaker device
//!
//! Provides a class with a single audio streaming interface (host to device),
//! that advertises itself as a speaker. Includes explicit sample rate feedback.
//!
//! Various aspects of the audio stream can be configured, for example:
//! - sample rate
//! - sample resolution
//! - audio channel count and assignment
//!
//! The class provides volume and mute controls for each channel.

pub use super::Volume;
use super::function::{AudioFunction, FeatureUnitControls};
pub use super::function::{ControlMonitor, Feedback, State};
use super::terminal_type::TerminalType;
use super::{Channel, FeedbackRefresh, SampleWidth};
use crate::Builder;
use crate::driver::{Driver, Endpoint, EndpointError, EndpointOut, EndpointType};

/// Implementation of the USB audio class 1.0.
pub struct Speaker<'d, D: Driver<'d>> {
    /// Stream
    pub stream: Stream<'d, D>,
    /// Feedback
    pub feedback: Feedback<'d, D>,
    /// Control Monitor
    pub control_monitor: ControlMonitor<'d>,
}

impl<'d, D: Driver<'d>> Speaker<'d, D> {
    /// Creates a new [`Speaker`] device, split into a stream, feedback, and a control change notifier.
    ///
    /// The packet size should be chosen, based on the expected transfer size of samples per (micro)frame.
    /// For example, a stereo stream at 32 bit resolution and 48 kHz sample rate yields packets of 384 byte for
    /// full-speed USB (1 ms frame interval) or 48 byte for high-speed USB (125 us microframe interval).
    /// When using feedback, the packet size varies and thus, the `max_packet_size` should be increased (e.g. to double).
    ///
    /// # Arguments
    ///
    /// * `builder` - The builder for the class.
    /// * `state` - The internal state of the class.
    /// * `max_packet_size` - The maximum packet size per (micro)frame.
    /// * `resolution` - The audio sample resolution.
    /// * `sample_rates_hz` - The supported sample rates in Hz (at most ten).
    /// * `channels` - The advertised audio channels (up to 12), in stream order.
    /// * `feedback_refresh_period` - The refresh period for the feedback value.
    ///
    /// # Panics
    ///
    /// If there is no sample rate or more than ten, no channel or more than
    /// twelve, channels duplicated or out of `wChannelConfig` bit order, a
    /// sample rate above 24 bits, `max_packet_size` above the 1024-byte
    /// isochronous limit (1023 at full speed), or a too-small `control_buf`.
    pub fn new(
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d>,
        max_packet_size: u16,
        resolution: SampleWidth,
        sample_rates_hz: &'d [u32],
        channels: &'d [Channel],
        feedback_refresh_period: FeedbackRefresh,
    ) -> Self {
        // Terminal topology:
        // Input terminal (receives audio stream) -> Feature Unit (mute and volume) -> Output terminal (e.g. towards speaker)
        let function = AudioFunction {
            channels,
            sample_width: resolution,
            sample_rates_hz,
            input_terminal: TerminalType::UsbStreaming,
            output_terminal: TerminalType::OutSpeaker,
            // Mute and volume control on every channel; the master channel has none of its own.
            feature_unit: Some((FeatureUnitControls::NONE, FeatureUnitControls::ALL)),
        };

        let (streaming_endpoint, feedback, control_monitor) = function.build(
            builder,
            state,
            max_packet_size,
            Some(feedback_refresh_period),
            |alt, max_packet_size| alt.alloc_endpoint_out(EndpointType::Isochronous, None, max_packet_size, 1),
        );

        Self {
            stream: Stream { streaming_endpoint },
            feedback: feedback.unwrap(),
            control_monitor,
        }
    }
}

/// Used for reading audio frames.
pub struct Stream<'d, D: Driver<'d>> {
    streaming_endpoint: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Stream<'d, D> {
    /// Reads a single packet from the OUT endpoint
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.streaming_endpoint.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.streaming_endpoint.wait_enabled().await;
    }
}
