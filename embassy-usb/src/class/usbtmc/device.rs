//! USB Test & Measurement Class (USBTMC) device implementation.

use core::cell::Cell;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::types::*;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointAddress, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, Handler};

/// Configuration for the USBTMC device class.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Maximum packet size in bytes for bulk endpoints (default: 64 for full-speed).
    pub max_packet_size: u16,
    /// Whether to enable the USB488 subclass protocol.
    pub usb488: bool,
    /// Whether to allocate an Interrupt-IN endpoint for SRQ notifications.
    pub interrupt_endpoint: bool,
    /// Polling interval in ms for the Interrupt-IN endpoint.
    pub interrupt_poll_ms: u8,
    /// Capabilities reported to the host in `GET_CAPABILITIES`.
    pub capabilities: Capabilities,
}

impl Config {
    /// Creates a default USBTMC 1.0 configuration without USB488 extensions.
    pub const fn new() -> Self {
        Self {
            max_packet_size: 64,
            usb488: false,
            interrupt_endpoint: false,
            interrupt_poll_ms: 8,
            capabilities: Capabilities {
                listen_only: false,
                talk_only: false,
                supports_indicator_pulse: true,
                can_end_bulk_in_on_term_char: false,
                supports_trigger: false,
                supports_ren_gtl_llo: false,
                is_488_2: false,
                dt1: false,
                rl1: false,
                sr1: false,
                scpi: false,
            },
        }
    }

    /// Creates a default configuration with USB488 and SCPI support enabled.
    pub const fn new_usb488() -> Self {
        Self {
            max_packet_size: 64,
            usb488: true,
            interrupt_endpoint: false,
            interrupt_poll_ms: 8,
            capabilities: Capabilities {
                listen_only: false,
                talk_only: false,
                supports_indicator_pulse: true,
                can_end_bulk_in_on_term_char: false,
                supports_trigger: true,
                supports_ren_gtl_llo: false,
                is_488_2: true,
                dt1: false,
                rl1: false,
                sr1: false,
                scpi: true,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new_usb488()
    }
}

#[derive(Clone, Copy, Debug)]
struct ControlInner {
    status_byte: u8,
    clear_pending: bool,
    abort_out_tag: u8,
    abort_in_tag: u8,
    bytes_rx: u32,
    bytes_tx: u32,
    indicator_pulses: u32,
    trigger_count: u32,
}

impl ControlInner {
    const fn new() -> Self {
        Self {
            status_byte: 0,
            clear_pending: false,
            abort_out_tag: 0,
            abort_in_tag: 0,
            bytes_rx: 0,
            bytes_tx: 0,
            indicator_pulses: 0,
            trigger_count: 0,
        }
    }
}

/// Shared data between control request handler and application API.
pub struct ControlShared {
    pub(crate) capabilities: Capabilities,
    pub(crate) usb488: bool,
    inner: CriticalSectionMutex<Cell<ControlInner>>,
}

impl ControlShared {
    /// Creates a new `ControlShared`.
    pub const fn new(capabilities: Capabilities, usb488: bool) -> Self {
        Self {
            capabilities,
            usb488,
            inner: CriticalSectionMutex::new(Cell::new(ControlInner::new())),
        }
    }

    /// Sets the IEEE 488.2 status byte (`*STB`).
    pub fn set_status_byte(&self, stb: u8) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.status_byte = stb;
            cell.set(inner);
        });
    }

    /// Gets the current IEEE 488.2 status byte (`*STB`).
    pub fn status_byte(&self) -> u8 {
        self.inner.lock(|cell| cell.get().status_byte)
    }

    /// Checks if a clear request was issued by the host, clearing the flag if set.
    pub fn check_clear(&self) -> bool {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            let pending = inner.clear_pending;
            inner.clear_pending = false;
            cell.set(inner);
            pending
        })
    }

    /// Sets or clears the pending clear flag.
    pub fn set_clear_pending(&self, val: bool) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.clear_pending = val;
            cell.set(inner);
        });
    }

    /// Gets the number of indicator pulses received from the host.
    pub fn indicator_pulse_count(&self) -> u32 {
        self.inner.lock(|cell| cell.get().indicator_pulses)
    }

    /// Increments the indicator pulse counter.
    pub fn increment_indicator_pulse(&self) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.indicator_pulses = inner.indicator_pulses.saturating_add(1);
            cell.set(inner);
        });
    }

    /// Gets the total number of hardware trigger commands received.
    pub fn trigger_count(&self) -> u32 {
        self.inner.lock(|cell| cell.get().trigger_count)
    }

    pub(crate) fn trigger(&self) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.trigger_count = inner.trigger_count.saturating_add(1);
            cell.set(inner);
        });
    }

    pub(crate) fn set_abort_out(&self, tag: u8) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.abort_out_tag = tag;
            cell.set(inner);
        });
    }

    pub(crate) fn set_abort_in(&self, tag: u8) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.abort_in_tag = tag;
            cell.set(inner);
        });
    }

    pub(crate) fn add_bytes_rx(&self, count: u32) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.bytes_rx = inner.bytes_rx.wrapping_add(count);
            cell.set(inner);
        });
    }

    pub(crate) fn bytes_rx(&self) -> u32 {
        self.inner.lock(|cell| cell.get().bytes_rx)
    }

    pub(crate) fn add_bytes_tx(&self, count: u32) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.bytes_tx = inner.bytes_tx.wrapping_add(count);
            cell.set(inner);
        });
    }

    pub(crate) fn bytes_tx(&self) -> u32 {
        self.inner.lock(|cell| cell.get().bytes_tx)
    }

    /// Resets all internal state flags and counters.
    pub fn reset(&self) {
        self.inner.lock(|cell| {
            cell.set(ControlInner::new());
        });
    }
}

/// State for the USBTMC USB class.
pub struct State {
    control: MaybeUninit<Control>,
    shared: ControlShared,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Create a new `State` with default USB488 settings.
    pub const fn new() -> Self {
        Self::with_config(
            Capabilities {
                listen_only: false,
                talk_only: false,
                supports_indicator_pulse: true,
                can_end_bulk_in_on_term_char: false,
                supports_trigger: true,
                supports_ren_gtl_llo: false,
                is_488_2: true,
                dt1: false,
                rl1: false,
                sr1: false,
                scpi: true,
            },
            true,
        )
    }

    /// Create a new `State` with custom capabilities and protocol mode.
    pub const fn with_config(capabilities: Capabilities, usb488: bool) -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared::new(capabilities, usb488),
        }
    }
}

struct Control {
    comm_if: InterfaceNumber,
    iface_string: StringIndex,
    bulk_out_ep: EndpointAddress,
    bulk_in_ep: EndpointAddress,
    #[allow(unused)]
    int_in_ep: Option<EndpointAddress>,
    shared: &'static ControlShared,
}

impl Handler for Control {
    fn reset(&mut self) {
        self.shared.reset();
    }

    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if index == self.iface_string {
            Some("USBTMC Interface")
        } else {
            None
        }
    }

    fn control_out(&mut self, req: Request, _data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface)
            || req.index != self.comm_if.0 as u16
        {
            return None;
        }

        match req.request {
            REQ_INDICATOR_PULSE => {
                self.shared.increment_indicator_pulse();
                Some(OutResponse::Accepted)
            }
            REQ_USB488_REN_CONTROL | REQ_USB488_GO_TO_LOCAL | REQ_USB488_LOCAL_LOCKOUT => Some(OutResponse::Rejected),
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        match (req.request_type, req.recipient) {
            (RequestType::Class, Recipient::Interface) if req.index == self.comm_if.0 as u16 => {
                match req.request {
                    REQ_GET_CAPABILITIES => {
                        let caps_bytes = self.shared.capabilities.to_bytes(self.shared.usb488);
                        assert!(buf.len() >= 24);
                        let len = core::cmp::min(req.length as usize, 24);
                        buf[..len].copy_from_slice(&caps_bytes[..len]);
                        Some(InResponse::Accepted(&buf[..len]))
                    }
                    REQ_INITIATE_CLEAR => {
                        self.shared.set_clear_pending(true);
                        assert!(buf.len() >= 1);
                        buf[0] = UsbtmcStatus::Success as u8;
                        Some(InResponse::Accepted(&buf[..1]))
                    }
                    REQ_CHECK_CLEAR_STATUS => {
                        assert!(buf.len() >= 2);
                        buf[0] = UsbtmcStatus::Success as u8;
                        buf[1] = 0x00; // bmClear: 0 indicates Bulk-IN FIFO is empty
                        Some(InResponse::Accepted(&buf[..2]))
                    }
                    REQ_INDICATOR_PULSE => {
                        self.shared.increment_indicator_pulse();
                        assert!(buf.len() >= 1);
                        buf[0] = UsbtmcStatus::Success as u8;
                        Some(InResponse::Accepted(&buf[..1]))
                    }
                    REQ_USB488_READ_STATUS_BYTE if self.shared.usb488 => {
                        let b_tag = (req.value & 0x7F) as u8;
                        if b_tag < 2 || (req.value & !0x7F) != 0 {
                            return Some(InResponse::Rejected);
                        }
                        assert!(buf.len() >= 3);
                        buf[0] = UsbtmcStatus::Success as u8;
                        buf[1] = b_tag;
                        buf[2] = self.shared.status_byte();
                        Some(InResponse::Accepted(&buf[..3]))
                    }
                    _ => Some(InResponse::Rejected),
                }
            }
            (RequestType::Class, Recipient::Endpoint) => {
                let ep_addr = EndpointAddress::from(req.index as u8);
                if ep_addr == self.bulk_out_ep {
                    match req.request {
                        REQ_INITIATE_ABORT_BULK_OUT => {
                            let b_tag = (req.value & 0xFF) as u8;
                            self.shared.set_abort_out(b_tag);
                            assert!(buf.len() >= 2);
                            buf[0] = UsbtmcStatus::Success as u8;
                            buf[1] = b_tag;
                            Some(InResponse::Accepted(&buf[..2]))
                        }
                        REQ_CHECK_ABORT_BULK_OUT_STATUS => {
                            assert!(buf.len() >= 8);
                            buf[0] = UsbtmcStatus::Success as u8;
                            buf[1] = 0x00;
                            buf[2] = 0x00;
                            buf[3] = 0x00;
                            buf[4..8].copy_from_slice(&self.shared.bytes_rx().to_le_bytes());
                            Some(InResponse::Accepted(&buf[..8]))
                        }
                        _ => Some(InResponse::Rejected),
                    }
                } else if ep_addr == self.bulk_in_ep {
                    match req.request {
                        REQ_INITIATE_ABORT_BULK_IN => {
                            let b_tag = (req.value & 0xFF) as u8;
                            self.shared.set_abort_in(b_tag);
                            assert!(buf.len() >= 2);
                            buf[0] = UsbtmcStatus::Success as u8;
                            buf[1] = b_tag;
                            Some(InResponse::Accepted(&buf[..2]))
                        }
                        REQ_CHECK_ABORT_BULK_IN_STATUS => {
                            assert!(buf.len() >= 8);
                            buf[0] = UsbtmcStatus::Success as u8;
                            buf[1] = 0x00;
                            buf[2] = 0x00;
                            buf[3] = 0x00;
                            buf[4..8].copy_from_slice(&self.shared.bytes_tx().to_le_bytes());
                            Some(InResponse::Accepted(&buf[..8]))
                        }
                        _ => Some(InResponse::Rejected),
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// USB device class for USB Test & Measurement Class (USBTMC).
pub struct UsbtmcClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    int_ep: Option<D::EndpointIn>,
    shared: &'d ControlShared,
    pending_request_in: Option<RequestDevDepMsgInHeader>,
    max_packet_size: u16,
}

impl<'d, D: Driver<'d>> UsbtmcClass<'d, D> {
    /// Creates a new `UsbtmcClass` device instance.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State, config: Config) -> Self {
        state.shared.capabilities = config.capabilities;
        state.shared.usb488 = config.usb488;

        let protocol = if config.usb488 {
            USB_PROTOCOL_USB488
        } else {
            USB_PROTOCOL_USBTMC
        };

        let iface_string = builder.string();
        let mut function = builder.function(USB_CLASS_APPL_SPEC, USB_SUBCLASS_TEST_MEASUREMENT, protocol);
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(
            USB_CLASS_APPL_SPEC,
            USB_SUBCLASS_TEST_MEASUREMENT,
            protocol,
            Some(iface_string),
        );
        let read_ep = alt.endpoint_bulk_out(None, config.max_packet_size);
        let write_ep = alt.endpoint_bulk_in(None, config.max_packet_size);
        let int_ep = if config.interrupt_endpoint {
            Some(alt.endpoint_interrupt_in(None, 2, config.interrupt_poll_ms))
        } else {
            None
        };
        let comm_if = interface.interface_number();
        drop(function);

        let bulk_out_ep = read_ep.info().addr;
        let bulk_in_ep = write_ep.info().addr;
        let int_in_ep = int_ep.as_ref().map(|ep| ep.info().addr);

        // Safety: the handler is bound to the lifetime of state
        let shared_static: &'static ControlShared = unsafe { core::mem::transmute(&state.shared) };

        builder.handler(state.control.write(Control {
            comm_if,
            iface_string,
            bulk_out_ep,
            bulk_in_ep,
            int_in_ep,
            shared: shared_static,
        }));

        UsbtmcClass {
            read_ep,
            write_ep,
            int_ep,
            shared: &state.shared,
            pending_request_in: None,
            max_packet_size: config.max_packet_size,
        }
    }

    /// Waits until the USB host configures and enables this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Stalls Bulk-OUT when `e` leaves the bulk message stream out of sync, and returns it.
    ///
    /// The bulk message stream is a reassembly of headers and payloads with no way to
    /// resynchronize: a bad `bTag`, an unexpected MsgID, a packet too short to hold a
    /// header, or a payload longer than the caller's buffer all mean the next bytes on the
    /// wire are payload that would be parsed as a header. Halt the endpoint instead, so
    /// the host sees the failure and aborts the transfer (or issues `INITIATE_CLEAR`)
    /// rather than having the device silently misread the following messages.
    ///
    /// Endpoint errors are not protocol errors - the driver or the host already knows
    /// about those - so they are passed through without stalling.
    fn fail(&mut self, e: UsbtmcError) -> UsbtmcError {
        if matches!(
            e,
            UsbtmcError::InvalidTag
                | UsbtmcError::UnexpectedMsgId(_)
                | UsbtmcError::PacketTooShort
                | UsbtmcError::BufferTooSmall
                | UsbtmcError::MessageTruncated
        ) {
            self.read_ep.set_stall(true);
        }
        e
    }

    async fn read_message_payload<'a>(
        &mut self,
        buf: &'a mut [u8],
        packet_buf: &mut [u8],
        n: usize,
    ) -> Result<ReceivedMessage<'a>, UsbtmcError> {
        let max_pkt = self.max_packet_size as usize;
        let header = DevDepMsgOutHeader::parse(packet_buf[0..12].try_into().unwrap()).map_err(|e| self.fail(e))?;
        let transfer_size = header.transfer_size as usize;
        if transfer_size > buf.len() {
            return Err(self.fail(UsbtmcError::BufferTooSmall));
        }

        let first_chunk = core::cmp::min(transfer_size, n - 12);
        buf[..first_chunk].copy_from_slice(&packet_buf[12..12 + first_chunk]);
        let mut bytes_copied = first_chunk;
        let padding = padding_bytes(transfer_size);
        let total_wire_bytes = 12 + transfer_size + padding;
        let mut total_wire_read = n;

        while total_wire_read < total_wire_bytes {
            let m = self.read_ep.read(packet_buf).await?;
            if m == 0 {
                break;
            }
            total_wire_read += m;
            if bytes_copied < transfer_size {
                let to_copy = core::cmp::min(m, transfer_size - bytes_copied);
                buf[bytes_copied..bytes_copied + to_copy].copy_from_slice(&packet_buf[..to_copy]);
                bytes_copied += to_copy;
            }
            if m < max_pkt {
                break;
            }
        }

        if bytes_copied < transfer_size {
            return Err(self.fail(UsbtmcError::MessageTruncated));
        }

        self.shared.add_bytes_rx(total_wire_read as u32);
        Ok(ReceivedMessage {
            payload: &buf[..bytes_copied],
            eom: header.eom,
            tag: header.b_tag,
        })
    }

    /// Reads the next event from the Bulk-OUT endpoint.
    pub async fn read_event<'a>(&mut self, buf: &'a mut [u8]) -> Result<UsbtmcEvent<'a>, UsbtmcError> {
        let mut packet = [0u8; 512];
        let max_pkt = self.max_packet_size as usize;
        let packet_buf = &mut packet[..max_pkt];
        let n = self.read_ep.read(packet_buf).await?;
        if n < 12 {
            return Err(self.fail(UsbtmcError::PacketTooShort));
        }

        match packet_buf[0] {
            MSG_ID_DEV_DEP_MSG_OUT => {
                let msg = self.read_message_payload(buf, packet_buf, n).await?;
                Ok(UsbtmcEvent::Message(msg))
            }
            MSG_ID_VENDOR_SPECIFIC_OUT => {
                let msg = self.read_message_payload(buf, packet_buf, n).await?;
                Ok(UsbtmcEvent::VendorSpecific(msg))
            }
            MSG_ID_REQUEST_DEV_DEP_MSG_IN | MSG_ID_REQUEST_VENDOR_SPECIFIC_IN => {
                let header =
                    RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap()).map_err(|e| self.fail(e))?;
                self.shared.add_bytes_rx(n as u32);
                Ok(UsbtmcEvent::RequestIn(header))
            }
            MSG_ID_TRIGGER => {
                let base = MsgHeader::parse(packet_buf[0..4].try_into().unwrap()).map_err(|e| self.fail(e))?;
                self.shared.add_bytes_rx(n as u32);
                self.shared.trigger();
                Ok(UsbtmcEvent::Trigger(base.b_tag))
            }
            id => Err(self.fail(UsbtmcError::UnexpectedMsgId(id))),
        }
    }

    /// Reads the next command message from the host.
    ///
    /// If an incoming `REQUEST_DEV_DEP_MSG_IN` or `TRIGGER` event is encountered, it is
    /// recorded in the internal state before continuing to wait for a command message.
    pub async fn read_message<'a>(&mut self, buf: &'a mut [u8]) -> Result<ReceivedMessage<'a>, UsbtmcError> {
        loop {
            let mut packet = [0u8; 512];
            let max_pkt = self.max_packet_size as usize;
            let packet_buf = &mut packet[..max_pkt];
            let n = self.read_ep.read(packet_buf).await?;
            if n < 12 {
                return Err(self.fail(UsbtmcError::PacketTooShort));
            }

            match packet_buf[0] {
                MSG_ID_DEV_DEP_MSG_OUT | MSG_ID_VENDOR_SPECIFIC_OUT => {
                    return self.read_message_payload(buf, packet_buf, n).await;
                }
                MSG_ID_REQUEST_DEV_DEP_MSG_IN | MSG_ID_REQUEST_VENDOR_SPECIFIC_IN => {
                    let header = RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap())
                        .map_err(|e| self.fail(e))?;
                    self.shared.add_bytes_rx(n as u32);
                    self.pending_request_in = Some(header);
                }
                MSG_ID_TRIGGER => {
                    let _base = MsgHeader::parse(packet_buf[0..4].try_into().unwrap()).map_err(|e| self.fail(e))?;
                    self.shared.add_bytes_rx(n as u32);
                    self.shared.trigger();
                }
                id => return Err(self.fail(UsbtmcError::UnexpectedMsgId(id))),
            }
        }
    }

    /// Waits for the host to send a `REQUEST_DEV_DEP_MSG_IN` request on Bulk-OUT.
    pub async fn wait_request_in(&mut self) -> Result<RequestDevDepMsgInHeader, UsbtmcError> {
        if let Some(req) = self.pending_request_in.take() {
            return Ok(req);
        }
        let mut packet = [0u8; 512];
        let max_pkt = self.max_packet_size as usize;
        let packet_buf = &mut packet[..max_pkt];
        loop {
            let n = self.read_ep.read(packet_buf).await?;
            if n < 12 {
                return Err(self.fail(UsbtmcError::PacketTooShort));
            }
            match packet_buf[0] {
                MSG_ID_REQUEST_DEV_DEP_MSG_IN | MSG_ID_REQUEST_VENDOR_SPECIFIC_IN => {
                    let header = RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap())
                        .map_err(|e| self.fail(e))?;
                    self.shared.add_bytes_rx(n as u32);
                    return Ok(header);
                }
                MSG_ID_TRIGGER => {
                    self.shared.add_bytes_rx(n as u32);
                    self.shared.trigger();
                }
                _ => {}
            }
        }
    }

    /// Sends a response message to the host, automatically awaiting the matching
    /// `REQUEST_DEV_DEP_MSG_IN` and using its tag.
    pub async fn write_message(&mut self, data: &[u8], eom: bool) -> Result<(), UsbtmcError> {
        let req = self.wait_request_in().await?;
        self.write_message_with_tag(req.b_tag, data, eom).await
    }

    /// Writes a response message with an explicit `b_tag`.
    pub async fn write_message_with_tag(&mut self, tag: u8, data: &[u8], eom: bool) -> Result<(), UsbtmcError> {
        let transfer_size = data.len();
        let padding = padding_bytes(transfer_size);
        let total_wire_bytes = 12 + transfer_size + padding;
        let max_pkt = self.max_packet_size as usize;

        let header = DevDepMsgInHeader {
            b_tag: tag,
            transfer_size: transfer_size as u32,
            eom,
            using_term_char: false,
        };

        let mut packet = [0u8; 512];
        header.serialize(&mut packet[0..12]);

        let first_len = core::cmp::min(total_wire_bytes, max_pkt);
        let first_data_len = core::cmp::min(transfer_size, max_pkt - 12);
        packet[12..12 + first_data_len].copy_from_slice(&data[..first_data_len]);
        if 12 + first_data_len < first_len {
            packet[12 + first_data_len..first_len].fill(0);
        }
        self.write_ep.write(&packet[..first_len]).await?;

        let mut data_offset = first_data_len;
        while data_offset + max_pkt <= transfer_size {
            self.write_ep.write(&data[data_offset..data_offset + max_pkt]).await?;
            data_offset += max_pkt;
        }

        if data_offset < transfer_size || (total_wire_bytes > first_len && padding > 0) {
            let rem_data = transfer_size - data_offset;
            packet[..rem_data].copy_from_slice(&data[data_offset..]);
            packet[rem_data..rem_data + padding].fill(0);
            self.write_ep.write(&packet[..rem_data + padding]).await?;
        }

        if total_wire_bytes % max_pkt == 0 {
            self.write_ep.write(&[]).await?;
        }

        self.shared.add_bytes_tx(total_wire_bytes as u32);
        Ok(())
    }

    /// Transmits a USB488 Service Request (SRQ) notification on the Interrupt-IN endpoint.
    pub async fn send_srq(&mut self, status_byte: u8) -> Result<(), UsbtmcError> {
        let Some(ref mut int_ep) = self.int_ep else {
            return Err(UsbtmcError::NotConnected);
        };
        let packet = [USB488_BNOTIFY1_SRQ, status_byte];
        int_ep.write_transfer(&packet, false).await?;
        Ok(())
    }

    /// Sets the IEEE 488.2 status byte (`*STB`).
    pub fn set_status_byte(&self, stb: u8) {
        self.shared.set_status_byte(stb);
    }

    /// Gets the current IEEE 488.2 status byte (`*STB`).
    pub fn status_byte(&self) -> u8 {
        self.shared.status_byte()
    }

    /// Checks if a clear operation was requested by the host (`viClear()`).
    ///
    /// A clear also clears any STALL condition left by an unparseable message, so the
    /// instrument can be used again without the application calling [`Self::clear_stall`]
    /// itself.
    pub fn check_clear(&mut self) -> bool {
        if self.shared.check_clear() {
            self.clear_stall();
            true
        } else {
            false
        }
    }

    /// Stalls both bulk endpoints.
    ///
    /// The class does this itself when a Bulk-OUT message cannot be parsed; see
    /// [`UsbtmcError`]. Call it to reject a transfer deliberately, for example for a
    /// command the instrument does not support: the host sees the failed transfer
    /// instead of an empty reply, and can abort it with [`Self::clear_stall`],
    /// `CLEAR_FEATURE(ENDPOINT_HALT)` or `viClear()`.
    ///
    /// While the endpoints are stalled, reads and writes do not complete, so the
    /// application has to call [`Self::clear_stall`] (or wait for the host to clear the
    /// halt) before using the class again.
    pub fn stall(&mut self) {
        self.read_ep.set_stall(true);
        self.write_ep.set_stall(true);
    }

    /// Clears the STALL condition on the bulk endpoints.
    ///
    /// Bulk-OUT is stalled when the class cannot parse a message from the host; see
    /// [`UsbtmcError`]. The host normally clears it with `CLEAR_FEATURE(ENDPOINT_HALT)`,
    /// and [`Self::check_clear`] clears it when the host issues `viClear()`. Call this
    /// directly to recover without waiting for either.
    pub fn clear_stall(&mut self) {
        self.read_ep.set_stall(false);
        self.write_ep.set_stall(false);
    }

    /// Returns whether a bulk endpoint is currently stalled.
    pub fn is_stalled(&self) -> bool {
        self.read_ep.is_stalled() || self.write_ep.is_stalled()
    }

    /// Gets the count of indicator pulse requests received.
    pub fn indicator_pulse_count(&self) -> u32 {
        self.shared.indicator_pulse_count()
    }

    /// Gets the count of hardware trigger commands received.
    pub fn trigger_count(&self) -> u32 {
        self.shared.trigger_count()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::cell::RefCell;

    use std::boxed::Box;
    use std::rc::Rc;
    use std::vec::Vec;

    use embassy_futures::block_on;

    use super::*;
    use crate::control::{Recipient, Request, RequestType};
    use crate::driver::{
        Bus, ControlPipe, Direction, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
    };

    #[test]
    fn test_control_capabilities() {
        let caps = Capabilities::default();
        let shared = ControlShared::new(caps, true);
        let mut control = Control {
            comm_if: InterfaceNumber(0),
            iface_string: StringIndex::new(1),
            bulk_out_ep: EndpointAddress::from(0x01),
            bulk_in_ep: EndpointAddress::from(0x81),
            int_in_ep: None,
            shared: unsafe { core::mem::transmute(&shared) },
        };

        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_GET_CAPABILITIES,
            value: 0,
            index: 0,
            length: 24,
        };
        let mut buf = [0u8; 64];
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b.len() == 24));
        assert_eq!(buf[0], UsbtmcStatus::Success as u8);
        assert_eq!(buf[2..4], [0x00, 0x01]); // bcdUSBTMC 1.00
        assert_eq!(buf[12..14], [0x00, 0x01]); // bcdUSB488 1.00
    }

    #[test]
    fn test_control_clear() {
        let caps = Capabilities::default();
        let shared = ControlShared::new(caps, true);
        let mut control = Control {
            comm_if: InterfaceNumber(0),
            iface_string: StringIndex::new(1),
            bulk_out_ep: EndpointAddress::from(0x01),
            bulk_in_ep: EndpointAddress::from(0x81),
            int_in_ep: None,
            shared: unsafe { core::mem::transmute(&shared) },
        };

        // INITIATE_CLEAR
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_INITIATE_CLEAR,
            value: 0,
            index: 0,
            length: 1,
        };
        let mut buf = [0u8; 64];
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b == &[1]));
        assert!(shared.check_clear());

        // CHECK_CLEAR_STATUS
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_CHECK_CLEAR_STATUS,
            value: 0,
            index: 0,
            length: 2,
        };
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b == &[1, 0]));
    }

    #[test]
    fn test_control_status_byte() {
        let caps = Capabilities::default();
        let shared = ControlShared::new(caps, true);
        shared.set_status_byte(0x50); // MAV + SRQ

        let mut control = Control {
            comm_if: InterfaceNumber(1),
            iface_string: StringIndex::new(1),
            bulk_out_ep: EndpointAddress::from(0x02),
            bulk_in_ep: EndpointAddress::from(0x82),
            int_in_ep: None,
            shared: unsafe { core::mem::transmute(&shared) },
        };

        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_USB488_READ_STATUS_BYTE,
            value: 42, // bTag = 42
            index: 1,  // Interface 1
            length: 3,
        };
        let mut buf = [0u8; 64];
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b == &[1, 42, 0x50]));
    }

    #[test]
    fn test_control_abort() {
        let caps = Capabilities::default();
        let shared = ControlShared::new(caps, true);
        shared.add_bytes_rx(128);
        shared.add_bytes_tx(256);

        let mut control = Control {
            comm_if: InterfaceNumber(0),
            iface_string: StringIndex::new(1),
            bulk_out_ep: EndpointAddress::from(0x01),
            bulk_in_ep: EndpointAddress::from(0x81),
            int_in_ep: None,
            shared: unsafe { core::mem::transmute(&shared) },
        };

        // INITIATE_ABORT_BULK_OUT
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Endpoint,
            request: REQ_INITIATE_ABORT_BULK_OUT,
            value: 10,   // bTag to abort
            index: 0x01, // Bulk-OUT EP
            length: 2,
        };
        let mut buf = [0u8; 64];
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b == &[1, 10]));

        // CHECK_ABORT_BULK_OUT_STATUS
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Endpoint,
            request: REQ_CHECK_ABORT_BULK_OUT_STATUS,
            value: 0,
            index: 0x01,
            length: 8,
        };
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(_))));
        assert_eq!(buf[0], 1); // Success
        assert_eq!(u32::from_le_bytes(buf[4..8].try_into().unwrap()), 128);

        // INITIATE_ABORT_BULK_IN
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Endpoint,
            request: REQ_INITIATE_ABORT_BULK_IN,
            value: 11,   // bTag to abort
            index: 0x81, // Bulk-IN EP
            length: 2,
        };
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(b)) if b == &[1, 11]));

        // CHECK_ABORT_BULK_IN_STATUS
        let req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Endpoint,
            request: REQ_CHECK_ABORT_BULK_IN_STATUS,
            value: 0,
            index: 0x81,
            length: 8,
        };
        let res = control.control_in(req, &mut buf);
        assert!(matches!(res, Some(InResponse::Accepted(_))));
        assert_eq!(buf[0], 1); // Success
        assert_eq!(u32::from_le_bytes(buf[4..8].try_into().unwrap()), 256);
    }

    #[test]
    fn test_control_indicator_pulse() {
        let caps = Capabilities::default();
        let shared = ControlShared::new(caps, true);
        let mut control = Control {
            comm_if: InterfaceNumber(0),
            iface_string: StringIndex::new(1),
            bulk_out_ep: EndpointAddress::from(0x01),
            bulk_in_ep: EndpointAddress::from(0x81),
            int_in_ep: None,
            shared: unsafe { core::mem::transmute(&shared) },
        };

        let req = Request {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_INDICATOR_PULSE,
            value: 0,
            index: 0,
            length: 0,
        };
        let res = control.control_out(req, &[]);
        assert!(matches!(res, Some(OutResponse::Accepted)));
        assert_eq!(shared.indicator_pulse_count(), 1);
    }

    // Bulk-OUT stall on messages the class cannot parse.

    #[derive(Default)]
    struct MockState {
        out_stalled: Cell<bool>,
        in_stalled: Cell<bool>,
        fail_read: Cell<bool>,
        packets: RefCell<Vec<Vec<u8>>>,
        written: RefCell<Vec<Vec<u8>>>,
    }

    struct MockEndpoint {
        info: EndpointInfo,
        state: Rc<MockState>,
        dir: Direction,
    }

    impl Endpoint for MockEndpoint {
        fn info(&self) -> &EndpointInfo {
            &self.info
        }

        async fn wait_enabled(&mut self) {}

        fn set_stall(&mut self, stalled: bool) {
            match self.dir {
                Direction::Out => self.state.out_stalled.set(stalled),
                Direction::In => self.state.in_stalled.set(stalled),
            }
        }

        fn is_stalled(&self) -> bool {
            match self.dir {
                Direction::Out => self.state.out_stalled.get(),
                Direction::In => self.state.in_stalled.get(),
            }
        }
    }

    impl EndpointOut for MockEndpoint {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
            if self.state.fail_read.get() {
                return Err(EndpointError::Disabled);
            }
            let packet = self.state.packets.borrow_mut().remove(0);
            buf[..packet.len()].copy_from_slice(&packet);
            Ok(packet.len())
        }
    }

    impl EndpointIn for MockEndpoint {
        async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
            self.state.written.borrow_mut().push(buf.to_vec());
            Ok(())
        }
    }

    struct MockDriver;

    impl<'d> Driver<'d> for MockDriver {
        type EndpointOut = MockEndpoint;
        type EndpointIn = MockEndpoint;
        type ControlPipe = Unused;
        type Bus = Unused;

        fn alloc_endpoint_out(
            &mut self,
            _: EndpointType,
            _: Option<EndpointAddress>,
            _: u16,
            _: u8,
        ) -> Result<MockEndpoint, EndpointAllocError> {
            unreachable!()
        }

        fn alloc_endpoint_in(
            &mut self,
            _: EndpointType,
            _: Option<EndpointAddress>,
            _: u16,
            _: u8,
        ) -> Result<MockEndpoint, EndpointAllocError> {
            unreachable!()
        }

        fn start(self, _: u16) -> (Unused, Unused) {
            unreachable!()
        }
    }

    struct Unused;

    impl Endpoint for Unused {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {
            unreachable!()
        }

        fn set_stall(&mut self, _: bool) {
            unreachable!()
        }

        fn is_stalled(&self) -> bool {
            unreachable!()
        }
    }

    impl EndpointOut for Unused {
        async fn read(&mut self, _: &mut [u8]) -> Result<usize, EndpointError> {
            unreachable!()
        }
    }

    impl EndpointIn for Unused {
        async fn write(&mut self, _: &[u8]) -> Result<(), EndpointError> {
            unreachable!()
        }
    }

    impl Bus for Unused {
        async fn enable(&mut self) {
            unreachable!()
        }

        async fn disable(&mut self) {
            unreachable!()
        }

        async fn poll(&mut self) -> Event {
            unreachable!()
        }

        fn endpoint_set_enabled(&mut self, _: EndpointAddress, _: bool) {
            unreachable!()
        }

        fn endpoint_set_stalled(&mut self, _: EndpointAddress, _: bool) {
            unreachable!()
        }

        fn endpoint_is_stalled(&mut self, _: EndpointAddress) -> bool {
            unreachable!()
        }

        async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
            unreachable!()
        }
    }

    impl ControlPipe for Unused {
        fn max_packet_size(&self) -> usize {
            unreachable!()
        }

        async fn setup(&mut self) -> [u8; 8] {
            unreachable!()
        }

        async fn data_out(&mut self, _: &mut [u8], _: bool, _: bool) -> Result<usize, EndpointError> {
            unreachable!()
        }

        async fn data_in(&mut self, _: &[u8], _: bool, _: bool) -> Result<(), EndpointError> {
            unreachable!()
        }

        async fn accept(&mut self) {
            unreachable!()
        }

        async fn reject(&mut self) {
            unreachable!()
        }

        async fn accept_set_address(&mut self, _: u8) {
            unreachable!()
        }
    }

    fn class(state: &Rc<MockState>) -> UsbtmcClass<'static, MockDriver> {
        let shared: &'static ControlShared = Box::leak(Box::new(ControlShared::new(Capabilities::default(), true)));
        let info = |addr| EndpointInfo {
            addr,
            ep_type: EndpointType::Bulk,
            max_packet_size: 64,
            interval_ms: 0,
        };
        UsbtmcClass {
            read_ep: MockEndpoint {
                info: info(EndpointAddress::from(0x01)),
                state: state.clone(),
                dir: Direction::Out,
            },
            write_ep: MockEndpoint {
                info: info(EndpointAddress::from(0x81)),
                state: state.clone(),
                dir: Direction::In,
            },
            int_ep: None,
            shared,
            pending_request_in: None,
            max_packet_size: 64,
        }
    }

    fn header(msg_id: u8, tag: u8, transfer_size: u32) -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&[msg_id, tag, !tag, 0x00]);
        h.extend_from_slice(&transfer_size.to_le_bytes());
        h.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // EOM
        h
    }

    fn read_fails(tmc: &mut UsbtmcClass<'_, MockDriver>, buf: &mut [u8]) -> UsbtmcError {
        match block_on(tmc.read_message(buf)) {
            Ok(_) => panic!("expected the read to fail"),
            Err(e) => e,
        }
    }

    #[test]
    fn unexpected_msg_id_stalls_bulk_out() {
        let state = Rc::new(MockState::default());
        state.packets.borrow_mut().push(Vec::from([0x42_u8; 12]));
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(
            read_fails(&mut tmc, &mut buf),
            UsbtmcError::UnexpectedMsgId(0x42)
        ));
        assert!(tmc.is_stalled());

        tmc.clear_stall();
        assert!(!tmc.is_stalled());
    }

    #[test]
    fn invalid_tag_stalls_bulk_out() {
        let state = Rc::new(MockState::default());
        let mut packet = header(MSG_ID_DEV_DEP_MSG_OUT, 1, 0);
        packet[2] = 0x00; // bTagInverse must be !bTag
        state.packets.borrow_mut().push(packet);
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(read_fails(&mut tmc, &mut buf), UsbtmcError::InvalidTag));
        assert!(tmc.is_stalled());
    }

    #[test]
    fn short_packet_stalls_bulk_out() {
        let state = Rc::new(MockState::default());
        state.packets.borrow_mut().push(Vec::from([0x01_u8, 0x01, 0xfe, 0x00]));
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(read_fails(&mut tmc, &mut buf), UsbtmcError::PacketTooShort));
        assert!(tmc.is_stalled());
    }

    #[test]
    fn too_small_buffer_stalls_bulk_out() {
        let state = Rc::new(MockState::default());
        let mut packet = header(MSG_ID_DEV_DEP_MSG_OUT, 1, 100);
        packet.extend_from_slice(&[0u8; 4]);
        state.packets.borrow_mut().push(packet);
        let mut tmc = class(&state);

        let mut buf = [0u8; 16];
        assert!(matches!(read_fails(&mut tmc, &mut buf), UsbtmcError::BufferTooSmall));
        assert!(tmc.is_stalled());
    }

    #[test]
    fn truncated_message_stalls_bulk_out() {
        let state = Rc::new(MockState::default());
        let mut packet = header(MSG_ID_DEV_DEP_MSG_OUT, 1, 8);
        packet.extend_from_slice(&[0xaa; 4]);
        state.packets.borrow_mut().push(packet);
        state.packets.borrow_mut().push(Vec::new()); // the host ends the message early
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(read_fails(&mut tmc, &mut buf), UsbtmcError::MessageTruncated));
        assert!(tmc.is_stalled());
    }

    #[test]
    fn endpoint_error_does_not_stall() {
        let state = Rc::new(MockState::default());
        state.fail_read.set(true);
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(
            read_fails(&mut tmc, &mut buf),
            UsbtmcError::Endpoint(EndpointError::Disabled)
        ));
        assert!(!tmc.is_stalled());
    }

    #[test]
    fn valid_message_does_not_stall() {
        let state = Rc::new(MockState::default());
        let mut packet = header(MSG_ID_DEV_DEP_MSG_OUT, 1, 3);
        packet.extend_from_slice(b"abc\0");
        state.packets.borrow_mut().push(packet);
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        match block_on(tmc.read_message(&mut buf)) {
            Ok(msg) => assert_eq!(msg.payload, b"abc"),
            Err(e) => panic!("unexpected error: {}", e),
        }
        assert!(!tmc.is_stalled());
    }

    #[test]
    fn clear_request_clears_stall() {
        let state = Rc::new(MockState::default());
        state.packets.borrow_mut().push(Vec::from([0x42_u8; 12]));
        let mut tmc = class(&state);

        let mut buf = [0u8; 64];
        assert!(matches!(
            read_fails(&mut tmc, &mut buf),
            UsbtmcError::UnexpectedMsgId(0x42)
        ));
        assert!(tmc.is_stalled());

        // viClear() from the host: the class clears the stall by itself.
        tmc.shared.set_clear_pending(true);
        assert!(tmc.check_clear());
        assert!(!tmc.is_stalled());
    }

    #[test]
    fn stall_halts_both_endpoints() {
        let state = Rc::new(MockState::default());
        let mut tmc = class(&state);

        tmc.stall();
        assert!(tmc.is_stalled());
        assert!(state.out_stalled.get());
        assert!(state.in_stalled.get());

        tmc.clear_stall();
        assert!(!tmc.is_stalled());
        assert!(!state.out_stalled.get());
        assert!(!state.in_stalled.get());
    }
}
