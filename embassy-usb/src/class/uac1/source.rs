//! USB Audio Class 1.0 - Audio source device (device to host), e.g. a microphone.
//!
//! Configured through [`Config`]; built and used like [`super::speaker::Speaker`].

pub use super::Volume;
use super::function::AudioFunction;
pub use super::function::{ControlMonitor, FeatureUnitControls, State};
use super::terminal_type::TerminalType;
use super::{Channel, SampleWidth};
use crate::Builder;
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointType};

/// Everything the audio source advertises to the host.
///
/// The stream has no synch (feedback) endpoint: an asynchronous source has
/// none [USB 2.0 5.12.4.2] — the data stream itself carries the rate, and the
/// host follows it.
#[derive(Clone, Copy)]
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
    /// Which Feature Unit controls the host gets, on each audio channel (the
    /// master channel advertises none, like the Speaker);
    /// [`FeatureUnitControls::NONE`] leaves the Feature Unit out of the
    /// function altogether.
    pub feature_unit: FeatureUnitControls,
}

impl<'d> Config<'d> {
    /// A microphone with a mute control and no volume control: the common case.
    pub const fn new(sample_rates_hz: &'d [u32], sample_width: SampleWidth, channels: &'d [Channel]) -> Self {
        Self {
            sample_rates_hz,
            sample_width,
            channels,
            input_terminal: TerminalType::InMicrophone,
            feature_unit: FeatureUnitControls {
                mute: true,
                volume: false,
            },
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
    /// terminal, or a too-small `control_buf`.
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
            // The Feature Unit only if any control was asked for, with the
            // controls on every channel; the master has none of its own,
            // matching the Speaker.
            feature_unit: (feature_unit != FeatureUnitControls::NONE)
                .then_some((FeatureUnitControls::NONE, feature_unit)),
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
                alt.alloc_endpoint_in(EndpointType::Isochronous, None, max_packet_size, 1)
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
    /// Writes a single packet into the IN endpoint
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.streaming_endpoint.write(data).await
    }

    /// Waits for the USB host to enable this interface: it is listening.
    pub async fn wait_connection(&mut self) {
        self.streaming_endpoint.wait_enabled().await;
    }
}
