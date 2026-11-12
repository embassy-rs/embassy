//! Shared host-side test utilities: mock allocator, loopback pipes, and golden descriptors.
#![allow(missing_docs)]

use core::cell::RefCell;

use embassy_usb_driver::host::{HostError, PipeError, SplitInfo, TimeoutConfig, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{EndpointInfo, Speed};
use heapless::Deque;

use crate::host::descriptor::DeviceDescriptor;
use crate::host::handler::{BusRoute, EnumerationInfo};

/// Golden CDC ACM configuration descriptor.
///
/// Layout: config, IAD, CDC communication interface, interrupt IN endpoint,
/// CDC data interface, bulk IN endpoint, bulk OUT endpoint.
#[rustfmt::skip]
pub const CDC_ACM_CONFIG: &[u8] = &[
    // Configuration descriptor (9 bytes, total 56).
    0x09, 0x02, 0x38, 0x00, 0x02, 0x01, 0x00, 0x80, 0x32,
    // Interface Association Descriptor (8 bytes).
    0x08, 0x0b, 0x00, 0x02, 0x02, 0x02, 0x00, 0x00,
    // Interface 0: CDC ACM communication (9 bytes, 1 endpoint).
    0x09, 0x04, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00,
    // Interrupt IN endpoint (7 bytes).
    0x07, 0x05, 0x81, 0x03, 0x08, 0x00, 0x0a,
    // Interface 1: CDC data (9 bytes, 2 endpoints).
    0x09, 0x04, 0x01, 0x00, 0x02, 0x0a, 0x00, 0x00, 0x00,
    // Bulk IN endpoint (7 bytes).
    0x07, 0x05, 0x82, 0x02, 0x40, 0x00, 0x00,
    // Bulk OUT endpoint (7 bytes).
    0x07, 0x05, 0x02, 0x02, 0x40, 0x00, 0x00,
];

/// Enumeration info matching [`CDC_ACM_CONFIG`].
pub fn enumeration_info() -> EnumerationInfo {
    EnumerationInfo {
        device_address: 1,
        route: BusRoute::Direct(Speed::Full),
        device_desc: DeviceDescriptor {
            bcd_usb: 0x0200,
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            max_packet_size0: 64,
            vendor_id: 0x1234,
            product_id: 0x5678,
            bcd_device: 0x0100,
            manufacturer: 0,
            product: 0,
            serial_number: 0,
            num_configurations: 1,
        },
    }
}

/// Shared mock host state.
pub struct MockState {
    inner: RefCell<MockStateInner>,
}

struct MockStateInner {
    loopback: Deque<u8, 256>,
    line_coding: [u8; 7],
    control_line_state: u16,
    set_interface: Option<(u16, u16)>,
    gtb: heapless::Vec<u8, 256>,
    gtb_request: Option<(u16, u16)>,
    out_ends_transaction: Option<bool>,
    string: heapless::Vec<u8, 64>,
}

impl MockState {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(MockStateInner {
                loopback: Deque::new(),
                line_coding: [0; 7],
                control_line_state: 0,
                set_interface: None,
                gtb: heapless::Vec::new(),
                gtb_request: None,
                out_ends_transaction: None,
                string: heapless::Vec::new(),
            }),
        }
    }

    pub fn loopback_len(&self) -> usize {
        self.inner.borrow().loopback.len()
    }

    pub fn line_coding(&self) -> [u8; 7] {
        self.inner.borrow().line_coding
    }

    pub fn control_line_state(&self) -> u16 {
        self.inner.borrow().control_line_state
    }

    /// Last `SET_INTERFACE` request as `(interface, alternate_setting)`.
    pub fn set_interface(&self) -> Option<(u16, u16)> {
        self.inner.borrow().set_interface
    }

    /// Sets the bytes returned for a MIDI 2.0 Group Terminal Block `GET_DESCRIPTOR`.
    pub fn set_gtb_descriptors(&self, bytes: &[u8]) {
        let mut inner = self.inner.borrow_mut();
        inner.gtb.clear();
        inner.gtb.extend_from_slice(bytes).unwrap();
    }

    /// Sets the string returned for any non-zero string descriptor index; index 0 lists en-US.
    pub fn set_string(&self, text: &str) {
        let mut inner = self.inner.borrow_mut();
        inner.string.clear();
        inner.string.push(0).unwrap();
        inner.string.push(0x03).unwrap();
        for unit in text.encode_utf16() {
            inner.string.extend_from_slice(&unit.to_le_bytes()).unwrap();
        }
        let len = inner.string.len() as u8;
        inner.string[0] = len;
    }

    /// `ensure_transaction_end` of the last bulk OUT request.
    pub fn out_ends_transaction(&self) -> Option<bool> {
        self.inner.borrow().out_ends_transaction
    }

    /// Last Group Terminal Block request as `(wValue, wIndex)`.
    pub fn gtb_request(&self) -> Option<(u16, u16)> {
        self.inner.borrow().gtb_request
    }
}

impl Default for MockState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock host allocator.
#[derive(Clone, Copy)]
pub struct MockHost<'d> {
    state: &'d MockState,
}

impl<'d> MockHost<'d> {
    pub const fn new(state: &'d MockState) -> Self {
        Self { state }
    }
}

impl<'d> UsbHostAllocator<'d> for MockHost<'d> {
    type Pipe<T: pipe::Type, D: pipe::Direction> = MockPipe<'d>;

    fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
        &self,
        _addr: u8,
        _endpoint: &EndpointInfo,
        _split: Option<SplitInfo>,
    ) -> Result<Self::Pipe<T, D>, HostError> {
        Ok(MockPipe { state: self.state })
    }
}

/// Mock pipe. Bulk OUT data is echoed back on bulk IN; control requests
/// implement the CDC ACM line-coding subset used by the class driver.
pub struct MockPipe<'d> {
    state: &'d MockState,
}

impl<T: pipe::Type, D: pipe::Direction> UsbPipe<T, D> for MockPipe<'_> {
    async fn control_in(&mut self, setup: &[u8; 8], buf: &mut [u8]) -> Result<usize, PipeError>
    where
        T: pipe::IsControl,
        D: pipe::IsIn,
    {
        // GET_LINE_CODING: bmRequestType 0xA1, bRequest 0x21.
        if setup[0] == 0xA1 && setup[1] == 0x21 {
            let inner = self.state.inner.borrow();
            let n = buf.len().min(inner.line_coding.len());
            buf[..n].copy_from_slice(&inner.line_coding[..n]);
            Ok(n)
        } else if setup[0] == 0x80 && setup[1] == 0x06 && setup[3] == 0x03 {
            // Standard GET_DESCRIPTOR for a string: index 0 lists the language, others return the set string.
            let inner = self.state.inner.borrow();
            let data: &[u8] = if setup[2] == 0 {
                &[4, 3, 0x09, 0x04]
            } else {
                &inner.string
            };
            let n = buf.len().min(data.len());
            buf[..n].copy_from_slice(&data[..n]);
            Ok(n)
        } else if setup[0] == 0x81 && setup[1] == 0x06 && setup[3] == 0x26 {
            // Standard GET_DESCRIPTOR for a MIDI 2.0 Group Terminal Block.
            let mut inner = self.state.inner.borrow_mut();
            inner.gtb_request = Some((
                u16::from_le_bytes([setup[2], setup[3]]),
                u16::from_le_bytes([setup[4], setup[5]]),
            ));
            let n = buf.len().min(inner.gtb.len());
            buf[..n].copy_from_slice(&inner.gtb[..n]);
            Ok(n)
        } else {
            Err(PipeError::Stall)
        }
    }

    async fn control_out(&mut self, setup: &[u8; 8], buf: &[u8]) -> Result<(), PipeError>
    where
        T: pipe::IsControl,
        D: pipe::IsOut,
    {
        let mut inner = self.state.inner.borrow_mut();
        match setup[1] {
            // SET_LINE_CODING
            0x20 if buf.len() >= 7 => {
                inner.line_coding.copy_from_slice(&buf[..7]);
                Ok(())
            }
            // SET_CONTROL_LINE_STATE
            0x22 => {
                inner.control_line_state = u16::from_le_bytes([setup[2], setup[3]]);
                Ok(())
            }
            // SET_INTERFACE
            0x0b => {
                inner.set_interface = Some((
                    u16::from_le_bytes([setup[4], setup[5]]),
                    u16::from_le_bytes([setup[2], setup[3]]),
                ));
                Ok(())
            }
            _ => Err(PipeError::Stall),
        }
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, PipeError>
    where
        D: pipe::IsIn,
    {
        let mut inner = self.state.inner.borrow_mut();
        let n = buf.len().min(inner.loopback.len());
        for slot in buf.iter_mut().take(n) {
            *slot = inner.loopback.pop_front().unwrap();
        }
        Ok(n)
    }

    async fn request_out(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError>
    where
        D: pipe::IsOut,
    {
        let mut inner = self.state.inner.borrow_mut();
        inner.out_ends_transaction = Some(ensure_transaction_end);
        for &byte in buf {
            inner.loopback.push_back(byte).map_err(|_| PipeError::BufferOverflow)?;
        }
        Ok(())
    }

    fn set_timeout(&mut self, _timeout: TimeoutConfig)
    where
        T: pipe::IsControl,
    {
    }

    fn reset_data_toggle(&mut self)
    where
        T: pipe::IsBulkOrInterrupt,
    {
    }
}
