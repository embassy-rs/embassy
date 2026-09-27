//! MIDI class implementation.

use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut, EndpointType};
use crate::types::StringIndex;
use crate::{Builder, Handler};

/// This should be used as `device_class` when building the `UsbDevice`.
pub const USB_AUDIO_CLASS: u8 = 0x01;

const USB_AUDIOCONTROL_SUBCLASS: u8 = 0x01;
const USB_MIDISTREAMING_SUBCLASS: u8 = 0x03;
const MIDI_IN_JACK_SUBTYPE: u8 = 0x02;
const MIDI_OUT_JACK_SUBTYPE: u8 = 0x03;
const EMBEDDED: u8 = 0x01;
const EXTERNAL: u8 = 0x02;
const CS_INTERFACE: u8 = 0x24;
const CS_ENDPOINT: u8 = 0x25;
const HEADER_SUBTYPE: u8 = 0x01;
const MS_HEADER_SUBTYPE: u8 = 0x01;
const MS_GENERAL: u8 = 0x01;
const PROTOCOL_NONE: u8 = 0x00;
const MIDI_IN_SIZE: u8 = 0x06;
const MIDI_OUT_SIZE: u8 = 0x09;
const MAX_MIDI_JACKS: u8 = 16;

/// Configuration for the MIDI class.
///
/// For the jacks, the field names use the terminology used by the USB specification,
/// which defines the direction from the perspective of the host.
///
/// This struct also implements the `Default` trait with the most common setup:
/// 1 input jack, 1 output jack and a maximum packet size of 64.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct MidiClassConfig<'d> {
    /// Number of jacks for sending data to the host via the IN endpoint.
    /// If set to 0, the IN endpoint will not be allocated.
    pub n_in_jacks: u8,

    /// Number of jacks for receiving data from the host via the OUT endpoint.
    /// If set to 0, the OUT endpoint will not be allocated.
    pub n_out_jacks: u8,

    /// Maximum packet size for the endpoints.
    /// For full-speed devices, the value has to be one of 8, 16, 32 or 64.
    pub max_packet_size: u16,

    /// Name of the MIDIStreaming interface.
    pub interface_name: Option<&'d str>,

    /// Name of each IN jack.
    pub in_jack_names: &'d [Option<&'d str>],

    /// Name of each OUT jack.
    pub out_jack_names: &'d [Option<&'d str>],
}

impl Default for MidiClassConfig<'_> {
    fn default() -> Self {
        Self {
            n_in_jacks: 1,
            n_out_jacks: 1,
            max_packet_size: 64,
            interface_name: None,
            in_jack_names: &[],
            out_jack_names: &[],
        }
    }
}

/// Internal state for a [`MidiClass`].
///
/// Holds names/string-indices for interface and jacks.
#[derive(Default)]
pub struct MidiClassState<'d> {
    first: u8,
    interface_name: Option<&'d str>,
    in_jack_names: &'d [Option<&'d str>],
    out_jack_names: &'d [Option<&'d str>],
}

impl<'d> MidiClassState<'d> {
    /// Creates a new `State`.
    pub const fn new() -> Self {
        MidiClassState {
            first: 0,
            interface_name: None,
            in_jack_names: &[],
            out_jack_names: &[],
        }
    }

    /// The names, in allocation order: interface, IN jacks, OUT jacks.
    fn names(&self) -> impl Iterator<Item = &Option<&'d str>> {
        core::iter::once(&self.interface_name)
            .chain(self.in_jack_names)
            .chain(self.out_jack_names)
    }

    /// The name in slot `n`, if there is one.
    fn name(&self, n: usize) -> Option<&'d str> {
        self.names().nth(n).copied().flatten()
    }

    // returns absolute StringIndex, or None if name is absent (or out of range).
    fn index(&self, n: usize) -> Option<StringIndex> {
        self.name(n).map(|_| StringIndex(self.first + n as u8))
    }

    fn id(&self, n: usize) -> u8 {
        self.index(n).map_or(0, |i| i.0)
    }

    fn interface(&self) -> Option<StringIndex> {
        self.index(0)
    }

    fn in_jack(&self, i: u8) -> u8 {
        self.id(1 + i as usize)
    }

    fn out_jack(&self, i: u8) -> u8 {
        self.id(1 + self.in_jack_names.len() + i as usize)
    }
}

impl Handler for MidiClassState<'_> {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        self.name(index.0.checked_sub(self.first)? as usize)
    }
}

/// Packet level implementation of a USB MIDI device.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like port.
/// The following constraints must be followed if you use this class directly:
///
/// - `read_packet` must be called with a buffer large enough to hold `max_packet_size` bytes.
/// - `write_packet` must not be called with a buffer larger than `max_packet_size` bytes.
/// - If you write a packet that is exactly `max_packet_size` bytes long, it won't be processed by the
///   host operating system until a subsequent shorter packet is sent. A zero-length packet (ZLP)
///   can be sent if there is no other data to send. This is because USB bulk transactions must be
///   terminated with a short packet, even if the bulk endpoint is used for stream-like data.
pub struct MidiClass<'d, D: Driver<'d>> {
    read_ep: Option<D::EndpointOut>,
    write_ep: Option<D::EndpointIn>,
}

impl<'d, D: Driver<'d>> MidiClass<'d, D> {
    /// Creates a new `MidiClass` with the provided UsbBuilder and configuration.
    ///
    /// The names in `config` are ignored, use [`MidiClass::new_with_names`] if you need to name jacks.
    pub fn new(builder: &mut Builder<'d, D>, config: MidiClassConfig<'d>) -> Self {
        Self::build(builder, config, &MidiClassState::new())
    }

    /// Creates a new `MidiClass` with the provided UsbBuilder that names its interface and jacks.
    pub fn new_with_names(
        builder: &mut Builder<'d, D>,
        state: &'d mut MidiClassState<'d>,
        config: MidiClassConfig<'d>,
    ) -> Self {
        *state = MidiClassState {
            first: builder.string().0,
            interface_name: config.interface_name,
            in_jack_names: config.in_jack_names,
            out_jack_names: config.out_jack_names,
        };

        // string index for interface allocated above, now allocate the jacks.
        for _ in 1..state.names().count() {
            builder.string();
        }

        let class = Self::build(builder, config, state);
        builder.handler(state);
        class
    }

    /// Creates a new `MidiClass` with the provided UsbBuilder and configuration.
    fn build(builder: &mut Builder<'d, D>, config: MidiClassConfig<'d>, names: &MidiClassState<'d>) -> Self {
        let MidiClassConfig {
            n_in_jacks,
            n_out_jacks,
            max_packet_size,
            ..
        } = config;

        // Some sanity checks.
        assert!(
            n_in_jacks != 0 || n_out_jacks != 0,
            "n_in_jacks and n_out_jacks are both 0"
        );
        assert!(
            n_in_jacks <= MAX_MIDI_JACKS,
            "n_in_jacks is larger than {}",
            MAX_MIDI_JACKS,
        );
        assert!(
            n_out_jacks <= MAX_MIDI_JACKS,
            "n_out_jacks is larger than {}",
            MAX_MIDI_JACKS,
        );

        let mut func = builder.function(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE);

        // Audio control interface
        let mut iface = func.interface();
        let audio_if = iface.interface_number();
        let midi_if = u8::from(audio_if) + 1;
        let mut alt = iface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE, None);
        alt.descriptor(CS_INTERFACE, &[HEADER_SUBTYPE, 0x00, 0x01, 0x09, 0x00, 0x01, midi_if]);

        // MIDIStreaming interface
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(
            USB_AUDIO_CLASS,
            USB_MIDISTREAMING_SUBCLASS,
            PROTOCOL_NONE,
            names.interface(),
        );

        let midi_streaming_total_length = 7
            + (n_in_jacks + n_out_jacks) as usize * (MIDI_IN_SIZE + MIDI_OUT_SIZE) as usize
            + if n_out_jacks > 0 {
                9 + (4 + n_out_jacks as usize)
            } else {
                0
            }
            + if n_in_jacks > 0 {
                9 + (4 + n_in_jacks as usize)
            } else {
                0
            };

        alt.descriptor(
            CS_INTERFACE,
            &[
                MS_HEADER_SUBTYPE,
                0x00,
                0x01,
                (midi_streaming_total_length & 0xFF) as u8,
                ((midi_streaming_total_length >> 8) & 0xFF) as u8,
            ],
        );

        // Calculates the index'th embedded midi out jack id
        let out_jack_id_emb = |index| 2 * index + 1;
        // Calculates the index'th external midi in jack id
        let in_jack_id_ext = |index| 2 * index + 2;
        // Calculates the index'th embedded midi in jack id
        let in_jack_id_emb = |index| 2 * n_in_jacks + 2 * index + 1;
        // Calculates the index'th external midi out jack id
        let out_jack_id_ext = |index| 2 * n_in_jacks + 2 * index + 2;

        for i in 0..n_in_jacks {
            let i_jack = names.in_jack(i);
            alt.descriptor(
                CS_INTERFACE,
                &[
                    MIDI_OUT_JACK_SUBTYPE,
                    EMBEDDED,
                    out_jack_id_emb(i),
                    0x01,
                    in_jack_id_ext(i),
                    0x01,
                    i_jack,
                ],
            );
            alt.descriptor(
                CS_INTERFACE,
                &[MIDI_IN_JACK_SUBTYPE, EXTERNAL, in_jack_id_ext(i), i_jack],
            );
        }

        for i in 0..n_out_jacks {
            let i_jack = names.out_jack(i);
            alt.descriptor(
                CS_INTERFACE,
                &[MIDI_IN_JACK_SUBTYPE, EMBEDDED, in_jack_id_emb(i), i_jack],
            );
            alt.descriptor(
                CS_INTERFACE,
                &[
                    MIDI_OUT_JACK_SUBTYPE,
                    EXTERNAL,
                    out_jack_id_ext(i),
                    0x01,
                    in_jack_id_emb(i),
                    0x01,
                    i_jack,
                ],
            );
        }

        let mut endpoint_data = [
            MS_GENERAL, 0, // Number of jacks
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Jack mappings
        ];

        let read_ep = if n_out_jacks > 0 {
            endpoint_data[1] = n_out_jacks;
            for i in 0..n_out_jacks {
                endpoint_data[2 + i as usize] = in_jack_id_emb(i);
            }
            let read_ep = alt.endpoint_out(
                EndpointType::Bulk,
                None,
                max_packet_size,
                0,
                SynchronizationType::NoSynchronization,
                UsageType::DataEndpoint,
                &[0, 0],
            );
            alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_out_jacks as usize]);
            Some(read_ep)
        } else {
            None
        };

        let write_ep = if n_in_jacks > 0 {
            endpoint_data[1] = n_in_jacks;
            for i in 0..n_in_jacks {
                endpoint_data[2 + i as usize] = out_jack_id_emb(i);
            }
            let write_ep = alt.endpoint_in(
                EndpointType::Bulk,
                None,
                max_packet_size,
                0,
                SynchronizationType::NoSynchronization,
                UsageType::DataEndpoint,
                &[0, 0],
            );
            alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_in_jacks as usize]);
            Some(write_ep)
        } else {
            None
        };

        MidiClass { read_ep, write_ep }
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        if let Some(read_ep) = &self.read_ep {
            read_ep.info().max_packet_size
        } else if let Some(write_ep) = &self.write_ep {
            write_ep.info().max_packet_size
        } else {
            0
        }
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let write_ep = self.write_ep.as_mut().ok_or(EndpointError::Disabled)?;
        write_ep.write(data).await
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        let read_ep = self.read_ep.as_mut().ok_or(EndpointError::Disabled)?;
        read_ep.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        if let Some(read_ep) = &mut self.read_ep {
            read_ep.wait_enabled().await;
        }
    }

    /// Split the class into a sender and receiver.
    ///
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(mut self) -> (Option<Sender<'d, D>>, Option<Receiver<'d, D>>) {
        let sender = self.write_ep.take().map(|write_ep| Sender { write_ep });
        let receiver = self.read_ep.take().map(|read_ep| Receiver { read_ep });
        (sender, receiver)
    }
}

/// Midi class packet sender.
///
/// You can obtain a `Sender` with [`MidiClass::split`]
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.write_ep.info().max_packet_size
    }

    /// Writes a single packet.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await;
    }
}

/// Midi class packet receiver.
///
/// You can obtain a `Receiver` with [`MidiClass::split`]
pub struct Receiver<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Reads a single packet.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }
}
