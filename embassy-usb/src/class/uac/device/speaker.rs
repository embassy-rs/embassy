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
//! Mute and volume controls are optional and configured through [`Config::feature_unit`].

use embassy_usb_driver::EndpointInterval;

pub use super::Volume;
use super::function::AudioFunction;
pub use super::function::{ControlMonitor, FeatureUnitControls, State};
use super::{Channel, FeedbackRefresh, MAX_AUDIO_CHANNEL_COUNT, SampleWidth};
use crate::class::uac::terminal_type::TerminalType;
use crate::driver::Driver;
use crate::{Builder, EndpointOptions};

/// Speaker stream settings and optional host controls.
#[derive(Clone)]
pub struct Config<'d> {
    /// Supported sample rates in Hz: one to ten discrete values.
    /// The first is reported before the host selects a rate.
    pub sample_rates_hz: &'d [u32],
    /// The audio sample resolution.
    pub sample_width: SampleWidth,
    /// Audio channels in stream order (one to twelve). Entries must be unique
    /// and follow `wChannelConfig` bit order, as listed in [`Channel`].
    pub channels: &'d [Channel],
    /// Controls the USB host can change on each channel.
    ///
    /// Use `heapless::Vec::new()` to omit the Feature Unit. Otherwise, provide
    /// `channels.len() + 1` entries: the master first, then each audio channel in
    /// [`Self::channels`] order. Use [`FeatureUnitControls::empty`] for no controls
    /// on a channel.
    pub feature_unit: heapless::Vec<FeatureUnitControls, MAX_AUDIO_CHANNEL_COUNT, u8>,
    /// Maximum bytes per USB packet. Allow room for sample rate variation,
    /// for example twice the bytes played per (micro)frame.
    pub max_packet_size: u16,
    /// How often the device sends sample rate feedback to the host.
    pub feedback_refresh_period: FeedbackRefresh,
}

impl<'d> Config<'d> {
    /// Creates a speaker configuration without a Feature Unit.
    pub const fn new(
        sample_rates_hz: &'d [u32],
        sample_width: SampleWidth,
        channels: &'d [Channel],
        max_packet_size: u16,
        feedback_refresh_period: FeedbackRefresh,
    ) -> Self {
        Self {
            sample_rates_hz,
            sample_width,
            channels,
            feature_unit: heapless::Vec::new(),
            max_packet_size,
            feedback_refresh_period,
        }
    }
}

/// Implementation of the USB audio class 1.0.
pub struct Speaker<'d, D: Driver<'d>> {
    /// Stream
    pub stream: D::IsoEndpointOut,
    /// Feedback
    pub feedback: D::IsoEndpointIn,
    /// Control Monitor
    pub control_monitor: ControlMonitor<'d>,
}

impl<'d, D: Driver<'d>> Speaker<'d, D> {
    /// Creates a new [`Speaker`] device, split into a stream, feedback, and a control change notifier.
    ///
    /// # Panics
    ///
    /// If there is no sample rate or more than ten, no channel or more than
    /// twelve, channels duplicated or out of `wChannelConfig` bit order, a
    /// sample rate above 24 bits, `max_packet_size` above the 1024-byte
    /// isochronous limit (1023 at full speed), a nonempty `feature_unit` without
    /// one entry for the master and each audio channel, unsupported control
    /// bits, or a too-small `control_buf`.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let Config {
            sample_rates_hz,
            sample_width,
            channels,
            feature_unit,
            max_packet_size,
            feedback_refresh_period,
        } = config;
        // Terminal topology:
        // Input terminal (USB stream) -> [Feature Unit] -> Output terminal (speaker)
        let function = AudioFunction {
            channels,
            sample_width,
            sample_rates_hz,
            input_terminal: TerminalType::UsbStreaming,
            output_terminal: TerminalType::OutSpeaker,
            feature_unit,
        };

        let (stream, feedback, control_monitor) = function.build(
            builder,
            state,
            max_packet_size,
            Some(feedback_refresh_period),
            |alt, max_packet_size| {
                alt.alloc_iso_endpoint_out(
                    None,
                    max_packet_size,
                    EndpointInterval::from_ms(1),
                    EndpointOptions::new(),
                )
            },
        );

        Self {
            stream,
            feedback: feedback.unwrap(),
            control_monitor,
        }
    }
}
