//! USB Audio Class 1.0 - Audio source device (device to host), e.g. a microphone.
//!
//! Configured through [`Config`]; built and used like [`super::speaker::Speaker`].

use core::num::NonZeroU32;

use embassy_usb_driver::EndpointInterval;

pub use super::Volume;
use super::function::AudioFunction;
pub use super::function::{ControlMonitor, FeatureUnitControls, State};
use super::{Channel, MAX_AUDIO_CHANNEL_COUNT, SampleWidth};
use crate::class::uac::terminal_type::TerminalType;
use crate::driver::{Driver, Endpoint, EndpointDisabledError, EndpointType, IsoEndpointIn, IsoWriteError};
use crate::{Builder, EndpointOptions};

/// Everything the audio source advertises to the host.
///
/// The stream has no synch (feedback) endpoint: an asynchronous source has
/// none [USB 2.0 5.12.4.2] — the data stream itself carries the rate, and the
/// host follows it.
#[derive(Clone)]
pub struct Config<'d> {
    /// The supported sample rates in Hz, as discrete values. At least one, at
    /// most ten. The first is the one reported before the host sets any.
    pub sample_rates_hz: &'d [u32],
    /// The audio sample resolution.
    pub sample_width: SampleWidth,
    /// The audio channels, in stream order (up to 12). The host numbers the
    /// logical channels by `wChannelConfig` bit order, so the entries must be
    /// unique and in that order. One entry is a mono stream.
    pub channels: &'d [Channel],
    /// What the Input Terminal is; the host shows the device by it. Must not
    /// be [`TerminalType::UsbStreaming`] — that is the Output Terminal's role.
    pub input_terminal: TerminalType,
    /// Controls the USB host can change on each channel.
    ///
    /// Use `heapless::Vec::new()` to omit the Feature Unit. Otherwise, provide
    /// `channels.len() + 1` entries: the master first, then each audio channel in
    /// [`Self::channels`] order. Use [`FeatureUnitControls::empty`] for no controls
    /// on a channel.
    pub feature_unit: heapless::Vec<FeatureUnitControls, MAX_AUDIO_CHANNEL_COUNT, u8>,
}

impl<'d> Config<'d> {
    /// Creates a microphone configuration without a Feature Unit.
    pub const fn new(sample_rates_hz: &'d [u32], sample_width: SampleWidth, channels: &'d [Channel]) -> Self {
        Self {
            sample_rates_hz,
            sample_width,
            channels,
            input_terminal: TerminalType::InMicrophone,
            feature_unit: heapless::Vec::new(),
        }
    }
}

/// Implementation of the USB audio class 1.0, source side.
pub struct AudioSource<'d, D: Driver<'d>> {
    /// The audio stream toward the host.
    pub stream: Stream<'d, D>,
    /// Control Monitor
    pub control_monitor: ControlMonitor<'d>,
}

impl<'d, D: Driver<'d>> AudioSource<'d, D> {
    /// Creates a new [`AudioSource`] device, and registers its control handler.
    ///
    /// The streaming endpoint's packet size is one full-speed frame (1 ms) of
    /// samples at the highest sample rate, plus one frame of slack: the
    /// source's clock is its own, and it is allowed to get a sample per
    /// channel ahead of the host.
    ///
    /// # Panics
    ///
    /// If `config` has no sample rate or more than ten, no channel or more
    /// than twelve, channels duplicated or out of `wChannelConfig` bit order,
    /// a sample rate above 24 bits, a packet size above the 1023-byte
    /// full-speed limit, [`TerminalType::UsbStreaming`] as the input
    /// terminal, a nonempty `feature_unit` without exactly one entry for the
    /// master and each audio channel, unsupported control bits, or a too-small
    /// `control_buf`.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let Config {
            sample_rates_hz,
            sample_width,
            channels,
            input_terminal,
            feature_unit,
        } = config;

        // Terminal topology:
        // Input terminal (e.g. a microphone) -> [Feature Unit (mute and volume)] -> Output terminal (USB stream)
        let function = AudioFunction {
            channels,
            sample_width,
            sample_rates_hz,
            input_terminal,
            output_terminal: TerminalType::UsbStreaming,
            feature_unit,
        };

        // One millisecond of samples at the fastest rate, plus one frame of
        // slack, computed in u64 so no sample rate can overflow it. The source
        // always declares 1 ms packets — full-speed framing, where isochronous
        // payloads are at most 1023 bytes [USB 2.0 5.6.3].
        let frame_bytes = channels.len() as u64 * sample_width as u64;
        let max_rate = sample_rates_hz.iter().copied().max().unwrap_or(0);
        let max_packet_size = (max_rate as u64 * frame_bytes).div_ceil(1000) + frame_bytes;
        assert!(
            max_packet_size <= 1023,
            "at most 1023 bytes per full-speed isochronous packet"
        );

        let (streaming_endpoint, _, control_monitor) =
            function.build(builder, state, max_packet_size as u16, None, |alt, max_packet_size| {
                alt.alloc_endpoint_in(
                    EndpointType::Isochronous,
                    None,
                    max_packet_size,
                    EndpointInterval::from_ms(1),
                    EndpointOptions::new(),
                )
            });

        Self {
            stream: Stream { streaming_endpoint },
            control_monitor,
        }
    }
}

/// Used for writing audio frames.
pub struct Stream<'d, D: Driver<'d>> {
    streaming_endpoint: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Stream<'d, D> {
    /// Wait until the next packet can be queued, returning a reserved slot.
    ///
    /// [`IsoInSlot::skipped`](crate::driver::IsoInSlot::skipped) reports how many intervals this
    /// writer left empty before this slot.
    pub async fn ready(&mut self) -> Result<<D::EndpointIn as IsoEndpointIn>::Slot<'_>, EndpointDisabledError>
    where
        D::EndpointIn: IsoEndpointIn,
    {
        self.streaming_endpoint.ready().await
    }

    /// Writes one packet for the next service interval.
    ///
    /// Returns how many intervals were skipped before this packet, if any. When the packet
    /// contents depend on timing or skipped intervals, call [`ready`](Self::ready) first, build
    /// the packet, and submit it with [`IsoInSlot::write`](crate::driver::IsoInSlot::write).
    /// A packet the host does not poll for is dropped rather than sent late.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<Option<NonZeroU32>, IsoWriteError>
    where
        D::EndpointIn: IsoEndpointIn,
    {
        crate::class::write_iso_packet(&mut self.streaming_endpoint, data).await
    }

    /// Waits for the USB host to enable this interface: it is listening.
    pub async fn wait_connection(&mut self) {
        self.streaming_endpoint.wait_enabled().await;
    }
}
