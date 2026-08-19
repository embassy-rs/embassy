//! What the UAC1 classes share: the audio function's descriptors, the feature unit and sampling
//! frequency control handler, and the feedback endpoint. Only the stream direction differs.

use core::cell::Cell;
use core::ops::RangeInclusive;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::Vec;

use super::class_codes::*;
use super::terminal_type::TerminalType;
use super::{
    Channel, ChannelConfig, FeedbackRefresh, MAX_AUDIO_CHANNEL_COUNT, MAX_AUDIO_CHANNEL_INDEX, SampleWidth, Volume,
};
use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointInfo, EndpointType};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler, InterfaceAltBuilder};

/// Maximum allowed sampling rate in Hz: the 3-byte tSamFreq field's limit.
const MAX_SAMPLE_RATE_HZ: u32 = 0xFF_FFFF;

/// The wChannelNumber addressing the master and every channel at once [UAC 5.2.1.2].
const ALL_CHANNELS: u8 = 0xFF;

// Volume settings go from -25600 to 0, in steps of 256.
// Therefore, the volume settings are 8q8 values in units of dB.
const VOLUME_STEPS_PER_DB: i16 = 256;
const MIN_VOLUME_DB: i16 = -100;
const MAX_VOLUME_DB: i16 = 0;

/// The CUR setting for silence (-∞ dB), which every volume control accepts
/// regardless of its advertised range [UAC 5.2.2.4.3.2].
const VOLUME_SILENCE_8Q8_DB: i16 = i16::MIN;

/// Maximum number of supported discrete sample rates.
const MAX_SAMPLE_RATE_COUNT: usize = 10;

/// Which feature unit controls the host is offered. With neither, no feature unit is described.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FeatureUnitControls {
    /// A mute control on the master channel and on each audio channel.
    pub mute: bool,
    /// A volume control on the master channel and on each audio channel.
    pub volume: bool,
}

impl FeatureUnitControls {
    /// Mute and volume.
    pub const ALL: FeatureUnitControls = FeatureUnitControls {
        mute: true,
        volume: true,
    };
    /// No feature unit.
    pub const NONE: FeatureUnitControls = FeatureUnitControls {
        mute: false,
        volume: false,
    };

    /// The bmaControls byte for one channel.
    pub(super) fn bitmap(&self) -> u8 {
        let mut controls = FU_CONTROL_UNDEFINED;
        if self.mute {
            controls |= MUTE_CONTROL;
        }
        if self.volume {
            controls |= VOLUME_CONTROL;
        }
        controls
    }
}

/// Internal state for a USB Audio Class 1.0 class.
pub struct State<'d> {
    control: Option<Control<'d>>,
    shared: SharedControl<'d>,
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub fn new() -> Self {
        Self {
            control: None,
            shared: SharedControl::default(),
        }
    }

    /// Wire the state up — what the host may set, and where — and register
    /// the handler. Returns the monitor a class hands back.
    pub(super) fn register<'b, D: Driver<'d>>(
        &'d mut self,
        builder: &'b mut Builder<'d, D>,
        channels: &'d [Channel],
        sample_rates_hz: &'d [u32],
        controls: (FeatureUnitControls, FeatureUnitControls),
        control_interface: InterfaceNumber,
        streaming_endpoint_address: u8,
    ) -> ControlMonitor<'d> {
        self.shared.channels = channels;
        self.shared.sample_rates_hz = sample_rates_hz;
        self.shared.controls = controls;
        self.shared
            .sample_rate_hz
            .store(self.shared.default_sample_rate_hz(), Ordering::Relaxed);

        self.control = Some(Control {
            shared: &self.shared,
            streaming_endpoint_address,
            control_interface_number: control_interface,
        });

        builder.handler(self.control.as_mut().unwrap());

        ControlMonitor { shared: &self.shared }
    }
}

/// Audio settings for the feature unit.
///
/// Contains volume and mute control.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioSettings {
    /// Channel mute states.
    muted: [bool; MAX_AUDIO_CHANNEL_COUNT],
    /// Channel volume levels in 8.8 format (in dB).
    volume_8q8_db: [i16; MAX_AUDIO_CHANNEL_COUNT],
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            muted: [false; MAX_AUDIO_CHANNEL_COUNT],
            volume_8q8_db: [MAX_VOLUME_DB * VOLUME_STEPS_PER_DB; MAX_AUDIO_CHANNEL_COUNT],
        }
    }
}

struct Control<'d> {
    control_interface_number: InterfaceNumber,
    streaming_endpoint_address: u8,
    shared: &'d SharedControl<'d>,
}

/// Shared data between [`Control`] and the class.
struct SharedControl<'d> {
    /// The collection of audio settings (volumes, mute states).
    audio_settings: CriticalSectionMutex<Cell<AudioSettings>>,

    /// Channel assignments.
    channels: &'d [Channel],

    /// The advertised Feature Unit controls on the master channel and on each
    /// audio channel; requests follow the descriptor [UAC 5.2.1].
    controls: (FeatureUnitControls, FeatureUnitControls),

    /// The sample rates the stream offers.
    sample_rates_hz: &'d [u32],

    /// The audio sample rate in Hz.
    sample_rate_hz: AtomicU32,

    /// Change notification: single consumer, cross-executor safe.
    changed: Signal<CriticalSectionRawMutex, ()>,
}

impl<'d> Default for SharedControl<'d> {
    fn default() -> Self {
        SharedControl {
            audio_settings: CriticalSectionMutex::new(Cell::new(AudioSettings::default())),
            channels: &[],
            controls: (FeatureUnitControls::NONE, FeatureUnitControls::NONE),
            sample_rates_hz: &[],
            sample_rate_hz: AtomicU32::new(0),
            changed: Signal::new(),
        }
    }
}

impl<'d> SharedControl<'d> {
    /// The device's default sample rate: the first advertised one. It is what
    /// the host reads before setting any, and what a bus reset restores.
    fn default_sample_rate_hz(&self) -> u32 {
        self.sample_rates_hz.first().copied().unwrap_or(0)
    }

    /// The advertised controls for a logical channel: the master (0) has its
    /// own set, distinct from the audio channels'.
    fn controls(&self, channel_index: u8) -> FeatureUnitControls {
        if channel_index == 0 {
            self.controls.0
        } else {
            self.controls.1
        }
    }
}

/// Control status change monitor.
///
/// Await [`Self::changed`], then read the updated settings with [`Self::muted`], [`Self::volume`] and [`Self::sample_rate_hz`].
pub struct ControlMonitor<'d> {
    shared: &'d SharedControl<'d>,
}

impl<'d> ControlMonitor<'d> {
    fn audio_settings(&self) -> AudioSettings {
        self.shared.audio_settings.lock(|x| x.get())
    }

    fn get_logical_channel(&self, search_channel: Channel) -> Option<usize> {
        let index = self.shared.channels.iter().position(|&c| c == search_channel)?;

        // The logical channels start at one (zero is the master channel).
        Some(index + 1)
    }

    /// Whether the host has muted the stream: the master channel, or every
    /// audio channel. A stream that honours it plays or sends silence.
    pub fn muted(&self) -> bool {
        let settings = self.audio_settings();
        settings.muted[0]
            || settings.muted[1..=self.shared.channels.len()]
                .iter()
                .all(|&muted| muted)
    }

    /// Get the effective volume of a channel: [`Volume::Muted`] when muted or
    /// set to -∞ dB, otherwise the host-set level (default 0 dB). `None` if
    /// the channel does not exist or advertises no controls.
    pub fn volume(&self, channel: Channel) -> Option<Volume> {
        let channel_index = self.get_logical_channel(channel)?;
        let controls = self.shared.controls(channel_index as u8);
        if !controls.mute && !controls.volume {
            return None;
        }

        let settings = self.audio_settings();
        // The host silences a channel through the mute control or by setting
        // its volume to silence (-∞ dB) [UAC 5.2.2.4.3.2].
        if settings.muted[channel_index] || settings.volume_8q8_db[channel_index] == VOLUME_SILENCE_8Q8_DB {
            return Some(Volume::Muted);
        }

        Some(Volume::DeciBel(
            (settings.volume_8q8_db[channel_index] as f32) / (VOLUME_STEPS_PER_DB as f32),
        ))
    }

    /// Get the streaming endpoint's sample rate in Hz. Before the host sets
    /// one — and again after a bus reset — it is the first advertised rate.
    pub fn sample_rate_hz(&self) -> u32 {
        self.shared.sample_rate_hz.load(Ordering::Relaxed)
    }

    /// Wait for the control settings to change. Single consumer: await this
    /// from at most one task.
    pub async fn changed(&self) {
        self.shared.changed.wait().await;
    }
}

impl<'d> Control<'d> {
    fn changed(&mut self) {
        self.shared.changed.signal(());
    }

    /// Whether `channel_index` is the master or one of the stream's channels.
    fn has_channel(&self, channel_index: u8) -> bool {
        channel_index as usize <= self.shared.channels.len()
    }

    /// The logical channels a request's wChannelNumber addresses: one, or —
    /// for 0xFF — the master and every channel [UAC 5.2.1.2]. `None` for a
    /// channel the stream does not have.
    fn addressed_channels(&self, channel_index: u8) -> Option<RangeInclusive<usize>> {
        if channel_index == ALL_CHANNELS {
            Some(0..=self.shared.channels.len())
        } else if self.has_channel(channel_index) {
            Some(channel_index as usize..=channel_index as usize)
        } else {
            None
        }
    }

    fn interface_set_request(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        let interface_number = req.index as u8;
        let entity_index = (req.index >> 8) as u8;
        let channel_index = req.value as u8;
        let control_unit = (req.value >> 8) as u8;

        if interface_number != self.control_interface_number.into() {
            debug!("Unhandled interface set request for interface {}", interface_number);
            return None;
        }

        let channels = match self.addressed_channels(channel_index) {
            Some(channels) if entity_index == FEATURE_UNIT_ID && req.request == SET_CUR => channels,
            _ => {
                debug!(
                    "Unsupported interface set request {} for entity {} channel {}",
                    req.request, entity_index, channel_index
                );
                return Some(OutResponse::Rejected);
            }
        };

        // The parameter block holds one entry per addressed channel, the
        // master first for 0xFF [UAC 5.2.2.4].
        let entry_size = match control_unit {
            MUTE_CONTROL => 1,
            VOLUME_CONTROL => 2,
            _ => return Some(OutResponse::Rejected),
        };
        if data.len() < channels.clone().count() * entry_size {
            return Some(OutResponse::Rejected);
        }

        let accepted = self.shared.audio_settings.lock(|x| {
            let mut audio_settings = x.get();
            // Entries apply where the descriptor advertises the control; a
            // request addressing only unadvertised controls is rejected.
            let mut any_advertised = false;
            for (entry, channel) in channels.clone().enumerate() {
                let controls = self.shared.controls(channel as u8);
                match control_unit {
                    MUTE_CONTROL if controls.mute => {
                        audio_settings.muted[channel] = data[entry] != 0;
                        any_advertised = true;
                    }
                    VOLUME_CONTROL if controls.volume => {
                        let volume = i16::from_le_bytes([data[2 * entry], data[2 * entry + 1]]);
                        // CUR lies within the advertised range [UAC 5.2.2.4.2],
                        // except silence, which is always accepted.
                        if volume != VOLUME_SILENCE_8Q8_DB
                            && !(MIN_VOLUME_DB * VOLUME_STEPS_PER_DB..=MAX_VOLUME_DB * VOLUME_STEPS_PER_DB)
                                .contains(&volume)
                        {
                            return false;
                        }
                        audio_settings.volume_8q8_db[channel] = volume;
                        any_advertised = true;
                    }
                    _ => {}
                }
            }
            if any_advertised {
                x.set(audio_settings);
            }
            any_advertised
        });
        if !accepted {
            return Some(OutResponse::Rejected);
        }

        debug!(
            "Set feature unit control {} for channel {}",
            control_unit, channel_index
        );
        self.changed();

        Some(OutResponse::Accepted)
    }

    fn endpoint_set_request(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        let control_selector = (req.value >> 8) as u8;
        let endpoint_address = req.index as u8;

        if endpoint_address != self.streaming_endpoint_address {
            debug!(
                "Unhandled endpoint set request for endpoint {} and control {} with data {:?}",
                endpoint_address, control_selector, data
            );
            return None;
        }

        if req.request != SET_CUR || control_selector != SAMPLING_FREQ_CONTROL || data.len() < 3 {
            debug!(
                "Unsupported endpoint set request {} for control selector {}",
                req.request, control_selector
            );
            return Some(OutResponse::Rejected);
        }

        let sample_rate_hz = u32::from_le_bytes([data[0], data[1], data[2], 0]);
        if !self.shared.sample_rates_hz.contains(&sample_rate_hz) {
            debug!("Unsupported sample rate {} Hz", sample_rate_hz);
            return Some(OutResponse::Rejected);
        }
        self.shared.sample_rate_hz.store(sample_rate_hz, Ordering::Relaxed);

        debug!("Set endpoint {} sample rate to {} Hz", endpoint_address, sample_rate_hz);

        self.changed();

        Some(OutResponse::Accepted)
    }

    fn interface_get_request<'r>(&'r mut self, req: Request, buf: &'r mut [u8]) -> Option<InResponse<'r>> {
        let interface_number = req.index as u8;
        let entity_index = (req.index >> 8) as u8;
        let channel_index = req.value as u8;
        let control_unit = (req.value >> 8) as u8;

        if interface_number != self.control_interface_number.into() {
            debug!("Unhandled interface get request for interface {}.", interface_number);
            return None;
        }

        let channels = match self.addressed_channels(channel_index) {
            // Only this function's Feature Unit can be handled at the moment.
            Some(channels) if entity_index == FEATURE_UNIT_ID => channels,
            _ => {
                debug!(
                    "Unsupported interface get request for entity {} channel {}.",
                    entity_index, channel_index
                );
                return Some(InResponse::Rejected);
            }
        };
        let count = channels.clone().count();
        // At least one addressed channel must advertise the control
        // [UAC 5.2.1]; in a 0xFF block, the others report their fixed default.
        let mute = channels.clone().any(|channel| self.shared.controls(channel as u8).mute);
        let volume = channels
            .clone()
            .any(|channel| self.shared.controls(channel as u8).volume);

        match (req.request, control_unit) {
            (GET_CUR, MUTE_CONTROL) if mute && buf.len() >= count => {
                let audio_settings = self.shared.audio_settings.lock(|x| x.get());
                for (entry, channel) in channels.enumerate() {
                    buf[entry] = audio_settings.muted[channel].into();
                }
                debug!("Got channel {} mute state.", channel_index);
                Some(InResponse::Accepted(&buf[..count]))
            }
            (GET_CUR, VOLUME_CONTROL) if volume && buf.len() >= 2 * count => {
                let audio_settings = self.shared.audio_settings.lock(|x| x.get());
                for (entry, channel) in channels.enumerate() {
                    buf[2 * entry..2 * entry + 2].copy_from_slice(&audio_settings.volume_8q8_db[channel].to_le_bytes());
                }
                debug!("Got channel {} volume.", channel_index);
                Some(InResponse::Accepted(&buf[..2 * count]))
            }
            (GET_MIN | GET_MAX | GET_RES, VOLUME_CONTROL) if volume && buf.len() >= 2 * count => {
                let value = match req.request {
                    GET_MIN => MIN_VOLUME_DB * VOLUME_STEPS_PER_DB,
                    GET_MAX => MAX_VOLUME_DB * VOLUME_STEPS_PER_DB,
                    _ => VOLUME_STEPS_PER_DB,
                };
                for entry in 0..count {
                    buf[2 * entry..2 * entry + 2].copy_from_slice(&value.to_le_bytes());
                }
                Some(InResponse::Accepted(&buf[..2 * count]))
            }
            _ => Some(InResponse::Rejected),
        }
    }

    fn endpoint_get_request<'r>(&'r mut self, req: Request, buf: &'r mut [u8]) -> Option<InResponse<'r>> {
        let control_selector = (req.value >> 8) as u8;
        let endpoint_address = req.index as u8;

        if endpoint_address != self.streaming_endpoint_address {
            debug!("Unhandled endpoint get request for endpoint {}.", endpoint_address);
            return None;
        }

        if control_selector != SAMPLING_FREQ_CONTROL {
            debug!(
                "Unsupported endpoint get request for control selector {}.",
                control_selector
            );
            return Some(InResponse::Rejected);
        }

        // The rates are advertised as a discrete list, so only the current one
        // can be asked for; requests for unsupported attributes (GET_MIN,
        // GET_MAX, GET_RES) are stalled [UAC 5.2.1].
        if req.request != GET_CUR {
            debug!("Unsupported endpoint get request {}.", req.request);
            return Some(InResponse::Rejected);
        }

        let sample_rate_hz = self.shared.sample_rate_hz.load(Ordering::Relaxed);
        buf[..3].copy_from_slice(&sample_rate_hz.to_le_bytes()[..3]);

        Some(InResponse::Accepted(&buf[..3]))
    }
}

impl<'d> Handler for Control<'d> {
    /// Called after a USB reset after the bus reset sequence is complete.
    /// The device returns to its default state [USB 2.0 9.1.1.5]: settings
    /// and the sample rate revert to their defaults.
    fn reset(&mut self) {
        let shared = self.shared;
        shared.audio_settings.lock(|x| x.set(AudioSettings::default()));
        shared
            .sample_rate_hz
            .store(shared.default_sample_rate_hz(), Ordering::Relaxed);

        self.changed();
    }

    /// Called when the bus has entered or exited the suspend state.
    fn suspended(&mut self, suspended: bool) {
        debug!("USB device suspended: {}", suspended);
    }

    // Handle control set requests.
    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        match req.request_type {
            RequestType::Class => match req.recipient {
                Recipient::Interface => self.interface_set_request(req, data),
                Recipient::Endpoint => self.endpoint_set_request(req, data),
                _ => None,
            },
            _ => None,
        }
    }

    // Handle control get requests.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        match req.request_type {
            RequestType::Class => match req.recipient {
                Recipient::Interface => self.interface_get_request(req, buf),
                Recipient::Endpoint => self.endpoint_get_request(req, buf),
                _ => None,
            },
            _ => None,
        }
    }
}

/// An audio function [UAC 3]: Input Terminal → [Feature Unit] → Output Terminal, carrying a PCM
/// stream of `channels` at `sample_width` and any of `sample_rates_hz`. One terminal is the USB stream.
pub(super) struct AudioFunction<'a> {
    pub channels: &'a [Channel],
    pub sample_width: SampleWidth,
    pub sample_rates_hz: &'a [u32],
    /// What the Input Terminal is.
    pub input_terminal: TerminalType,
    /// What the Output Terminal is.
    pub output_terminal: TerminalType,
    /// The Feature Unit's controls on the master channel and on each
    /// channel, or `None` for no Feature Unit at all.
    pub feature_unit: Option<(FeatureUnitControls, FeatureUnitControls)>,
}

/// Unit ids, unique within a function.
const INPUT_UNIT_ID: u8 = 0x01;
const FEATURE_UNIT_ID: u8 = 0x02;
const OUTPUT_UNIT_ID: u8 = 0x03;

/// Every class-specific descriptor has a 2-byte header on top of its body.
const DESCRIPTOR_HEADER_SIZE: usize = 2;

impl<'d> AudioFunction<'d> {
    /// Build the whole audio function on `builder`: the IAD, the AudioControl and AudioStreaming
    /// interfaces, the streaming endpoint from `alloc_endpoint` and — with `feedback_refresh` —
    /// the feedback endpoint. Panics on an invalid configuration: the asserts spell out the limits.
    pub(super) fn build<D: Driver<'d>, E: Endpoint>(
        self,
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d>,
        max_packet_size: u16,
        feedback_refresh: Option<FeedbackRefresh>,
        alloc_endpoint: impl FnOnce(&mut InterfaceAltBuilder<'_, 'd, D>, u16) -> E,
    ) -> (E, Option<Feedback<'d, D>>, ControlMonitor<'d>) {
        assert!(
            (1..=MAX_SAMPLE_RATE_COUNT).contains(&self.sample_rates_hz.len()),
            "between one and ten sample rates"
        );
        assert!(
            (1..=MAX_AUDIO_CHANNEL_INDEX).contains(&self.channels.len()),
            "between one and twelve channels"
        );
        assert!(
            self.sample_rates_hz.iter().all(|rate| *rate <= MAX_SAMPLE_RATE_HZ),
            "sample rates are at most 24 bits"
        );
        // Isochronous data payloads are at most 1023 bytes at full speed
        // [USB 2.0 5.6.3], 1024 per transaction at high speed.
        assert!(max_packet_size <= 1024, "at most 1024 bytes per isochronous packet");
        assert!(
            (self.input_terminal == TerminalType::UsbStreaming) != (self.output_terminal == TerminalType::UsbStreaming),
            "exactly one terminal must be the USB stream"
        );
        // Room for the largest control request: an all-channels (0xFF) volume
        // block, one 16-bit entry for the master and each channel [UAC 5.2.2.4].
        assert!(
            builder.control_buf_len() >= 2 * (self.channels.len() + 1),
            "control_buf too small for this function's control requests"
        );

        // The IAD's class/subclass need not match its interfaces', but Microsoft recommends
        // that the first interface of the collection match the IAD.
        let mut func = builder.function(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE);

        // Audio control interface (mandatory) [UAC 4.3.1]
        let mut interface = func.interface();
        let control_interface = interface.interface_number();
        let streaming_interface = u8::from(control_interface) + 1;
        let mut alt = interface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE, None);

        self.write_ac_interface_descriptors(&mut alt, streaming_interface);

        // Audio streaming interface [UAC 4.5.1]: alternate setting 0 is zero-bandwidth
        // and has nothing in it; alternate setting 1 carries the stream.
        let mut interface = func.interface();
        interface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOSTREAMING_SUBCLASS, PROTOCOL_NONE, None);
        let mut alt = interface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOSTREAMING_SUBCLASS, PROTOCOL_NONE, None);

        self.write_as_interface_descriptors(&mut alt);

        let streaming_endpoint = alloc_endpoint(&mut alt, max_packet_size);
        // The USB-stream terminal decides the direction: streaming toward the
        // host (device output is the USB stream) takes an IN endpoint.
        assert!(
            streaming_endpoint.info().addr.is_in() == (self.output_terminal == TerminalType::UsbStreaming),
            "the endpoint direction must match the USB-stream terminal"
        );
        let feedback = feedback_refresh.map(|refresh| (Feedback::allocate(&mut alt), refresh));
        // The audio data endpoint's descriptors point at the synch endpoint; the synch endpoint's go after.
        Self::write_as_endpoint_descriptors(
            &mut alt,
            streaming_endpoint.info(),
            feedback.as_ref().map(|(feedback, _)| feedback),
        );
        if let Some((feedback, refresh)) = &feedback {
            feedback.write_descriptor(&mut alt, *refresh);
        }

        // Free up the builder: registering the handler needs it back.
        drop(func);

        let control_monitor = state.register(
            builder,
            self.channels,
            self.sample_rates_hz,
            self.feature_unit
                .unwrap_or((FeatureUnitControls::NONE, FeatureUnitControls::NONE)),
            control_interface,
            streaming_endpoint.info().addr.into(),
        );

        (
            streaming_endpoint,
            feedback.map(|(feedback, _)| feedback),
            control_monitor,
        )
    }
}

impl AudioFunction<'_> {
    /// Write the class-specific AudioControl interface descriptors [UAC 4.3.2]:
    /// the header, then each terminal and unit. `streaming_interface` is the
    /// AudioStreaming interface's number.
    fn write_ac_interface_descriptors<'d, D: Driver<'d>>(
        &self,
        alt: &mut InterfaceAltBuilder<'_, 'd, D>,
        streaming_interface: u8,
    ) {
        // Input Terminal Descriptor [UAC 4.3.2.1]
        let terminal_type: u16 = self.input_terminal.into();
        let channel_config = self.channel_config();
        let input_terminal = [
            INPUT_TERMINAL, // bDescriptorSubtype
            INPUT_UNIT_ID,  // bTerminalID
            terminal_type as u8,
            (terminal_type >> 8) as u8, // wTerminalType
            0x00,                       // bAssocTerminal (none)
            self.channels.len() as u8,  // bNrChannels
            channel_config as u8,
            (channel_config >> 8) as u8, // wChannelConfig
            0x00,                        // iChannelNames (none)
            0x00,                        // iTerminal (none)
        ];

        // Feature Unit Descriptor [UAC 4.3.2.5]
        let feature_unit = self.feature_unit.map(|(master, per_channel)| {
            let mut feature_unit: Vec<u8, { 5 + MAX_AUDIO_CHANNEL_COUNT + 1 }> = Vec::new();
            feature_unit
                .extend_from_slice(&[
                    FEATURE_UNIT,    // bDescriptorSubtype (Feature Unit)
                    FEATURE_UNIT_ID, // bUnitID
                    INPUT_UNIT_ID,   // bSourceID
                    1,               // bControlSize (one byte per control)
                    master.bitmap(), // Master controls
                ])
                .unwrap();
            for _channel in self.channels {
                feature_unit.push(per_channel.bitmap()).unwrap();
            }
            feature_unit.push(0x00).unwrap(); // iFeature (none)
            feature_unit
        });

        // Output Terminal Descriptor [UAC 4.3.2.2]
        let terminal_type: u16 = self.output_terminal.into();
        let output_terminal = [
            OUTPUT_TERMINAL, // bDescriptorSubtype
            OUTPUT_UNIT_ID,  // bTerminalID
            terminal_type as u8,
            (terminal_type >> 8) as u8, // wTerminalType
            0x00,                       // bAssocTerminal (none)
            // bSourceID: the feature unit, or the input terminal directly
            if self.feature_unit.is_some() {
                FEATURE_UNIT_ID
            } else {
                INPUT_UNIT_ID
            },
            0x00, // iTerminal (none)
        ];

        // Class-specific AC Interface Descriptor [UAC 4.3.2]; wTotalLength counts
        // itself and every unit, headers included.
        const HEADER_LEN: usize = 7;
        let total_length = [HEADER_LEN, input_terminal.len(), output_terminal.len()]
            .into_iter()
            .chain(feature_unit.as_ref().map(|feature_unit| feature_unit.len()))
            .map(|len| len + DESCRIPTOR_HEADER_SIZE)
            .sum::<usize>();
        let header: [u8; HEADER_LEN] = [
            HEADER_SUBTYPE, // bDescriptorSubtype (Header)
            ADC_VERSION as u8,
            (ADC_VERSION >> 8) as u8, // bcdADC
            total_length as u8,
            (total_length >> 8) as u8, // wTotalLength
            0x01,                      // bInCollection (1 streaming interface)
            streaming_interface,       // baInterfaceNr
        ];

        alt.descriptor(CS_INTERFACE, &header);
        alt.descriptor(CS_INTERFACE, &input_terminal);
        if let Some(feature_unit) = &feature_unit {
            alt.descriptor(CS_INTERFACE, feature_unit);
        }
        alt.descriptor(CS_INTERFACE, &output_terminal);
    }

    /// Write the class-specific AudioStreaming interface descriptors [UAC 4.5.2]:
    /// the general one, linking to the USB streaming terminal, and the format.
    fn write_as_interface_descriptors<'d, D: Driver<'d>>(&self, alt: &mut InterfaceAltBuilder<'_, 'd, D>) {
        // Class-specific AS Interface Descriptor: the stream connects to the USB terminal.
        let terminal_link = if self.input_terminal == TerminalType::UsbStreaming {
            INPUT_UNIT_ID
        } else {
            OUTPUT_UNIT_ID
        };
        alt.descriptor(
            CS_INTERFACE,
            &[
                AS_GENERAL,    // bDescriptorSubtype
                terminal_link, // bTerminalLink
                0x00,          // bDelay (none)
                PCM as u8,
                (PCM >> 8) as u8, // wFormatTag (PCM format)
            ],
        );

        // Type I Format Type Descriptor [UAC Formats 2.2.5]
        let mut format: Vec<u8, { 6 + 3 * MAX_SAMPLE_RATE_COUNT }> = Vec::from_slice(&[
            FORMAT_TYPE,                      // bDescriptorSubtype
            FORMAT_TYPE_I,                    // bFormatType
            self.channels.len() as u8,        // bNrChannels
            self.sample_width as u8,          // bSubframeSize
            self.sample_width.in_bit() as u8, // bBitResolution
            self.sample_rates_hz.len() as u8, // bSamFreqType (discrete)
        ])
        .unwrap();
        for sample_rate_hz in self.sample_rates_hz {
            format.extend_from_slice(&sample_rate_hz.to_le_bytes()[..3]).unwrap();
        }
        alt.descriptor(CS_INTERFACE, &format);
    }

    /// Write the isochronous audio data endpoint's descriptors: the standard
    /// one [UAC 4.6.1.1], pointing at the synch endpoint if there is one, and
    /// the class-specific one [UAC 4.6.1.2] with its sampling frequency control.
    fn write_as_endpoint_descriptors<'d, D: Driver<'d>>(
        alt: &mut InterfaceAltBuilder<'_, 'd, D>,
        endpoint: &EndpointInfo,
        feedback: Option<&Feedback<'d, D>>,
    ) {
        alt.endpoint_descriptor(
            endpoint,
            SynchronizationType::Asynchronous,
            UsageType::DataEndpoint,
            &[
                0x00,                                     // bRefresh (0)
                feedback.map_or(0x00, Feedback::address), // bSynchAddress (the feedback endpoint, or none)
            ],
        );
        alt.descriptor(
            CS_ENDPOINT,
            &[
                EP_GENERAL,               // bDescriptorSubtype (General)
                EP_CS_ATTR_SAMPLING_FREQ, // bmAttributes (support sampling frequency control)
                0x02,                     // bLockDelayUnits (PCM)
                0x00,
                0x00, // wLockDelay (0)
            ],
        );
    }

    /// The wChannelConfig bitmap. The host numbers the logical channels by
    /// bit order [UAC 3.7.2.3], so the stream's channels must come in that
    /// order; panics on a duplicate or out-of-order channel.
    fn channel_config(&self) -> u16 {
        let mut channel_config: u16 = ChannelConfig::None.into();
        for channel in self.channels {
            let channel: u16 = channel.get_channel_config().into();

            if channel <= channel_config {
                panic!("Channel {} is duplicated or out of wChannelConfig order.", channel);
            }
            channel_config |= channel;
        }
        channel_config
    }
}

/// The isochronous synch endpoint [UAC 4.6.2]: used for writing sample rate
/// feedback to the host.
pub struct Feedback<'d, D: Driver<'d>> {
    feedback_endpoint: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Feedback<'d, D> {
    /// Allocate the synch endpoint: isochronous IN, 24-bit packets.
    fn allocate(alt: &mut InterfaceAltBuilder<'_, 'd, D>) -> Self {
        Self {
            feedback_endpoint: alt.alloc_endpoint_in(
                EndpointType::Isochronous,
                None,
                4, // Feedback packets are 3 bytes (10.14 format) at full speed, 4 (16.16) at high speed.
                1,
            ),
        }
    }

    /// The endpoint's address, for the audio data endpoint's bSynchAddress.
    fn address(&self) -> u8 {
        self.feedback_endpoint.info().addr.into()
    }

    /// Write the endpoint's standard descriptor [UAC 4.6.2.1]. The class
    /// specification wants it after the audio data endpoint's, hence a
    /// separate step.
    fn write_descriptor(&self, alt: &mut InterfaceAltBuilder<'_, 'd, D>, refresh: FeedbackRefresh) {
        alt.endpoint_descriptor(
            self.feedback_endpoint.info(),
            SynchronizationType::NoSynchronization,
            UsageType::FeedbackEndpoint,
            &[
                refresh as u8, // bRefresh
                0x00,          // bSynchAddress (none)
            ],
        );
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.feedback_endpoint.write(data).await
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.feedback_endpoint.wait_enabled().await;
    }
}
