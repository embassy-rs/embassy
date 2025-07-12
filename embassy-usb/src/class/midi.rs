//! MIDI class implementation.

use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::Builder;

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
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> MidiClass<'d, D> {
    /// Creates a new `MidiClass` with the provided UsbBus, number of input and output jacks and `max_packet_size` in bytes.
    /// For full-speed devices, `max_packet_size` has to be one of 8, 16, 32 or 64.
    pub fn new(builder: &mut Builder<'d, D>, n_in_jacks: u8, n_out_jacks: u8, max_packet_size: u16) -> Self {
        let mut func = builder.function(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE);

        // Audio control interface
        let mut iface = func.interface();
        let audio_if = iface.interface_number();
        let midi_if = u8::from(audio_if) + 1;
        let mut alt = iface.alt_setting(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE, None);
        alt.descriptor(CS_INTERFACE, &[HEADER_SUBTYPE, 0x00, 0x01, 0x09, 0x00, 0x01, midi_if]);

        // MIDIStreaming interface
        let mut iface = func.interface();
        let _midi_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_AUDIO_CLASS, USB_MIDISTREAMING_SUBCLASS, PROTOCOL_NONE, None);

        let midi_streaming_total_length = 7
            + (n_in_jacks + n_out_jacks) as usize * (MIDI_IN_SIZE + MIDI_OUT_SIZE) as usize
            + 7
            + (4 + n_out_jacks as usize)
            + 7
            + (4 + n_in_jacks as usize);

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

        // Calculates the index'th external midi in jack id
        let in_jack_id_ext = |index| 2 * index + 1;
        // Calculates the index'th embedded midi out jack id
        let out_jack_id_emb = |index| 2 * index + 2;
        // Calculates the index'th external midi out jack id
        let out_jack_id_ext = |index| 2 * n_in_jacks + 2 * index + 1;
        // Calculates the index'th embedded midi in jack id
        let in_jack_id_emb = |index| 2 * n_in_jacks + 2 * index + 2;

        for i in 0..n_in_jacks {
            alt.descriptor(CS_INTERFACE, &[MIDI_IN_JACK_SUBTYPE, EXTERNAL, in_jack_id_ext(i), 0x00]);
        }

        for i in 0..n_out_jacks {
            alt.descriptor(CS_INTERFACE, &[MIDI_IN_JACK_SUBTYPE, EMBEDDED, in_jack_id_emb(i), 0x00]);
        }

        for i in 0..n_out_jacks {
            alt.descriptor(
                CS_INTERFACE,
                &[
                    MIDI_OUT_JACK_SUBTYPE,
                    EXTERNAL,
                    out_jack_id_ext(i),
                    0x01,
                    in_jack_id_emb(i),
                    0x01,
                    0x00,
                ],
            );
        }

        for i in 0..n_in_jacks {
            alt.descriptor(
                CS_INTERFACE,
                &[
                    MIDI_OUT_JACK_SUBTYPE,
                    EMBEDDED,
                    out_jack_id_emb(i),
                    0x01,
                    in_jack_id_ext(i),
                    0x01,
                    0x00,
                ],
            );
        }

        let mut endpoint_data = [
            MS_GENERAL, 0, // Number of jacks
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Jack mappings
        ];
        endpoint_data[1] = n_out_jacks;
        for i in 0..n_out_jacks {
            endpoint_data[2 + i as usize] = in_jack_id_emb(i);
        }
        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_out_jacks as usize]);

        endpoint_data[1] = n_in_jacks;
        for i in 0..n_in_jacks {
            endpoint_data[2 + i as usize] = out_jack_id_emb(i);
        }
        let write_ep = alt.endpoint_bulk_in(max_packet_size);
        alt.descriptor(CS_ENDPOINT, &endpoint_data[0..2 + n_in_jacks as usize]);

        MidiClass { read_ep, write_ep }
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Split the class into a sender and receiver.
    ///
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
            },
            Receiver { read_ep: self.read_ep },
        )
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
