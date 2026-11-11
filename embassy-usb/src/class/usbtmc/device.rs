//! USB Test & Measurement Class (USBTMC) device implementation.

use core::cell::Cell;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::types::*;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointAddress, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler};

/// Configuration for the USBTMC device class.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Whether to enable the USB488 subclass protocol.
    pub usb488: bool,
    /// Whether to allocate an Interrupt-IN endpoint for SRQ notifications.
    pub interrupt_endpoint: bool,
    /// Polling interval in ms for the Interrupt-IN endpoint.
    pub interrupt_poll_ms: u8,
    /// Capabilities reported to the host in `GET_CAPABILITIES`.
    pub capabilities: Capabilities,
    /// Options for the bulk OUT endpoint, which receives the data the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the bulk IN endpoint, which sends data to the host.
    pub in_endpoint_options: EndpointOptions,
}

impl Config {
    /// Creates a default USBTMC 1.0 configuration without USB488 extensions.
    pub const fn new() -> Self {
        Self {
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
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
        }
    }

    /// Creates a default configuration with USB488 and SCPI support enabled.
    pub const fn new_usb488() -> Self {
        Self {
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
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
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

impl InterfaceHandler for Control {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.comm_if
    }
}

/// USB device class for USB Test & Measurement Class (USBTMC).
pub struct UsbtmcClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    int_ep: Option<D::EndpointIn>,
    shared: &'d ControlShared,
    pending_request_in: Option<RequestDevDepMsgInHeader>,
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
        let read_ep = alt.endpoint_bulk_out(None, config.out_endpoint_options);
        let write_ep = alt.endpoint_bulk_in(None, config.in_endpoint_options);
        assert!(
            read_ep.info().max_packet_size <= 512,
            "USBTMC supports bulk packets of up to 512 bytes"
        );
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

        builder.interface_handler(state.control.write(Control {
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
        }
    }

    /// Waits until the USB host configures and enables this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    async fn read_message_payload<'a>(
        &mut self,
        buf: &'a mut [u8],
        packet_buf: &mut [u8],
        n: usize,
    ) -> Result<ReceivedMessage<'a>, UsbtmcError> {
        let max_pkt = self.read_ep.info().max_packet_size as usize;
        let header = DevDepMsgOutHeader::parse(packet_buf[0..12].try_into().unwrap())?;
        let transfer_size = header.transfer_size as usize;
        if transfer_size > buf.len() {
            return Err(UsbtmcError::BufferTooSmall);
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
        let max_pkt = self.read_ep.info().max_packet_size as usize;
        let packet_buf = &mut packet[..max_pkt];
        let n = self.read_ep.read(packet_buf).await?;
        if n < 12 {
            return Err(UsbtmcError::PacketTooShort);
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
                let header = RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap())?;
                self.shared.add_bytes_rx(n as u32);
                Ok(UsbtmcEvent::RequestIn(header))
            }
            MSG_ID_TRIGGER => {
                let base = MsgHeader::parse(packet_buf[0..4].try_into().unwrap())?;
                self.shared.add_bytes_rx(n as u32);
                self.shared.trigger();
                Ok(UsbtmcEvent::Trigger(base.b_tag))
            }
            id => Err(UsbtmcError::UnexpectedMsgId(id)),
        }
    }

    /// Reads the next command message from the host.
    ///
    /// If an incoming `REQUEST_DEV_DEP_MSG_IN` or `TRIGGER` event is encountered, it is
    /// recorded in the internal state before continuing to wait for a command message.
    pub async fn read_message<'a>(&mut self, buf: &'a mut [u8]) -> Result<ReceivedMessage<'a>, UsbtmcError> {
        loop {
            let mut packet = [0u8; 512];
            let max_pkt = self.read_ep.info().max_packet_size as usize;
            let packet_buf = &mut packet[..max_pkt];
            let n = self.read_ep.read(packet_buf).await?;
            if n < 12 {
                return Err(UsbtmcError::PacketTooShort);
            }

            match packet_buf[0] {
                MSG_ID_DEV_DEP_MSG_OUT | MSG_ID_VENDOR_SPECIFIC_OUT => {
                    return self.read_message_payload(buf, packet_buf, n).await;
                }
                MSG_ID_REQUEST_DEV_DEP_MSG_IN | MSG_ID_REQUEST_VENDOR_SPECIFIC_IN => {
                    let header = RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap())?;
                    self.shared.add_bytes_rx(n as u32);
                    self.pending_request_in = Some(header);
                }
                MSG_ID_TRIGGER => {
                    let _base = MsgHeader::parse(packet_buf[0..4].try_into().unwrap())?;
                    self.shared.add_bytes_rx(n as u32);
                    self.shared.trigger();
                }
                id => return Err(UsbtmcError::UnexpectedMsgId(id)),
            }
        }
    }

    /// Waits for the host to send a `REQUEST_DEV_DEP_MSG_IN` request on Bulk-OUT.
    pub async fn wait_request_in(&mut self) -> Result<RequestDevDepMsgInHeader, UsbtmcError> {
        if let Some(req) = self.pending_request_in.take() {
            return Ok(req);
        }
        let mut packet = [0u8; 512];
        let max_pkt = self.read_ep.info().max_packet_size as usize;
        let packet_buf = &mut packet[..max_pkt];
        loop {
            let n = self.read_ep.read(packet_buf).await?;
            if n < 12 {
                return Err(UsbtmcError::PacketTooShort);
            }
            match packet_buf[0] {
                MSG_ID_REQUEST_DEV_DEP_MSG_IN | MSG_ID_REQUEST_VENDOR_SPECIFIC_IN => {
                    let header = RequestDevDepMsgInHeader::parse(packet_buf[0..12].try_into().unwrap())?;
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
        let max_pkt = self.write_ep.info().max_packet_size as usize;

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
    pub fn check_clear(&self) -> bool {
        self.shared.check_clear()
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
    use super::*;
    use crate::control::{Recipient, Request, RequestType};
    use crate::driver::Direction;

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
}
