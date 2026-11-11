//! MIDI class implementation.

use super::{
    CS_GR_TRM_BLOCK, GR_TRM_BLOCK_HEADER_LEN, GR_TRM_BLOCK_LEN, GroupTerminalBlock, GroupTerminalBlockConfig, MAX_GTBS,
    MAX_MIDI_JACKS, MAX_UMP_BYTES, MIDI_IN_JACK_LEN, MIDI_IN_JACK_SUBTYPE, MIDI_OUT_JACK_BASE_LEN,
    MIDI_OUT_JACK_SUBTYPE, MIDI_VERSION, MIDI2_VERSION, MS_GENERAL, MS_GENERAL_2_0, MS_GENERAL_BASE_LEN, MS_HEADER_LEN,
    MS_HEADER_SUBTYPE, MidiJackType, PROTOCOL_NONE, USB_AUDIO_CLASS, USB_MIDISTREAMING_SUBCLASS, UmpPacket, UmpReader,
    serialize_gtb_descriptors,
};
use crate::control::{InResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut, EndpointType};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, Handler};

const USB_AUDIOCONTROL_SUBCLASS: u8 = 0x01;
const CS_INTERFACE: u8 = 0x24;
const CS_ENDPOINT: u8 = 0x25;
const HEADER_SUBTYPE: u8 = 0x01;
const EMBEDDED: u8 = MidiJackType::Embedded as u8;
const EXTERNAL: u8 = MidiJackType::External as u8;
// Each OUT jack has one source pin.
const MIDI_OUT_JACK_LEN: u8 = MIDI_OUT_JACK_BASE_LEN + 2;

/// Configuration for MIDI 2.0 support (Alternate Setting 1).
///
/// The MIDI 2.0 alternate setting reuses the bulk endpoints allocated for the
/// MIDI 1.0 alternate setting, as required by the USB MIDI 2.0 specification.
/// The endpoints' packet size is therefore configured once through
/// [`MidiClassConfig::max_packet_size`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Midi2Config<'d> {
    /// Group Terminal Blocks declared by this interface.
    pub group_terminal_blocks: &'d [GroupTerminalBlockConfig<'d>],
}

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
    ///
    /// The same bulk endpoints are used by the MIDI 1.0 and MIDI 2.0 alternate
    /// settings, so this single value applies to both.
    /// For full-speed devices, the value has to be one of 8, 16, 32 or 64.
    pub max_packet_size: u16,

    /// Name of the MIDIStreaming interface.
    pub interface_name: Option<&'d str>,

    /// Name of each IN jack.
    pub in_jack_names: &'d [Option<&'d str>],

    /// Name of each OUT jack.
    pub out_jack_names: &'d [Option<&'d str>],

    /// Optional MIDI 2.0 configuration for Alternate Setting 1.
    ///
    /// When set, Alternate Setting 1 reuses the bulk endpoints of Alternate
    /// Setting 0 instead of allocating a second pair.
    pub midi2: Option<Midi2Config<'d>>,
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
            midi2: None,
        }
    }
}

/// Internal state for a [`MidiClass`].
///
/// Holds names/string-indices for interface, jacks, and Group Terminal Blocks,
/// and serves GTB descriptors over the control endpoint.
pub struct MidiClassState<'d> {
    first: u8,
    interface_name: Option<&'d str>,
    in_jack_names: &'d [Option<&'d str>],
    out_jack_names: &'d [Option<&'d str>],
    gtb_names: [Option<&'d str>; MAX_GTBS],
    gtb_count: usize,
    interface_number: u8,
    active_alt: u8,
    gtb_descriptors: [u8; GR_TRM_BLOCK_HEADER_LEN as usize + MAX_GTBS * GR_TRM_BLOCK_LEN as usize],
    gtb_len: usize,
}

impl Default for MidiClassState<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> MidiClassState<'d> {
    /// Creates a new `State`.
    pub const fn new() -> Self {
        MidiClassState {
            first: 0,
            interface_name: None,
            in_jack_names: &[],
            out_jack_names: &[],
            gtb_names: [None; MAX_GTBS],
            gtb_count: 0,
            interface_number: 0,
            active_alt: 0,
            gtb_descriptors: [0u8; GR_TRM_BLOCK_HEADER_LEN as usize + MAX_GTBS * GR_TRM_BLOCK_LEN as usize],
            gtb_len: 0,
        }
    }

    /// The names, in allocation order: interface, IN jacks, OUT jacks, GTB names.
    fn names(&self) -> impl Iterator<Item = &Option<&'d str>> {
        core::iter::once(&self.interface_name)
            .chain(self.in_jack_names)
            .chain(self.out_jack_names)
            .chain(self.gtb_names[..self.gtb_count].iter())
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

    /// Returns the currently active alternate setting (0 for MIDI 1.0, 1 for MIDI 2.0).
    pub fn active_alternate_setting(&self) -> u8 {
        self.active_alt
    }

    /// Returns whether MIDI 2.0 (Alternate Setting 1) is currently active.
    pub fn is_midi2_active(&self) -> bool {
        self.active_alt == 1
    }
}

impl Handler for MidiClassState<'_> {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        self.name(index.0.checked_sub(self.first)? as usize)
    }

    fn set_alternate_setting(&mut self, iface: InterfaceNumber, alternate_setting: u8) {
        if iface.0 == self.interface_number {
            self.active_alt = alternate_setting;
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, _buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type == RequestType::Class || req.request_type == RequestType::Standard)
            && req.recipient == Recipient::Interface
            && req.request == Request::GET_DESCRIPTOR
            && (req.value >> 8) == CS_GR_TRM_BLOCK as u16
            && (req.index as u8) == self.interface_number
        {
            if self.gtb_len > 0 {
                Some(InResponse::Accepted(&self.gtb_descriptors[..self.gtb_len]))
            } else {
                Some(InResponse::Rejected)
            }
        } else {
            None
        }
    }
}

/// Packet level implementation of a USB MIDI device supporting MIDI 1.0 and 2.0.
///
/// The MIDI 1.0 (Alternate Setting 0) and MIDI 2.0 (Alternate Setting 1)
/// protocols share one pair of bulk endpoints; the host selects which protocol
/// is carried over them with `SET_INTERFACE`.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like port.
pub struct MidiClass<'d, D: Driver<'d>> {
    read_ep: Option<D::EndpointOut>,
    write_ep: Option<D::EndpointIn>,
}

impl<'d, D: Driver<'d>> MidiClass<'d, D> {
    /// Creates a new `MidiClass` with the provided UsbBuilder and configuration.
    ///
    /// The names in `config` are ignored, use [`MidiClass::new_with_names`] if you need to name jacks
    /// or configure MIDI 2.0.
    pub fn new(builder: &mut Builder<'d, D>, config: MidiClassConfig<'d>) -> Self {
        assert!(
            config.midi2.is_none(),
            "MidiClass::new_with_names must be used when MIDI 2.0 is configured"
        );
        Self::build(builder, config, &mut MidiClassState::new())
    }

    /// Creates a new `MidiClass` with the provided UsbBuilder that names its interface, jacks,
    /// and Group Terminal Blocks.
    pub fn new_with_names(
        builder: &mut Builder<'d, D>,
        state: &'d mut MidiClassState<'d>,
        config: MidiClassConfig<'d>,
    ) -> Self {
        let mut gtb_names = [None; MAX_GTBS];
        let mut gtb_count = 0;
        let mut gtbs = heapless::Vec::<GroupTerminalBlock, MAX_GTBS>::new();

        if let Some(midi2_config) = &config.midi2 {
            for (i, b) in midi2_config.group_terminal_blocks.iter().enumerate().take(MAX_GTBS) {
                gtb_names[i] = b.name;
                gtb_count += 1;
            }
        }

        *state = MidiClassState {
            first: builder.string().0,
            interface_name: config.interface_name,
            in_jack_names: config.in_jack_names,
            out_jack_names: config.out_jack_names,
            gtb_names,
            gtb_count,
            interface_number: 0,
            active_alt: 0,
            gtb_descriptors: [0u8; GR_TRM_BLOCK_HEADER_LEN as usize + MAX_GTBS * GR_TRM_BLOCK_LEN as usize],
            gtb_len: 0,
        };

        // string index for interface allocated above, now allocate jacks and GTB strings.
        for _ in 1..state.names().count() {
            builder.string();
        }

        // Compute string indices and serialize GTB descriptors
        if let Some(midi2_config) = &config.midi2 {
            let gtb_str_start_offset = 1 + config.in_jack_names.len() + config.out_jack_names.len();
            for (i, b) in midi2_config.group_terminal_blocks.iter().enumerate() {
                let string_idx = if b.name.is_some() {
                    state.first + (gtb_str_start_offset + i) as u8
                } else {
                    0
                };
                let _ = gtbs.push(GroupTerminalBlock {
                    block_id: b.block_id,
                    block_type: b.block_type,
                    first_group: b.first_group,
                    num_groups: b.num_groups,
                    string_index: string_idx,
                    protocol: b.protocol,
                    max_input_bandwidth: b.max_input_bandwidth,
                    max_output_bandwidth: b.max_output_bandwidth,
                });
            }
            if let Ok(len) = serialize_gtb_descriptors(&gtbs, &mut state.gtb_descriptors) {
                state.gtb_len = len;
            }
        }

        let class = Self::build(builder, config, state);
        builder.handler(state);
        class
    }

    /// Creates a new `MidiClass` with the provided UsbBuilder and state.
    pub fn new_with_state(
        builder: &mut Builder<'d, D>,
        state: &'d mut MidiClassState<'d>,
        config: MidiClassConfig<'d>,
    ) -> Self {
        Self::new_with_names(builder, state, config)
    }

    /// Creates a new `MidiClass` with the provided UsbBuilder and configuration.
    fn build(builder: &mut Builder<'d, D>, config: MidiClassConfig<'d>, names: &mut MidiClassState<'d>) -> Self {
        let MidiClassConfig {
            n_in_jacks,
            n_out_jacks,
            max_packet_size,
            ..
        } = config;

        // Some sanity checks.
        assert!(
            n_in_jacks != 0 || n_out_jacks != 0 || config.midi2.is_some(),
            "n_in_jacks and n_out_jacks are both 0 and MIDI 2.0 is not configured"
        );
        assert!(
            n_in_jacks as usize <= MAX_MIDI_JACKS,
            "n_in_jacks is larger than {}",
            MAX_MIDI_JACKS,
        );
        assert!(
            n_out_jacks as usize <= MAX_MIDI_JACKS,
            "n_out_jacks is larger than {}",
            MAX_MIDI_JACKS,
        );

        let mut func = builder.function(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE);

        let has_midi2 = config.midi2.is_some();

        // The MIDI 2.0 alternate setting reuses the bulk endpoints of the MIDI
        // 1.0 alternate setting instead of allocating a second pair. Allocate
        // them once at function scope and write their standard descriptor into
        // every alternate setting that uses them.
        //
        // An endpoint is only described in Alt 0 when the corresponding jack
        // count is non-zero, but it is still allocated when MIDI 2.0 needs it,
        // so both alternate settings always reference the same addresses.
        let read_ep = (n_out_jacks > 0 || has_midi2)
            .then(|| func.alloc_endpoint_out(EndpointType::Bulk, None, max_packet_size, 0));
        let write_ep =
            (n_in_jacks > 0 || has_midi2).then(|| func.alloc_endpoint_in(EndpointType::Bulk, None, max_packet_size, 0));

        // Audio control interface
        let mut iface = func.interface();
        let audio_if = iface.interface_number();
        let midi_if = u8::from(audio_if) + 1;
        let mut alt = iface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE, None);
        alt.descriptor(CS_INTERFACE, &[HEADER_SUBTYPE, 0x00, 0x01, 0x09, 0x00, 0x01, midi_if]);

        // MIDIStreaming interface
        let mut iface = func.interface();
        let midi_if_num = iface.interface_number();
        names.interface_number = midi_if_num.0;

        let mut alt = iface.alt_setting(
            USB_AUDIO_CLASS,
            USB_MIDISTREAMING_SUBCLASS,
            PROTOCOL_NONE,
            names.interface(),
        );

        if n_in_jacks > 0 || n_out_jacks > 0 {
            let midi_streaming_total_length = MS_HEADER_LEN as usize
                + (n_in_jacks + n_out_jacks) as usize * (MIDI_IN_JACK_LEN + MIDI_OUT_JACK_LEN) as usize
                + if n_out_jacks > 0 {
                    9 + (MS_GENERAL_BASE_LEN as usize + n_out_jacks as usize)
                } else {
                    0
                }
                + if n_in_jacks > 0 {
                    9 + (MS_GENERAL_BASE_LEN as usize + n_in_jacks as usize)
                } else {
                    0
                };
            alt.descriptors_then_patch(
                |alt| {
                    alt.descriptor(
                        CS_INTERFACE,
                        &[
                            MS_HEADER_SUBTYPE,
                            MIDI_VERSION.to_le_bytes()[0],
                            MIDI_VERSION.to_le_bytes()[1],
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

                    if n_out_jacks > 0 {
                        endpoint_data[1] = n_out_jacks;
                        for i in 0..n_out_jacks {
                            endpoint_data[2 + i as usize] = in_jack_id_emb(i);
                        }
                        alt.endpoint_descriptor(
                            read_ep.as_ref().unwrap().info(),
                            SynchronizationType::NoSynchronization,
                            UsageType::DataEndpoint,
                            &[0, 0],
                        );
                        alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_out_jacks as usize]);
                    }

                    if n_in_jacks > 0 {
                        endpoint_data[1] = n_in_jacks;
                        for i in 0..n_in_jacks {
                            endpoint_data[2 + i as usize] = out_jack_id_emb(i);
                        }
                        alt.endpoint_descriptor(
                            write_ep.as_ref().unwrap().info(),
                            SynchronizationType::NoSynchronization,
                            UsageType::DataEndpoint,
                            &[0, 0],
                        );
                        alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_in_jacks as usize]);
                    }
                },
                |buffer| {
                    let len = buffer.len() as u16;
                    buffer[5..7].copy_from_slice(&len.to_le_bytes());
                },
            );
        } else {
            alt.descriptor(
                CS_INTERFACE,
                &[
                    MS_HEADER_SUBTYPE,
                    MIDI_VERSION.to_le_bytes()[0],
                    MIDI_VERSION.to_le_bytes()[1],
                    0x07,
                    0x00,
                ],
            );
        }

        // Alt 1: MIDI 2.0 (UMP)
        if let Some(midi2_config) = &config.midi2 {
            let mut alt1 = iface.alt_setting(
                USB_AUDIO_CLASS,
                USB_MIDISTREAMING_SUBCLASS,
                PROTOCOL_NONE,
                names.interface(),
            );

            // Alt 1 CS Header (Release 2.0, wTotalLength = 7)
            alt1.descriptor(
                CS_INTERFACE,
                &[
                    MS_HEADER_SUBTYPE,
                    (MIDI2_VERSION & 0xFF) as u8,
                    ((MIDI2_VERSION >> 8) & 0xFF) as u8,
                    0x07,
                    0x00,
                ],
            );

            let mut ep_cs_data = [0u8; 2 + MAX_GTBS];
            ep_cs_data[0] = MS_GENERAL_2_0;
            ep_cs_data[1] = midi2_config.group_terminal_blocks.len() as u8;
            for (i, b) in midi2_config.group_terminal_blocks.iter().enumerate() {
                ep_cs_data[2 + i] = b.block_id;
            }
            let ep_cs_len = 2 + midi2_config.group_terminal_blocks.len();

            // Reuse the bulk endpoints allocated for Alt 0. Only the
            // class-specific endpoint descriptor differs between the two
            // alternate settings.
            alt1.endpoint_descriptor(
                read_ep.as_ref().unwrap().info(),
                SynchronizationType::NoSynchronization,
                UsageType::DataEndpoint,
                &[0, 0],
            );
            alt1.descriptor(CS_ENDPOINT, &ep_cs_data[..ep_cs_len]);

            alt1.endpoint_descriptor(
                write_ep.as_ref().unwrap().info(),
                SynchronizationType::NoSynchronization,
                UsageType::DataEndpoint,
                &[0, 0],
            );
            alt1.descriptor(CS_ENDPOINT, &ep_cs_data[..ep_cs_len]);
        }

        MidiClass { read_ep, write_ep }
    }

    /// Gets the maximum packet size in bytes for the bulk endpoints.
    ///
    /// The MIDI 1.0 and MIDI 2.0 alternate settings share the same endpoints,
    /// so they have the same maximum packet size.
    pub fn max_packet_size(&self) -> u16 {
        if let Some(read_ep) = &self.read_ep {
            read_ep.info().max_packet_size
        } else if let Some(write_ep) = &self.write_ep {
            write_ep.info().max_packet_size
        } else {
            0
        }
    }

    /// Writes a single packet into the MIDI 1.0 IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let write_ep = self.write_ep.as_mut().ok_or(EndpointError::Disabled)?;
        write_ep.write(data).await
    }

    /// Reads a single packet from the MIDI 1.0 OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        let read_ep = self.read_ep.as_mut().ok_or(EndpointError::Disabled)?;
        read_ep.read(data).await
    }

    /// Writes a single Universal MIDI Packet (UMP) to the host.
    ///
    /// This is only meaningful while Alternate Setting 1 (MIDI 2.0) is active.
    pub async fn write_ump(&mut self, packet: &UmpPacket) -> Result<(), EndpointError> {
        let write_ep = self.write_ep.as_mut().ok_or(EndpointError::Disabled)?;
        let mut buf = [0u8; MAX_UMP_BYTES];
        let n = packet.to_bytes(&mut buf).map_err(|_| EndpointError::BufferOverflow)?;
        write_ep.write(&buf[..n]).await
    }

    /// Writes raw UMP bytes to the host.
    ///
    /// This is only meaningful while Alternate Setting 1 (MIDI 2.0) is active.
    pub async fn write_ump_bytes(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let write_ep = self.write_ep.as_mut().ok_or(EndpointError::Disabled)?;
        write_ep.write(data).await
    }

    /// Reads Universal MIDI Packets from the host.
    ///
    /// This is only meaningful while Alternate Setting 1 (MIDI 2.0) is active.
    pub async fn read_ump<'a>(&mut self, data: &'a mut [u8]) -> Result<UmpReader<'a>, EndpointError> {
        let read_ep = self.read_ep.as_mut().ok_or(EndpointError::Disabled)?;
        let n = read_ep.read(data).await?;
        Ok(UmpReader::new(&data[..n]))
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        if let Some(read_ep) = &mut self.read_ep {
            read_ep.wait_enabled().await;
        } else if let Some(write_ep) = &mut self.write_ep {
            write_ep.wait_enabled().await;
        }
    }

    /// Split the endpoints into a sender and receiver.
    ///
    /// The same endpoints carry both MIDI 1.0 and MIDI 2.0 data, so a single
    /// sender/receiver pair serves both alternate settings.
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(mut self) -> (Option<Sender<'d, D>>, Option<Receiver<'d, D>>) {
        let sender = self.write_ep.take().map(|write_ep| Sender { write_ep });
        let receiver = self.read_ep.take().map(|read_ep| Receiver { read_ep });
        (sender, receiver)
    }
}

/// Midi class packet sender.
///
/// You can obtain a `Sender` with [`MidiClass::split`].
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.write_ep.info().max_packet_size
    }

    /// Writes a single packet.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Writes a single Universal MIDI Packet (UMP).
    pub async fn write_ump(&mut self, packet: &UmpPacket) -> Result<(), EndpointError> {
        let mut buf = [0u8; MAX_UMP_BYTES];
        let n = packet.to_bytes(&mut buf).map_err(|_| EndpointError::BufferOverflow)?;
        self.write_ep.write(&buf[..n]).await
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await;
    }
}

/// Midi class packet receiver.
///
/// You can obtain a `Receiver` with [`MidiClass::split`].
pub struct Receiver<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.read_ep.info().max_packet_size
    }

    /// Reads a single packet.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Reads Universal MIDI Packets, returning a `UmpReader`.
    pub async fn read_ump<'a>(&mut self, data: &'a mut [u8]) -> Result<UmpReader<'a>, EndpointError> {
        let n = self.read_ep.read(data).await?;
        Ok(UmpReader::new(&data[..n]))
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_class_state_alternate_setting() {
        let mut state = MidiClassState::new();
        state.interface_number = 1;
        assert_eq!(state.active_alternate_setting(), 0);
        assert!(!state.is_midi2_active());

        state.set_alternate_setting(InterfaceNumber(1), 1);
        assert_eq!(state.active_alternate_setting(), 1);
        assert!(state.is_midi2_active());

        state.set_alternate_setting(InterfaceNumber(1), 0);
        assert_eq!(state.active_alternate_setting(), 0);
        assert!(!state.is_midi2_active());

        // Alternate setting for another interface should be ignored
        state.set_alternate_setting(InterfaceNumber(2), 1);
        assert_eq!(state.active_alternate_setting(), 0);
        assert!(!state.is_midi2_active());
    }
}
