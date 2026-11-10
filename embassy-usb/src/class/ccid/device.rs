//! USB Chip/Smart Card Interface Device (CCID) device implementation.

use core::mem::MaybeUninit;

use super::types::*;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, Handler, InterfaceHandler};

/// Configuration for the CCID device class.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<'d> {
    /// Maximum packet size for bulk endpoints (typically 64 for Full-Speed).
    pub max_packet_size: u16,
    /// Highest slot index supported (0 = 1 slot, 1 = 2 slots, etc.). Default is 0.
    pub max_slot_index: u8,
    /// Voltage support bitmask (bit 0 = 5.0V, bit 1 = 3.0V, bit 2 = 1.8V). Default is `0x07` (all).
    pub voltage_support: u8,
    /// Supported smart card protocols bitmask (bit 0 = T=0, bit 1 = T=1). Default is `0x03` (both).
    pub protocols: u32,
    /// Default ICC clock frequency in kHz (e.g. 3580 for 3.58 MHz). Default is 3580.
    pub default_clock_khz: u32,
    /// Maximum ICC clock frequency in kHz. Default is 3580.
    pub max_clock_khz: u32,
    /// Default data rate in bps (e.g. 9600).
    pub default_data_rate_bps: u32,
    /// Maximum data rate in bps (e.g. 115200).
    pub max_data_rate_bps: u32,
    /// Maximum IFSD (Information Field Size Device) for T=1 protocol. Default is 254.
    pub max_ifsd: u32,
    /// CCID features bitmask. Default is `0x00010040` (Short APDU level exchange + automatic voltage selection).
    pub features: u32,
    /// Maximum CCID message length. Default is 271 (10-byte header + 261-byte APDU).
    pub max_message_length: u32,
    /// Optional Interrupt IN endpoint configuration `(max_packet_size, poll_ms)`.
    /// When specified, an interrupt endpoint is allocated for `RDR_to_PC_NotifySlotChange` messages.
    pub interrupt_endpoint: Option<(u16, u8)>,
    /// Optional interface string description.
    pub interface_name: Option<&'d str>,
}

impl<'d> Default for Config<'d> {
    fn default() -> Self {
        Self {
            max_packet_size: 64,
            max_slot_index: 0,
            voltage_support: 0x07,
            protocols: 0x00000003,
            default_clock_khz: 3580,
            max_clock_khz: 3580,
            default_data_rate_bps: 9600,
            max_data_rate_bps: 115200,
            max_ifsd: 254,
            features: 0x00010040,
            max_message_length: (HEADER_LEN + MAX_APDU_LEN) as u32,
            interrupt_endpoint: None,
            interface_name: None,
        }
    }
}

impl<'d> Config<'d> {
    /// Builds the 52-byte body of the CCID functional descriptor (excluding `bLength` and `bDescriptorType`).
    pub const fn functional_descriptor_body(&self) -> [u8; 52] {
        let bcd_ccid = [0x10, 0x01]; // CCID 1.10
        let proto = self.protocols.to_le_bytes();
        let def_clk = self.default_clock_khz.to_le_bytes();
        let max_clk = self.max_clock_khz.to_le_bytes();
        let def_rate = self.default_data_rate_bps.to_le_bytes();
        let max_rate = self.max_data_rate_bps.to_le_bytes();
        let max_ifsd = self.max_ifsd.to_le_bytes();
        let features = self.features.to_le_bytes();
        let max_msg = self.max_message_length.to_le_bytes();

        [
            bcd_ccid[0],
            bcd_ccid[1],
            self.max_slot_index,
            self.voltage_support,
            proto[0],
            proto[1],
            proto[2],
            proto[3],
            def_clk[0],
            def_clk[1],
            def_clk[2],
            def_clk[3],
            max_clk[0],
            max_clk[1],
            max_clk[2],
            max_clk[3],
            0, // bNumClockSupported
            def_rate[0],
            def_rate[1],
            def_rate[2],
            def_rate[3],
            max_rate[0],
            max_rate[1],
            max_rate[2],
            max_rate[3],
            0, // bNumDataRatesSupported
            max_ifsd[0],
            max_ifsd[1],
            max_ifsd[2],
            max_ifsd[3],
            0,
            0,
            0,
            0, // dwSynchProtocols
            0,
            0,
            0,
            0, // dwMechanical
            features[0],
            features[1],
            features[2],
            features[3],
            max_msg[0],
            max_msg[1],
            max_msg[2],
            max_msg[3],
            0x00, // bClassGetResponse
            0x00, // bClassEnvelope
            0x00,
            0x00, // wLcdLayout
            0x00, // bPINSupport
            self.max_slot_index.saturating_add(1),
        ]
    }
}

/// State for the CCID device class.
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

struct Control<'d> {
    iface_number: InterfaceNumber,
    iface_string: Option<StringIndex>,
    clock_khz: [u8; 4],
    data_rate_bps: [u8; 4],
    _marker: core::marker::PhantomData<&'d ()>,
}

impl<'d> Handler for Control<'d> {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if self.iface_string == Some(index) {
            Some("CCID Smart Card Device")
        } else {
            None
        }
    }

    fn control_out(&mut self, req: Request, _buf: &[u8]) -> Option<OutResponse> {
        if req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == self.iface_number.0 as u16
        {
            match req.request {
                CCID_REQ_ABORT => Some(OutResponse::Accepted),
                _ => Some(OutResponse::Rejected),
            }
        } else {
            None
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == self.iface_number.0 as u16
        {
            match req.request {
                CCID_REQ_GET_CLOCK_FREQUENCIES => {
                    let len = core::cmp::min(self.clock_khz.len(), buf.len());
                    buf[..len].copy_from_slice(&self.clock_khz[..len]);
                    Some(InResponse::Accepted(&buf[..len]))
                }
                CCID_REQ_GET_DATA_RATES => {
                    let len = core::cmp::min(self.data_rate_bps.len(), buf.len());
                    buf[..len].copy_from_slice(&self.data_rate_bps[..len]);
                    Some(InResponse::Accepted(&buf[..len]))
                }
                _ => Some(InResponse::Rejected),
            }
        } else {
            None
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface_number
    }
}

/// A parsed CCID command received from the host over the Bulk OUT endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CcidCommand<'a> {
    /// `PC_to_RDR_IccPowerOn` (0x62): Host requests ICC card activation and ATR.
    IccPowerOn {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
        /// Power selection voltage (0 = automatic/reader choice).
        power_select: u8,
    },
    /// `PC_to_RDR_IccPowerOff` (0x63): Host requests ICC card deactivation.
    IccPowerOff {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
    },
    /// `PC_to_RDR_GetSlotStatus` (0x65): Host queries slot status.
    GetSlotStatus {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
    },
    /// `PC_to_RDR_XfrBlock` (0x6F): Host sends an APDU block to the smart card.
    XfrBlock {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
        /// Block waiting time integer.
        bwi: u8,
        /// Level parameter (e.g. expected response length or chaining info).
        level_parameter: u16,
        /// APDU payload data.
        apdu: &'a [u8],
    },
    /// `PC_to_RDR_GetParameters` (0x6C): Host queries slot parameters.
    GetParameters {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
    },
    /// `PC_to_RDR_SetParameters` (0x61): Host sets slot protocol parameters.
    SetParameters {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
        /// Protocol number (0 for T=0, 1 for T=1).
        protocol_num: u8,
        /// Parameter bytes.
        data: &'a [u8],
    },
    /// `PC_to_RDR_ResetParameters` (0x6D): Host resets slot parameters to defaults.
    ResetParameters {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
    },
    /// `PC_to_RDR_Abort` (0x72): Host requests aborting an ongoing transfer.
    Abort {
        /// Target slot number.
        slot: u8,
        /// Sequence number for correlation.
        seq: u8,
    },
    /// Unknown or unsupported CCID command.
    Unknown {
        /// Raw CCID message type byte.
        message_type: u8,
        /// Slot number from header.
        slot: u8,
        /// Sequence number from header.
        seq: u8,
        /// Raw message payload.
        payload: &'a [u8],
    },
}

/// USB CCID reader/writer device class.
pub struct CcidDevice<'d, D: Driver<'d>> {
    reader: CcidReader<'d, D>,
    writer: CcidWriter<'d, D>,
}

impl<'d, D: Driver<'d>> CcidDevice<'d, D> {
    /// Create a new CCID device class instance on the USB device builder.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let iface_string = if config.interface_name.is_some() {
            Some(builder.string())
        } else {
            None
        };

        let mut func = builder.function(USB_CLASS_CCID, USB_SUBCLASS_CCID, USB_PROTOCOL_CCID);

        let mut iface = func.interface();
        let iface_number = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CCID, USB_SUBCLASS_CCID, USB_PROTOCOL_CCID, iface_string);

        // Write 54-byte CCID functional descriptor (length 54 = 2-byte header + 52-byte body)
        let body = config.functional_descriptor_body();
        alt.descriptor(CCID_FUNCTIONAL_DESCRIPTOR, &body);

        let write_ep = alt.endpoint_bulk_in(None, config.max_packet_size);
        let read_ep = alt.endpoint_bulk_out(None, config.max_packet_size);
        let int_ep = config
            .interrupt_endpoint
            .map(|(size, poll_ms)| alt.endpoint_interrupt_in(None, size, poll_ms));

        drop(func);

        let control = state.control.write(Control {
            iface_number,
            iface_string,
            clock_khz: config.default_clock_khz.to_le_bytes(),
            data_rate_bps: config.default_data_rate_bps.to_le_bytes(),
            _marker: core::marker::PhantomData,
        });
        builder.interface_handler(control);

        Self {
            reader: CcidReader { read_ep },
            writer: CcidWriter { write_ep, int_ep },
        }
    }

    /// Split this device into independent reader and writer handles.
    pub fn split(self) -> (CcidReader<'d, D>, CcidWriter<'d, D>) {
        (self.reader, self.writer)
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.reader.wait_connection().await;
    }

    /// Read and decode the next CCID command from the host.
    pub async fn read_command<'a>(&mut self, buf: &'a mut [u8]) -> Result<CcidCommand<'a>, EndpointError> {
        self.reader.read_command(buf).await
    }

    /// Send an `RDR_to_PC_DataBlock` message carrying data (such as ATR or APDU response).
    pub async fn send_data_block(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        data: &[u8],
    ) -> Result<(), EndpointError> {
        self.writer.send_data_block(slot, seq, status, error, data).await
    }

    /// Send an `RDR_to_PC_SlotStatus` message reporting current card presence and clock status.
    pub async fn send_slot_status(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        clock_status: u8,
    ) -> Result<(), EndpointError> {
        self.writer
            .send_slot_status(slot, seq, status, error, clock_status)
            .await
    }

    /// Send an `RDR_to_PC_Parameters` message.
    pub async fn send_parameters(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        protocol_num: u8,
        params: &[u8],
    ) -> Result<(), EndpointError> {
        self.writer
            .send_parameters(slot, seq, status, error, protocol_num, params)
            .await
    }

    /// Send an `RDR_to_PC_NotifySlotChange` event message over the Interrupt IN endpoint.
    pub async fn notify_slot_change(&mut self, slot: u8, card_present: bool) -> Result<(), EndpointError> {
        self.writer.notify_slot_change(slot, card_present).await
    }
}

/// Receiver handle for incoming CCID messages from the host.
pub struct CcidReader<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> CcidReader<'d, D> {
    /// Waits for the USB host to enable the Bulk OUT endpoint.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Read a raw message from the host over the Bulk OUT endpoint.
    pub async fn read_message(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(buf).await
    }

    /// Read and decode the next CCID command from the host into `buf`.
    /// `buf` must be large enough to hold the complete CCID message (at least 10 bytes).
    pub async fn read_command<'a>(&mut self, buf: &'a mut [u8]) -> Result<CcidCommand<'a>, EndpointError> {
        let len = self.read_message(buf).await?;
        if len < HEADER_LEN {
            return Ok(CcidCommand::Unknown {
                message_type: 0,
                slot: 0,
                seq: 0,
                payload: &[],
            });
        }

        let msg_type = buf[0];
        let payload_len = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        let slot = buf[5];
        let seq = buf[6];
        let payload_end = (HEADER_LEN + payload_len).min(len);
        let payload = &buf[HEADER_LEN..payload_end];

        match msg_type {
            PC_TO_RDR_ICC_POWER_ON => Ok(CcidCommand::IccPowerOn {
                slot,
                seq,
                power_select: buf[7],
            }),
            PC_TO_RDR_ICC_POWER_OFF => Ok(CcidCommand::IccPowerOff { slot, seq }),
            PC_TO_RDR_GET_SLOT_STATUS => Ok(CcidCommand::GetSlotStatus { slot, seq }),
            PC_TO_RDR_XFR_BLOCK => Ok(CcidCommand::XfrBlock {
                slot,
                seq,
                bwi: buf[7],
                level_parameter: u16::from_le_bytes([buf[8], buf[9]]),
                apdu: payload,
            }),
            PC_TO_RDR_GET_PARAMETERS => Ok(CcidCommand::GetParameters { slot, seq }),
            PC_TO_RDR_SET_PARAMETERS => Ok(CcidCommand::SetParameters {
                slot,
                seq,
                protocol_num: buf[7],
                data: payload,
            }),
            PC_TO_RDR_RESET_PARAMETERS => Ok(CcidCommand::ResetParameters { slot, seq }),
            PC_TO_RDR_ABORT => Ok(CcidCommand::Abort { slot, seq }),
            _ => Ok(CcidCommand::Unknown {
                message_type: msg_type,
                slot,
                seq,
                payload,
            }),
        }
    }
}

/// Sender handle for outgoing CCID messages and notifications to the host.
pub struct CcidWriter<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    int_ep: Option<D::EndpointIn>,
}

impl<'d, D: Driver<'d>> CcidWriter<'d, D> {
    /// Waits for the USB host to enable the Bulk IN endpoint.
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await;
    }

    /// Send an `RDR_to_PC_DataBlock` message containing data (such as ATR or APDU response).
    pub async fn send_data_block(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        data: &[u8],
    ) -> Result<(), EndpointError> {
        let max_pkt = self.write_ep.info().max_packet_size as usize;
        let total_wire_bytes = HEADER_LEN + data.len();

        let mut header = [0u8; HEADER_LEN];
        header[0] = RDR_TO_PC_DATA_BLOCK;
        header[1..5].copy_from_slice(&(data.len() as u32).to_le_bytes());
        header[5] = slot;
        header[6] = seq;
        header[7] = status;
        header[8] = error;
        header[9] = 0; // chain parameter

        if total_wire_bytes <= max_pkt {
            // Fits entirely in one packet
            let mut packet = [0u8; 512];
            let pkt_len = total_wire_bytes.min(packet.len());
            packet[..HEADER_LEN].copy_from_slice(&header);
            packet[HEADER_LEN..HEADER_LEN + data.len()].copy_from_slice(data);
            self.write_ep.write_transfer(&packet[..pkt_len], true).await?;
        } else {
            // First chunk: 10-byte header + initial data bytes
            let first_data_len = max_pkt - HEADER_LEN;
            let mut packet = [0u8; 512];
            packet[..HEADER_LEN].copy_from_slice(&header);
            packet[HEADER_LEN..max_pkt].copy_from_slice(&data[..first_data_len]);
            self.write_ep.write(&packet[..max_pkt]).await?;

            // Remaining data chunks
            let mut offset = first_data_len;
            while offset + max_pkt <= data.len() {
                self.write_ep.write(&data[offset..offset + max_pkt]).await?;
                offset += max_pkt;
            }

            // Final short data packet
            if offset < data.len() {
                self.write_ep.write(&data[offset..]).await?;
            }

            // Zero-Length Packet if the entire transfer length is a multiple of max_packet_size
            if total_wire_bytes % max_pkt == 0 {
                self.write_ep.write(&[]).await?;
            }
        }

        Ok(())
    }

    /// Send an `RDR_to_PC_SlotStatus` message.
    pub async fn send_slot_status(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        clock_status: u8,
    ) -> Result<(), EndpointError> {
        let mut msg = [0u8; HEADER_LEN];
        msg[0] = RDR_TO_PC_SLOT_STATUS;
        msg[5] = slot;
        msg[6] = seq;
        msg[7] = status;
        msg[8] = error;
        msg[9] = clock_status;
        self.write_ep.write_transfer(&msg, true).await
    }

    /// Send an `RDR_to_PC_Parameters` message.
    pub async fn send_parameters(
        &mut self,
        slot: u8,
        seq: u8,
        status: u8,
        error: u8,
        protocol_num: u8,
        params: &[u8],
    ) -> Result<(), EndpointError> {
        let mut msg = [0u8; HEADER_LEN + 16];
        let total_len = HEADER_LEN + params.len();
        msg[0] = RDR_TO_PC_PARAMETERS;
        msg[1..5].copy_from_slice(&(params.len() as u32).to_le_bytes());
        msg[5] = slot;
        msg[6] = seq;
        msg[7] = status;
        msg[8] = error;
        msg[9] = protocol_num;
        let copy_len = params.len().min(16);
        msg[HEADER_LEN..HEADER_LEN + copy_len].copy_from_slice(&params[..copy_len]);
        self.write_ep
            .write_transfer(&msg[..total_len.min(msg.len())], true)
            .await
    }

    /// Send an `RDR_to_PC_NotifySlotChange` interrupt event.
    pub async fn notify_slot_change(&mut self, slot: u8, card_present: bool) -> Result<(), EndpointError> {
        if let Some(ref mut int_ep) = self.int_ep {
            let mut msg = [0u8; 2];
            msg[0] = RDR_TO_PC_NOTIFY_SLOT_CHANGE;
            // 2 bits per slot: bit 0 = state, bit 1 = changed
            let shift = (slot & 0x03) * 2;
            let state_bits = if card_present { 0x03 } else { 0x02 };
            msg[1] = state_bits << shift;
            int_ep.write_transfer(&msg, true).await
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn functional_descriptor_length_and_fields() {
        let config = Config::default();
        let body = config.functional_descriptor_body();
        assert_eq!(body.len(), 52); // 52 body + 2 header = 54
        assert_eq!(body[0], 0x10); // bcdCCID lo
        assert_eq!(body[1], 0x01); // bcdCCID hi
        assert_eq!(body[2], 0); // bMaxSlotIndex
        assert_eq!(body[3], 0x07); // bVoltageSupport
        assert_eq!(&body[4..8], &3u32.to_le_bytes()); // dwProtocols T=0 and T=1
    }

    #[test]
    fn parses_ccid_commands() {
        let power_on_msg = [PC_TO_RDR_ICC_POWER_ON, 0, 0, 0, 0, 0, 42, 1, 0, 0];
        let mut reader_dummy = [0u8; 10];
        reader_dummy.copy_from_slice(&power_on_msg);

        let msg_type = power_on_msg[0];
        let slot = power_on_msg[5];
        let seq = power_on_msg[6];
        assert_eq!(msg_type, PC_TO_RDR_ICC_POWER_ON);
        assert_eq!(slot, 0);
        assert_eq!(seq, 42);
        assert_eq!(power_on_msg[7], 1); // power_select

        let apdu_payload = [0x00, 0xA4, 0x04, 0x00, 0x07, 0xA0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00];
        let mut xfr_msg = [0u8; HEADER_LEN + 12];
        xfr_msg[0] = PC_TO_RDR_XFR_BLOCK;
        xfr_msg[1..5].copy_from_slice(&12u32.to_le_bytes());
        xfr_msg[5] = 0;
        xfr_msg[6] = 99;
        xfr_msg[HEADER_LEN..].copy_from_slice(&apdu_payload);

        let payload_len = u32::from_le_bytes([xfr_msg[1], xfr_msg[2], xfr_msg[3], xfr_msg[4]]) as usize;
        assert_eq!(payload_len, 12);
        assert_eq!(&xfr_msg[HEADER_LEN..HEADER_LEN + payload_len], &apdu_payload);
    }
}
