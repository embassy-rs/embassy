//! USB Audio Class 2.0 - audio I/O box
//!
//! Line out (host to device) and line in (device to host) in one audio function, in the
//! `IO_BOX` category. Both streams are asynchronous: line out reports the device's sample
//! rate through an explicit feedback endpoint, and line in sizes its packets after it.
//!
//! High speed only: one packet per microframe, and the feedback value is 16.16 samples per
//! microframe.
//!
//! ```text
//! CLOCK_SOURCE (internal programmable)
//! └── CLOCK_SELECTOR (one input)
//!     ├── IT (USB streaming) → [FU] → OT (line connector)   line out
//!     └── IT (line connector) → [FU] → OT (USB streaming)   line in
//! ```
//!
//! Both directions share one clock, so macOS groups them into one device. The clock selector
//! is for Linux, which otherwise doesn't start the feedback endpoint.
//!
//! Each direction has any number of channels the bandwidth allows: one packet of up to 1024
//! bytes per microframe, so for example 13 channels of 24-bit audio at 192 kHz, 26 at 96 kHz.
//! The channels are either numbered ([`Channels::Numbered`], the usual choice for an
//! interface's inputs and outputs) or speaker positions ([`Channels::Spatial`]).
//!
//! `N`, a const generic on the class's types, is the room for Feature Unit settings: the
//! master plus the channels of the direction with more of them. Without Feature Units it can
//! be 0.
//!
//! Stereo in and out at 48 or 96 kHz, 24-bit, with a volume control on each output:
//!
//! ```rust,ignore
//! use embassy_usb::class::uac::device::SampleWidth;
//! use embassy_usb::class::uac::device::uac2::io_box::{self, Channels, Config, FeatureUnitControls, IoBox, State};
//!
//! const RATES: [u32; 2] = [48_000, 96_000];
//! // the stream buffers: frames of one microframe at 96 kHz, plus one, 6 bytes each
//! const MAX_PACKET: usize = io_box::max_packet_size(96_000, SampleWidth::Width3Byte, 2);
//!
//! // room for the Feature Unit: the master and two channels
//! let mut config = Config::<3>::new(&RATES, SampleWidth::Width3Byte, Channels::Numbered(2), Channels::Numbered(2));
//! let v = FeatureUnitControls::VOLUME;
//! config.line_out_feature_unit = [FeatureUnitControls::empty(), v, v].into_iter().collect();
//! let mut io = IoBox::new(&mut builder, state, config);
//!
//! // each in its own task, after `wait_connection()`:
//! let n = io.line_out.read_packet(&mut buf).await?; // what the host plays
//! io.line_in.write_packet(&frames).await?; // what the device records, one microframe's worth
//! // the device's sample rate, 16.16 samples per microframe, every microframe or so
//! io.feedback.write_packet((((rate_hz as u64) << 16) / 8000) as u32).await?;
//!
//! // the rate the host selected, and the first output's volume
//! let hz = io.control_monitor.sample_rate_hz();
//! let volume = io.control_monitor.line_out_volume(0);
//! ```

use core::cell::{Cell, RefCell};
#[cfg(target_has_atomic = "32")]
use core::sync::atomic::AtomicU32;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::Vec;
#[cfg(not(target_has_atomic = "32"))]
use portable_atomic::AtomicU32;

pub use super::super::Volume;
pub use super::super::function::FeatureUnitControls;
use super::super::{Channel, SampleWidth};
use super::channel_config;
use crate::class::uac::terminal_type::TerminalType;
use crate::class::uac::uac2_codes::{
    AUDIO_FUNCTION, FUNCTION_SUBCLASS_UNDEFINED, ac_descriptor, as_descriptor, control_selector, descriptor_type,
    endpoint_descriptor, format_type, function_category, function_protocol, interface, request_code,
};
use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut, EndpointType};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

const CLOCK_SOURCE_ID: u8 = 0x01;
const CLOCK_SELECTOR_ID: u8 = 0x02;
const OUT_INPUT_TERMINAL_ID: u8 = 0x03;
const OUT_FEATURE_UNIT_ID: u8 = 0x04;
const OUT_OUTPUT_TERMINAL_ID: u8 = 0x05;
const IN_INPUT_TERMINAL_ID: u8 = 0x06;
const IN_FEATURE_UNIT_ID: u8 = 0x07;
const IN_OUTPUT_TERMINAL_ID: u8 = 0x08;

/// Audio Device Class release number (bcdADC).
const ADC_VERSION: u16 = 0x0200;

const CLOCK_TYPE_INTERNAL_PROGRAMMABLE: u8 = 0x03;

const CONTROL_RO: u8 = 0b01;
const CONTROL_RW: u8 = 0b11;

// Volume settings are 8q8 values in units of dB.
const VOLUME_STEPS_PER_DB: i16 = 256;

/// bInterval of the optional interrupt endpoint, in milliseconds.
const INTERRUPT_INTERVAL_MS: u8 = 4;
/// An interrupt data message: bInfo, bAttribute, wValue, wIndex.
const INTERRUPT_MESSAGE_LEN: usize = 6;

/// The CUR setting for silence (-∞ dB), which every volume control accepts
/// regardless of its advertised range.
const VOLUME_SILENCE_8Q8_DB: i16 = i16::MIN;

const MAX_SAMPLE_RATE_COUNT: usize = 10;

/// The isochronous packet limit at high speed (one transaction per microframe).
const MAX_ISO_PACKET_SIZE: usize = 1024;

/// A class-specific descriptor's body: bLength is a byte, and covers itself and bDescriptorType.
const MAX_DESCRIPTOR_BODY: usize = 255 - 2;

/// The channels of one direction, as the host is told about them.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channels<'d> {
    /// Channels without speaker positions (`bmChannelConfig` 0), one to 255: an interface's
    /// inputs and outputs. The host numbers them.
    Numbered(u8),
    /// One channel per speaker position, in stream order. Entries must be unique and follow
    /// `bmChannelConfig` bit order, as listed in [`Channel`].
    Spatial(&'d [Channel]),
}

impl Channels<'_> {
    /// The number of channels.
    pub const fn count(&self) -> usize {
        match self {
            Channels::Numbered(n) => *n as usize,
            Channels::Spatial(c) => c.len(),
        }
    }

    fn config(&self) -> u32 {
        match self {
            Channels::Numbered(_) => 0,
            Channels::Spatial(c) => channel_config(c),
        }
    }
}

/// The isochronous packet size a direction needs: the frames of one microframe at the highest
/// rate, plus one for the rate to drift, times the frame size. Size the stream buffers with it.
pub const fn max_packet_size(max_rate_hz: u32, sample_width: SampleWidth, channels: usize) -> usize {
    (max_rate_hz.div_ceil(8000) as usize + 1) * sample_width as usize * channels
}

/// The volume range a Feature Unit advertises, in whole dB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VolumeRange {
    /// Lowest volume.
    pub min_db: i8,
    /// Highest volume.
    pub max_db: i8,
    /// Step between settings.
    pub res_db: i8,
}

impl VolumeRange {
    /// -100 dB to 0 dB in steps of 1 dB.
    pub const DEFAULT: Self = Self {
        min_db: -100,
        max_db: 0,
        res_db: 1,
    };
}

impl Default for VolumeRange {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl VolumeRange {
    fn contains_8q8(&self, volume: i16) -> bool {
        (self.min_db as i16 * VOLUME_STEPS_PER_DB..=self.max_db as i16 * VOLUME_STEPS_PER_DB).contains(&volume)
    }

    /// The setting nearest to 0 dB, used until the host or the device sets one.
    fn initial_8q8(&self) -> i16 {
        0i16.clamp(self.min_db as i16, self.max_db as i16) * VOLUME_STEPS_PER_DB
    }
}

/// Returns the control selector, the high byte of `wValue`, as defined in `uac2_codes`.
fn selector(req: &Request) -> u16 {
    req.value & 0xFF00
}

/// I/O box stream settings and optional host controls.
#[derive(Clone)]
pub struct Config<'d, const N: usize> {
    /// Supported sample rates in Hz, shared by both directions: one to ten discrete values, in
    /// any order. The first is reported before the host selects a rate.
    pub sample_rates_hz: &'d [u32],
    /// The audio sample resolution of both streams.
    pub sample_width: SampleWidth,
    /// Line out channels.
    pub line_out_channels: Channels<'d>,
    /// Line in channels.
    pub line_in_channels: Channels<'d>,
    /// Controls the USB host can change on each line out channel.
    ///
    /// Use `heapless::Vec::new()` to omit the Feature Unit. Otherwise, provide one entry more
    /// than there are channels: the master first, then each channel in stream order. Use
    /// [`FeatureUnitControls::empty`] for no controls on a channel. At most 62 entries (the
    /// descriptor's length is a byte).
    pub line_out_feature_unit: Vec<FeatureUnitControls, N, u8>,
    /// Controls the USB host can change on each line in channel, as
    /// [`Self::line_out_feature_unit`].
    pub line_in_feature_unit: Vec<FeatureUnitControls, N, u8>,
    /// The volume range of the line out Feature Unit.
    pub line_out_volume_range: VolumeRange,
    /// The volume range of the line in Feature Unit.
    pub line_in_volume_range: VolumeRange,
    /// Add an AudioControl interrupt endpoint, so the host learns about volume and mute
    /// changes the device makes itself ([`ControlMonitor::set_line_out_volume`]).
    pub interrupt_endpoint: bool,
}

impl<'d, const N: usize> Config<'d, N> {
    /// Creates an I/O box configuration without Feature Units.
    pub const fn new(
        sample_rates_hz: &'d [u32],
        sample_width: SampleWidth,
        line_out_channels: Channels<'d>,
        line_in_channels: Channels<'d>,
    ) -> Self {
        Self {
            sample_rates_hz,
            sample_width,
            line_out_channels,
            line_in_channels,
            line_out_feature_unit: Vec::new(),
            line_in_feature_unit: Vec::new(),
            line_out_volume_range: VolumeRange::DEFAULT,
            line_in_volume_range: VolumeRange::DEFAULT,
            interrupt_endpoint: false,
        }
    }
}

/// Internal state for the I/O box.
#[derive(Default)]
pub struct State<'d, const N: usize> {
    control: Option<Control<'d, N>>,
    shared: SharedControl<'d, N>,
}

impl<'d, const N: usize> State<'d, N> {
    /// Create a new `State`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// USB Audio Class 2.0 I/O box: line out + line in with explicit feedback.
pub struct IoBox<'d, D: Driver<'d>, const N: usize> {
    /// Line out stream (host to device).
    pub line_out: LineOutStream<'d, D>,
    /// Line in stream (device to host).
    pub line_in: LineInStream<'d, D>,
    /// Line out's explicit feedback endpoint.
    pub feedback: Feedback<'d, D>,
    /// The AudioControl interrupt endpoint, if [`Config::interrupt_endpoint`] is set.
    pub interrupt: Option<Interrupt<'d, D, N>>,
    /// Control monitor.
    pub control_monitor: ControlMonitor<'d, N>,
}

impl<'d, D: Driver<'d>, const N: usize> IoBox<'d, D, N> {
    /// Creates a new [`IoBox`] function.
    ///
    /// # Panics
    ///
    /// If there is no sample rate or more than ten, no channel in a direction, a direction
    /// whose packets ([`max_packet_size`]) exceed the 1024-byte isochronous limit, a nonempty
    /// feature unit without one entry for the master and each channel, or with more than 62,
    /// unsupported control bits, or a `control_buf` too small for the sample rate list.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d, N>, config: Config<'d, N>) -> Self {
        let Config {
            sample_rates_hz,
            sample_width,
            line_out_channels,
            line_in_channels,
            line_out_feature_unit,
            line_in_feature_unit,
            line_out_volume_range,
            line_in_volume_range,
            interrupt_endpoint,
        } = config;

        assert!(
            (1..=MAX_SAMPLE_RATE_COUNT).contains(&sample_rates_hz.len()),
            "between one and ten sample rates"
        );
        let max_rate_hz = sample_rates_hz.iter().copied().max().unwrap_or(0);
        for (channels, feature_unit) in [
            (line_out_channels, &line_out_feature_unit),
            (line_in_channels, &line_in_feature_unit),
        ] {
            assert!(channels.count() > 0, "at least one channel");
            assert!(
                max_packet_size(max_rate_hz, sample_width, channels.count()) <= MAX_ISO_PACKET_SIZE,
                "the channels don't fit a 1024-byte isochronous packet at the highest rate"
            );
            assert!(
                feature_unit.is_empty() || feature_unit.len() == channels.count() + 1,
                "a feature unit must be empty or contain the master and every audio channel"
            );
            assert!(
                feature_unit_len(feature_unit.len()) <= MAX_DESCRIPTOR_BODY,
                "at most 62 feature unit entries"
            );
            assert!(
                feature_unit
                    .iter()
                    .all(|controls| FeatureUnitControls::all().contains(*controls)),
                "unsupported Feature Unit controls"
            );
        }
        for range in [line_out_volume_range, line_in_volume_range] {
            assert!(
                range.min_db < range.max_db && range.res_db > 0,
                "a volume range needs min < max and a positive step"
            );
        }
        let line_out_max_packet_size = max_packet_size(max_rate_hz, sample_width, line_out_channels.count()) as u16;
        let line_in_max_packet_size = max_packet_size(max_rate_hz, sample_width, line_in_channels.count()) as u16;
        assert!(
            builder.control_buf_len() >= 2 + 12 * sample_rates_hz.len(),
            "control_buf too small for the sample rate RANGE reply"
        );

        let mut func = builder.function(
            AUDIO_FUNCTION,
            FUNCTION_SUBCLASS_UNDEFINED,
            function_protocol::AF_VERSION_02_00,
        );

        let mut iface = func.interface();
        let control_interface = iface.interface_number();
        let mut alt = iface.alt_setting(
            interface::AUDIO,
            interface::subclass::AUDIOCONTROL,
            interface::protocol::IP_VERSION_02_00,
            None,
        );

        let clock_source = [
            ac_descriptor::CLOCK_SOURCE,
            CLOCK_SOURCE_ID,
            CLOCK_TYPE_INTERNAL_PROGRAMMABLE,
            CONTROL_RW | (CONTROL_RO << 2), // bmControls: frequency R/W, validity R/O
            0x00,                           // bAssocTerminal
            0x00,                           // iClockSource
        ];
        let clock_selector = [
            ac_descriptor::CLOCK_SELECTOR,
            CLOCK_SELECTOR_ID,
            0x01, // bNrInPins
            CLOCK_SOURCE_ID,
            CONTROL_RW, // bmControls: clock selector R/W
            0x00,       // iClockSelector
        ];

        let out_has_fu = !line_out_feature_unit.is_empty();
        let in_has_fu = !line_in_feature_unit.is_empty();

        let out_input_terminal = input_terminal(OUT_INPUT_TERMINAL_ID, TerminalType::UsbStreaming, line_out_channels);
        let out_feature_unit = feature_unit(OUT_FEATURE_UNIT_ID, OUT_INPUT_TERMINAL_ID, &line_out_feature_unit);
        let out_output_terminal = output_terminal(
            OUT_OUTPUT_TERMINAL_ID,
            TerminalType::ExtLineConnector,
            if out_has_fu {
                OUT_FEATURE_UNIT_ID
            } else {
                OUT_INPUT_TERMINAL_ID
            },
        );
        let in_input_terminal = input_terminal(IN_INPUT_TERMINAL_ID, TerminalType::ExtLineConnector, line_in_channels);
        let in_feature_unit = feature_unit(IN_FEATURE_UNIT_ID, IN_INPUT_TERMINAL_ID, &line_in_feature_unit);
        let in_output_terminal = output_terminal(
            IN_OUTPUT_TERMINAL_ID,
            TerminalType::UsbStreaming,
            if in_has_fu {
                IN_FEATURE_UNIT_ID
            } else {
                IN_INPUT_TERMINAL_ID
            },
        );

        let mut entities: Vec<&[u8], 8> = Vec::new();
        let _ = entities.push(&clock_source);
        let _ = entities.push(&clock_selector);
        let _ = entities.push(&out_input_terminal);
        if out_has_fu {
            let _ = entities.push(&out_feature_unit);
        }
        let _ = entities.push(&out_output_terminal);
        let _ = entities.push(&in_input_terminal);
        if in_has_fu {
            let _ = entities.push(&in_feature_unit);
        }
        let _ = entities.push(&in_output_terminal);

        // The total length covers the header and every entity, each with bLength and bDescriptorType.
        const HEADER_BODY: usize = 7;
        let total_len: usize = 2 + HEADER_BODY + entities.iter().map(|e| e.len() + 2).sum::<usize>();
        alt.descriptor(
            descriptor_type::CS_INTERFACE,
            &[
                ac_descriptor::HEADER,
                ADC_VERSION as u8,
                (ADC_VERSION >> 8) as u8,
                function_category::IO_BOX,
                total_len as u8,
                (total_len >> 8) as u8,
                0x00, // bmControls
            ],
        );
        for entity in entities {
            alt.descriptor(descriptor_type::CS_INTERFACE, entity);
        }
        let interrupt_ep = interrupt_endpoint
            .then(|| alt.endpoint_interrupt_in(None, INTERRUPT_MESSAGE_LEN as u16, INTERRUPT_INTERVAL_MS));

        let mut out_iface = func.interface();
        out_iface.alt_setting(
            interface::AUDIO,
            interface::subclass::AUDIOSTREAMING,
            interface::protocol::IP_VERSION_02_00,
            None,
        );
        let mut out_alt = out_iface.alt_setting(
            interface::AUDIO,
            interface::subclass::AUDIOSTREAMING,
            interface::protocol::IP_VERSION_02_00,
            None,
        );
        out_alt.descriptor(
            descriptor_type::CS_INTERFACE,
            &as_general(OUT_INPUT_TERMINAL_ID, line_out_channels),
        );
        out_alt.descriptor(descriptor_type::CS_INTERFACE, &format_type_i(sample_width));

        // Every microframe.
        let out_ep = out_alt.alloc_endpoint_out(EndpointType::Isochronous, None, line_out_max_packet_size, 0);
        // Allocated before the feedback endpoint, so that drivers allocating in order don't
        // give the feedback endpoint line out's endpoint number.
        let in_ep = out_alt.alloc_endpoint_in(EndpointType::Isochronous, None, line_in_max_packet_size, 0);

        out_alt.endpoint_descriptor(
            out_ep.info(),
            SynchronizationType::Asynchronous,
            UsageType::DataEndpoint,
            &[],
        );
        out_alt.descriptor(descriptor_type::CS_ENDPOINT, &iso_data_endpoint());

        let feedback_ep = out_alt.alloc_endpoint_in(EndpointType::Isochronous, None, 4, 1);
        out_alt.endpoint_descriptor(
            feedback_ep.info(),
            SynchronizationType::NoSynchronization,
            UsageType::FeedbackEndpoint,
            &[],
        );

        let mut in_iface = func.interface();
        in_iface.alt_setting(
            interface::AUDIO,
            interface::subclass::AUDIOSTREAMING,
            interface::protocol::IP_VERSION_02_00,
            None,
        );
        let mut in_alt = in_iface.alt_setting(
            interface::AUDIO,
            interface::subclass::AUDIOSTREAMING,
            interface::protocol::IP_VERSION_02_00,
            None,
        );
        in_alt.descriptor(
            descriptor_type::CS_INTERFACE,
            &as_general(IN_OUTPUT_TERMINAL_ID, line_in_channels),
        );
        in_alt.descriptor(descriptor_type::CS_INTERFACE, &format_type_i(sample_width));
        in_alt.endpoint_descriptor(
            in_ep.info(),
            SynchronizationType::Asynchronous,
            UsageType::DataEndpoint,
            &[],
        );
        in_alt.descriptor(descriptor_type::CS_ENDPOINT, &iso_data_endpoint());

        drop(func);

        let shared = &mut state.shared;
        shared.line_out_controls = line_out_feature_unit;
        shared.line_in_controls = line_in_feature_unit;
        shared.line_out_volume_range = line_out_volume_range;
        shared.line_in_volume_range = line_in_volume_range;
        for (settings, range) in [
            (&shared.line_out_settings, line_out_volume_range),
            (&shared.line_in_settings, line_in_volume_range),
        ] {
            settings.lock(|x| {
                *x.borrow_mut() = AudioSettings {
                    muted: [false; N],
                    volume_8q8_db: [range.initial_8q8(); N],
                }
            });
        }
        shared.sample_rates_hz = sample_rates_hz;
        shared.sample_rate_hz.store(sample_rates_hz[0], Ordering::Relaxed);

        state.control = Some(Control {
            shared: &state.shared,
            control_interface_number: control_interface,
        });
        builder.handler(state.control.as_mut().unwrap());

        Self {
            line_out: LineOutStream { ep: out_ep },
            line_in: LineInStream { ep: in_ep },
            feedback: Feedback { ep: feedback_ep },
            interrupt: interrupt_ep.map(|ep| Interrupt {
                ep,
                shared: &state.shared,
                interface: control_interface.into(),
            }),
            control_monitor: ControlMonitor { shared: &state.shared },
        }
    }
}

// Descriptor bodies (without bLength / bDescriptorType).

fn input_terminal(id: u8, terminal_type: TerminalType, channels: Channels) -> [u8; 15] {
    let terminal_type: u16 = terminal_type.into();
    let cfg = channels.config();
    [
        ac_descriptor::INPUT_TERMINAL,
        id,
        terminal_type as u8,
        (terminal_type >> 8) as u8,
        0x00, // bAssocTerminal
        CLOCK_SELECTOR_ID,
        channels.count() as u8,
        cfg as u8,
        (cfg >> 8) as u8,
        (cfg >> 16) as u8,
        (cfg >> 24) as u8,
        0x00, // iChannelNames
        0x00,
        0x00, // bmControls
        0x00, // iTerminal
    ]
}

fn output_terminal(id: u8, terminal_type: TerminalType, source_id: u8) -> [u8; 10] {
    let terminal_type: u16 = terminal_type.into();
    [
        ac_descriptor::OUTPUT_TERMINAL,
        id,
        terminal_type as u8,
        (terminal_type >> 8) as u8,
        0x00, // bAssocTerminal
        source_id,
        CLOCK_SELECTOR_ID,
        0x00,
        0x00, // bmControls
        0x00, // iTerminal
    ]
}

/// A Feature Unit descriptor's body for `entries` (the master and each channel).
const fn feature_unit_len(entries: usize) -> usize {
    3 + 4 * entries + 1
}

/// Feature Unit descriptor, with the master's controls first. Empty if there are no controls.
fn feature_unit(id: u8, source_id: u8, controls: &[FeatureUnitControls]) -> Vec<u8, MAX_DESCRIPTOR_BODY> {
    let mut v = Vec::new();
    if controls.is_empty() {
        return v;
    }
    let _ = v.extend_from_slice(&[ac_descriptor::FEATURE_UNIT, id, source_id]);
    for c in controls {
        let mut bits = 0u32;
        if c.contains(FeatureUnitControls::MUTE) {
            bits |= CONTROL_RW as u32;
        }
        if c.contains(FeatureUnitControls::VOLUME) {
            bits |= (CONTROL_RW as u32) << 2;
        }
        let _ = v.extend_from_slice(&bits.to_le_bytes());
    }
    let _ = v.push(0x00); // iFeature
    v
}

fn as_general(terminal_link: u8, channels: Channels) -> [u8; 14] {
    let cfg = channels.config();
    let formats = format_type::Type1::PCM.bits();
    [
        as_descriptor::GENERAL,
        terminal_link,
        0x00, // bmControls
        format_type::I,
        formats as u8,
        (formats >> 8) as u8,
        (formats >> 16) as u8,
        (formats >> 24) as u8,
        channels.count() as u8,
        cfg as u8,
        (cfg >> 8) as u8,
        (cfg >> 16) as u8,
        (cfg >> 24) as u8,
        0x00, // iChannelNames
    ]
}

fn format_type_i(sample_width: SampleWidth) -> [u8; 4] {
    [
        as_descriptor::FORMAT_TYPE,
        format_type::I,
        sample_width as u8,
        sample_width.in_bit() as u8,
    ]
}

/// Lock delay of 8 samples.
fn iso_data_endpoint() -> [u8; 6] {
    [endpoint_descriptor::EP_GENERAL, 0x00, 0x00, 0x02, 0x08, 0x00]
}

/// Line out stream (host to device, isochronous OUT endpoint).
pub struct LineOutStream<'d, D: Driver<'d>> {
    ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> LineOutStream<'d, D> {
    /// Reads one audio packet from the host. Returns the number of bytes received.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.ep.read(data).await
    }

    /// Waits for the host to select the streaming alternate setting.
    pub async fn wait_connection(&mut self) {
        self.ep.wait_enabled().await;
    }

    /// The isochronous OUT endpoint's index.
    pub fn endpoint_index(&self) -> usize {
        self.ep.info().addr.index()
    }
}

/// Line in stream (device to host, isochronous IN endpoint).
pub struct LineInStream<'d, D: Driver<'d>> {
    ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> LineInStream<'d, D> {
    /// Writes one audio packet to the host: the frames captured since the last one.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.ep.write(data).await
    }

    /// Waits for the host to select the streaming alternate setting.
    pub async fn wait_connection(&mut self) {
        self.ep.wait_enabled().await;
    }

    /// The isochronous IN endpoint's index.
    pub fn endpoint_index(&self) -> usize {
        self.ep.info().addr.index()
    }
}

/// Line out's explicit feedback endpoint: 16.16 samples per microframe.
///
/// Write it with [`Self::write_packet`], or from the SOF interrupt through
/// [`Self::endpoint_index`] if the driver supports that.
pub struct Feedback<'d, D: Driver<'d>> {
    ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Feedback<'d, D> {
    /// Writes one feedback value.
    pub async fn write_packet(&mut self, value: u32) -> Result<(), EndpointError> {
        self.ep.write(&value.to_le_bytes()).await
    }

    /// Waits for the host to select line out's streaming alternate setting.
    pub async fn wait_connection(&mut self) {
        self.ep.wait_enabled().await;
    }

    /// The feedback endpoint's index.
    pub fn endpoint_index(&self) -> usize {
        self.ep.info().addr.index()
    }
}

/// The AudioControl interrupt endpoint: tells the host which controls the device changed
/// itself, so it reads them again. Keep [`Self::run`] running.
pub struct Interrupt<'d, D: Driver<'d>, const N: usize> {
    ep: D::EndpointIn,
    shared: &'d SharedControl<'d, N>,
    interface: u8,
}

impl<'d, D: Driver<'d>, const N: usize> Interrupt<'d, D, N> {
    /// Sends a notification for each control changed through
    /// [`ControlMonitor::set_line_out_volume`] or [`ControlMonitor::set_line_in_volume`].
    pub async fn run(&mut self) -> ! {
        loop {
            self.ep.wait_enabled().await;
            loop {
                let p = self.shared.pending.lock(|x| x.replace(Pending::default()));
                if p.is_empty() {
                    self.shared.pending_signal.wait().await;
                    continue;
                }
                if self.notify(OUT_FEATURE_UNIT_ID, &p.line_out).await.is_err()
                    || self.notify(IN_FEATURE_UNIT_ID, &p.line_in).await.is_err()
                {
                    break;
                }
            }
        }
    }

    async fn notify(&mut self, entity: u8, changed: &[u8; N]) -> Result<(), EndpointError> {
        for (channel, &c) in changed.iter().enumerate() {
            for (bit, control) in [
                (PENDING_MUTE, control_selector::feature_unit::MUTE_CONTROL),
                (PENDING_VOLUME, control_selector::feature_unit::VOLUME_CONTROL),
            ] {
                if c & bit == 0 {
                    continue;
                }
                // bInfo: class-specific, interface; bAttribute: CUR; wValue: CS, CN; wIndex: entity, interface
                let msg = [
                    0x00,
                    request_code::CUR,
                    channel as u8,
                    (control >> 8) as u8,
                    self.interface,
                    entity,
                ];
                self.ep.write(&msg).await?;
            }
        }
        Ok(())
    }
}

/// Audio settings for a feature unit: mute and volume per channel, the master first.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct AudioSettings<const N: usize> {
    muted: [bool; N],
    volume_8q8_db: [i16; N],
}

impl<const N: usize> Default for AudioSettings<N> {
    fn default() -> Self {
        AudioSettings {
            muted: [false; N],
            volume_8q8_db: [0; N],
        }
    }
}

const PENDING_MUTE: u8 = 0b01;
const PENDING_VOLUME: u8 = 0b10;

/// Controls with changes for the host to read, per Feature Unit and channel (the master
/// first): [`PENDING_MUTE`], [`PENDING_VOLUME`].
#[derive(Clone, Copy)]
struct Pending<const N: usize> {
    line_out: [u8; N],
    line_in: [u8; N],
}

impl<const N: usize> Default for Pending<N> {
    fn default() -> Self {
        Pending {
            line_out: [0; N],
            line_in: [0; N],
        }
    }
}

impl<const N: usize> Pending<N> {
    fn is_empty(&self) -> bool {
        self.line_out.iter().chain(&self.line_in).all(|&c| c == 0)
    }
}

struct SharedControl<'d, const N: usize> {
    line_out_settings: CriticalSectionMutex<RefCell<AudioSettings<N>>>,
    line_in_settings: CriticalSectionMutex<RefCell<AudioSettings<N>>>,
    line_out_controls: Vec<FeatureUnitControls, N, u8>,
    line_in_controls: Vec<FeatureUnitControls, N, u8>,
    line_out_volume_range: VolumeRange,
    line_in_volume_range: VolumeRange,
    pending: CriticalSectionMutex<Cell<Pending<N>>>,
    pending_signal: Signal<CriticalSectionRawMutex, ()>,
    sample_rates_hz: &'d [u32],
    sample_rate_hz: AtomicU32,
    clock_valid: AtomicBool,
    changed: Signal<CriticalSectionRawMutex, ()>,
}

impl<'d, const N: usize> Default for SharedControl<'d, N> {
    fn default() -> Self {
        SharedControl {
            line_out_settings: CriticalSectionMutex::new(RefCell::new(AudioSettings::default())),
            line_in_settings: CriticalSectionMutex::new(RefCell::new(AudioSettings::default())),
            line_out_controls: Vec::new(),
            line_in_controls: Vec::new(),
            line_out_volume_range: VolumeRange::DEFAULT,
            line_in_volume_range: VolumeRange::DEFAULT,
            pending: CriticalSectionMutex::new(Cell::new(Pending::default())),
            pending_signal: Signal::new(),
            sample_rates_hz: &[],
            sample_rate_hz: AtomicU32::new(0),
            clock_valid: AtomicBool::new(true),
            changed: Signal::new(),
        }
    }
}

struct FeatureUnitRef<'a, const N: usize> {
    settings: &'a CriticalSectionMutex<RefCell<AudioSettings<N>>>,
    controls: &'a [FeatureUnitControls],
    range: VolumeRange,
}

impl<const N: usize> FeatureUnitRef<'_, N> {
    /// Controls for a USB channel number (0 = master). Missing entries have no controls.
    fn controls(&self, channel: u8) -> FeatureUnitControls {
        self.controls
            .get(channel as usize)
            .copied()
            .unwrap_or_else(FeatureUnitControls::empty)
    }
}

/// Control status change monitor.
///
/// Await [`Self::changed`], then read the updated settings. Channels are numbered from 0 in
/// stream order.
pub struct ControlMonitor<'d, const N: usize> {
    shared: &'d SharedControl<'d, N>,
}

impl<'d, const N: usize> ControlMonitor<'d, N> {
    fn volume(
        settings: &CriticalSectionMutex<RefCell<AudioSettings<N>>>,
        controls: &[FeatureUnitControls],
        channel: usize,
    ) -> Option<Volume> {
        // The logical channels start at one (zero is the master channel).
        let index = channel + 1;
        let has = |i: usize| controls.get(i).is_some_and(|c| !c.is_empty());
        if index >= controls.len() || (!has(0) && !has(index)) {
            return None;
        }
        // the two entries only: the lock holds interrupts off
        let (muted, volume) = settings.lock(|x| {
            let s = x.borrow();
            (
                [s.muted[0], s.muted[index]],
                [s.volume_8q8_db[0], s.volume_8q8_db[index]],
            )
        });
        if (0..2).any(|i| muted[i] || volume[i] == VOLUME_SILENCE_8Q8_DB) {
            return Some(Volume::Muted);
        }
        Some(Volume::DeciBel(
            (volume[0] as f32 + volume[1] as f32) / VOLUME_STEPS_PER_DB as f32,
        ))
    }

    /// A line out channel's volume with the master applied. `None` without a Feature Unit, for
    /// a channel past the last, or if neither the channel nor the master has controls.
    pub fn line_out_volume(&self, channel: usize) -> Option<Volume> {
        let s = self.shared;
        Self::volume(&s.line_out_settings, &s.line_out_controls, channel)
    }

    /// A line in channel's volume with the master applied, as [`Self::line_out_volume`].
    pub fn line_in_volume(&self, channel: usize) -> Option<Volume> {
        let s = self.shared;
        Self::volume(&s.line_in_settings, &s.line_in_controls, channel)
    }

    /// Get the sample rate in Hz. Before the host sets one, and again after a bus reset, it is
    /// the first advertised rate.
    pub fn sample_rate_hz(&self) -> u32 {
        self.shared.sample_rate_hz.load(Ordering::Relaxed)
    }

    /// Set the clock validity reported to the host (valid by default). Report the clock as
    /// invalid while it doesn't run at the selected rate, for example during a rate change.
    pub fn set_clock_valid(&self, valid: bool) {
        self.shared.clock_valid.store(valid, Ordering::Relaxed);
    }

    /// Set a line out channel's own volume (not the master's) from the device side, for
    /// example from a front panel. The host is told through the interrupt endpoint, if there is
    /// one; [`Self::changed`] doesn't fire. Without a mute control, [`Volume::Muted`] sets the
    /// volume to silence. Out-of-range volumes are clamped to the advertised range.
    pub fn set_line_out_volume(&self, channel: usize, volume: Volume) {
        let s = self.shared;
        let changed = Self::set_volume(
            &s.line_out_settings,
            &s.line_out_controls,
            s.line_out_volume_range,
            channel,
            volume,
        );
        self.add_pending(true, channel + 1, changed);
    }

    /// Set a line in channel's own volume from the device side, as [`Self::set_line_out_volume`].
    pub fn set_line_in_volume(&self, channel: usize, volume: Volume) {
        let s = self.shared;
        let changed = Self::set_volume(
            &s.line_in_settings,
            &s.line_in_controls,
            s.line_in_volume_range,
            channel,
            volume,
        );
        self.add_pending(false, channel + 1, changed);
    }

    /// Returns the controls that changed: [`PENDING_MUTE`], [`PENDING_VOLUME`].
    fn set_volume(
        settings: &CriticalSectionMutex<RefCell<AudioSettings<N>>>,
        controls: &[FeatureUnitControls],
        range: VolumeRange,
        channel: usize,
        volume: Volume,
    ) -> u8 {
        let index = channel + 1;
        let Some(&c) = controls.get(index) else {
            return 0;
        };
        // (mute, volume to set) worked out before the lock, which holds interrupts off
        let (muted, v) = match volume {
            Volume::Muted if c.contains(FeatureUnitControls::MUTE) => (true, None),
            Volume::Muted => (false, Some(VOLUME_SILENCE_8Q8_DB)),
            Volume::DeciBel(db) => {
                let db = db.clamp(range.min_db as f32, range.max_db as f32);
                // round half away from zero (no `f32::round` in core)
                let steps = db * VOLUME_STEPS_PER_DB as f32;
                (false, Some((steps + if steps < 0.0 { -0.5 } else { 0.5 }) as i16))
            }
        };
        settings.lock(|x| {
            let mut s = x.borrow_mut();
            let mut changed = 0;
            if c.contains(FeatureUnitControls::MUTE) && s.muted[index] != muted {
                s.muted[index] = muted;
                changed |= PENDING_MUTE;
            }
            match v {
                Some(v) if c.contains(FeatureUnitControls::VOLUME) && s.volume_8q8_db[index] != v => {
                    s.volume_8q8_db[index] = v;
                    changed |= PENDING_VOLUME;
                }
                _ => {}
            }
            changed
        })
    }

    fn add_pending(&self, line_out: bool, index: usize, changed: u8) {
        if changed == 0 {
            return;
        }
        self.shared.pending.lock(|x| {
            let mut p = x.get();
            let unit = if line_out { &mut p.line_out } else { &mut p.line_in };
            unit[index] |= changed;
            x.set(p);
        });
        self.shared.pending_signal.signal(());
    }

    /// Wait for the sample rate, a volume or a mute state to change. Single consumer: await this
    /// from at most one task.
    pub async fn changed(&self) {
        self.shared.changed.wait().await;
    }
}

struct Control<'d, const N: usize> {
    control_interface_number: InterfaceNumber,
    shared: &'d SharedControl<'d, N>,
}

impl<'d, const N: usize> Control<'d, N> {
    fn feature_unit(&self, entity: u8) -> Option<FeatureUnitRef<'d, N>> {
        let s = self.shared;
        match entity {
            OUT_FEATURE_UNIT_ID if !s.line_out_controls.is_empty() => Some(FeatureUnitRef {
                settings: &s.line_out_settings,
                controls: &s.line_out_controls,
                range: s.line_out_volume_range,
            }),
            IN_FEATURE_UNIT_ID if !s.line_in_controls.is_empty() => Some(FeatureUnitRef {
                settings: &s.line_in_settings,
                controls: &s.line_in_controls,
                range: s.line_in_volume_range,
            }),
            _ => None,
        }
    }

    fn set_request(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.index as u8 != u8::from(self.control_interface_number) {
            return None;
        }
        if req.request != request_code::CUR {
            return Some(OutResponse::Rejected);
        }
        let entity = (req.index >> 8) as u8;
        let channel = req.value as u8;

        match entity {
            CLOCK_SOURCE_ID => {
                if selector(&req) != control_selector::clock_source::SAMPLING_FREQ_CONTROL || data.len() < 4 {
                    return Some(OutResponse::Rejected);
                }
                let rate = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                if !self.shared.sample_rates_hz.contains(&rate) {
                    debug!("Unsupported sample rate {} Hz", rate);
                    return Some(OutResponse::Rejected);
                }
                self.shared.sample_rate_hz.store(rate, Ordering::Relaxed);
                debug!("Set sample rate to {} Hz", rate);
                self.shared.changed.signal(());
                Some(OutResponse::Accepted)
            }
            // There is one input, so pin 1 is the only valid selection.
            CLOCK_SELECTOR_ID => {
                if selector(&req) != control_selector::clock_selector::CLOCK_SELECTOR_CONTROL
                    || data.first() != Some(&1)
                {
                    return Some(OutResponse::Rejected);
                }
                Some(OutResponse::Accepted)
            }
            _ => {
                let Some(fu) = self.feature_unit(entity) else {
                    return Some(OutResponse::Rejected);
                };
                let (control, len) = match selector(&req) {
                    control_selector::feature_unit::MUTE_CONTROL => (FeatureUnitControls::MUTE, 1),
                    control_selector::feature_unit::VOLUME_CONTROL => (FeatureUnitControls::VOLUME, 2),
                    _ => return Some(OutResponse::Rejected),
                };
                if !fu.controls(channel).contains(control) || data.len() < len {
                    return Some(OutResponse::Rejected);
                }
                let accepted = fu.settings.lock(|x| {
                    let mut s = x.borrow_mut();
                    if control == FeatureUnitControls::MUTE {
                        s.muted[channel as usize] = data[0] != 0;
                    } else {
                        let volume = i16::from_le_bytes([data[0], data[1]]);
                        // CUR lies within the advertised range, except silence, which is always accepted.
                        if volume != VOLUME_SILENCE_8Q8_DB && !fu.range.contains_8q8(volume) {
                            return false;
                        }
                        s.volume_8q8_db[channel as usize] = volume;
                    }
                    true
                });
                if !accepted {
                    return Some(OutResponse::Rejected);
                }
                self.shared.changed.signal(());
                Some(OutResponse::Accepted)
            }
        }
    }

    fn get_request<'r>(&'r mut self, req: Request, buf: &'r mut [u8]) -> Option<InResponse<'r>> {
        if req.index as u8 != u8::from(self.control_interface_number) {
            return None;
        }
        let entity = (req.index >> 8) as u8;
        let channel = req.value as u8;

        match (entity, req.request, selector(&req)) {
            (CLOCK_SOURCE_ID, request_code::CUR, control_selector::clock_source::SAMPLING_FREQ_CONTROL)
                if buf.len() >= 4 =>
            {
                buf[..4].copy_from_slice(&self.shared.sample_rate_hz.load(Ordering::Relaxed).to_le_bytes());
                Some(InResponse::Accepted(&buf[..4]))
            }
            (CLOCK_SOURCE_ID, request_code::CUR, control_selector::clock_source::CLOCK_VALID_CONTROL)
                if !buf.is_empty() =>
            {
                buf[0] = self.shared.clock_valid.load(Ordering::Relaxed) as u8;
                Some(InResponse::Accepted(&buf[..1]))
            }
            (CLOCK_SOURCE_ID, request_code::RANGE, control_selector::clock_source::SAMPLING_FREQ_CONTROL) => {
                let rates = self.shared.sample_rates_hz;
                let len = 2 + 12 * rates.len();
                if buf.len() < len {
                    return Some(InResponse::Rejected);
                }
                buf[..2].copy_from_slice(&(rates.len() as u16).to_le_bytes());
                // Sub-ranges in ascending order, whatever the configured order.
                let mut sorted = [0u32; MAX_SAMPLE_RATE_COUNT];
                let sorted = &mut sorted[..rates.len()];
                sorted.copy_from_slice(rates);
                sorted.sort_unstable();
                for (i, &rate) in sorted.iter().enumerate() {
                    let sub = &mut buf[2 + 12 * i..2 + 12 * (i + 1)];
                    sub[..4].copy_from_slice(&rate.to_le_bytes()); // dMIN
                    sub[4..8].copy_from_slice(&rate.to_le_bytes()); // dMAX
                    sub[8..].copy_from_slice(&0u32.to_le_bytes()); // dRES
                }
                Some(InResponse::Accepted(&buf[..len]))
            }
            (CLOCK_SELECTOR_ID, request_code::CUR, control_selector::clock_selector::CLOCK_SELECTOR_CONTROL)
                if !buf.is_empty() =>
            {
                buf[0] = 1;
                Some(InResponse::Accepted(&buf[..1]))
            }
            (CLOCK_SOURCE_ID | CLOCK_SELECTOR_ID, _, _) => Some(InResponse::Rejected),
            _ => {
                let Some(fu) = self.feature_unit(entity) else {
                    return Some(InResponse::Rejected);
                };
                // a channel without controls (past the last, too) is rejected below
                let controls = fu.controls(channel);
                if controls.is_empty() {
                    return Some(InResponse::Rejected);
                }
                let (muted, volume) = fu.settings.lock(|x| {
                    let s = x.borrow();
                    (s.muted[channel as usize], s.volume_8q8_db[channel as usize])
                });
                match (req.request, selector(&req)) {
                    (request_code::CUR, control_selector::feature_unit::MUTE_CONTROL)
                        if controls.contains(FeatureUnitControls::MUTE) && !buf.is_empty() =>
                    {
                        buf[0] = muted as u8;
                        Some(InResponse::Accepted(&buf[..1]))
                    }
                    (request_code::CUR, control_selector::feature_unit::VOLUME_CONTROL)
                        if controls.contains(FeatureUnitControls::VOLUME) && buf.len() >= 2 =>
                    {
                        buf[..2].copy_from_slice(&volume.to_le_bytes());
                        Some(InResponse::Accepted(&buf[..2]))
                    }
                    // One sub-range: MIN, MAX, RES.
                    (request_code::RANGE, control_selector::feature_unit::VOLUME_CONTROL)
                        if controls.contains(FeatureUnitControls::VOLUME) && buf.len() >= 8 =>
                    {
                        buf[..2].copy_from_slice(&1u16.to_le_bytes());
                        let r = fu.range;
                        buf[2..4].copy_from_slice(&(r.min_db as i16 * VOLUME_STEPS_PER_DB).to_le_bytes());
                        buf[4..6].copy_from_slice(&(r.max_db as i16 * VOLUME_STEPS_PER_DB).to_le_bytes());
                        buf[6..8].copy_from_slice(&(r.res_db as i16 * VOLUME_STEPS_PER_DB).to_le_bytes());
                        Some(InResponse::Accepted(&buf[..8]))
                    }
                    _ => Some(InResponse::Rejected),
                }
            }
        }
    }
}

impl<'d, const N: usize> Handler for Control<'d, N> {
    fn reset(&mut self) {
        // Volume and mute stay as they are: the device may keep them itself (a front panel,
        // settings in flash), and the host reads them back after enumeration.
        let s = self.shared;
        if let Some(&rate) = s.sample_rates_hz.first() {
            s.sample_rate_hz.store(rate, Ordering::Relaxed);
        }
        s.changed.signal(());
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        match (req.request_type, req.recipient) {
            (RequestType::Class, Recipient::Interface) => self.set_request(req, data),
            _ => None,
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        match (req.request_type, req.recipient) {
            (RequestType::Class, Recipient::Interface) => self.get_request(req, buf),
            _ => None,
        }
    }
}
