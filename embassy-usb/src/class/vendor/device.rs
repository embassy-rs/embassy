//! Generic Vendor-specific bulk USB class device implementation.
//!
//! Provides raw bulk endpoint data transfer between device and host.
//! Optionally includes:
//! - Microsoft OS 2.0 descriptors for automatic driverless WinUSB binding on Windows 8.1+.
//! - WebUSB BOS platform capability descriptor and landing URL descriptor for browser access.
//! - Async stream I/O via the `embedded-io-async` traits (`Read` and `Write`).

use core::mem::MaybeUninit;

use crate::control::{InResponse, Recipient, Request, RequestType};
use crate::descriptor::capability_type;
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler, msos};

/// USB class code for Vendor Specific devices (0xFF).
pub const USB_CLASS_VENDOR: u8 = 0xFF;

/// Default subclass code (none/vendor-specific).
pub const USB_SUBCLASS_NONE: u8 = 0x00;

/// Default protocol code (none/vendor-specific).
pub const USB_PROTOCOL_NONE: u8 = 0x00;

const WEB_USB_REQUEST_GET_URL: u16 = 0x02;
const WEB_USB_DESCRIPTOR_TYPE_URL: u8 = 0x03;

// WebUSB Platform Capability UUID: {3408b638-09a9-47a0-8bfd-a0768815b665}
const WEB_USB_PLATFORM_CAPABILITY_UUID: [u8; 16] = [
    0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47, 0x8B, 0xFD, 0xA0, 0x76, 0x88, 0x15, 0xB6, 0x65,
];

/// Vendor class error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VendorError {
    /// USB is not connected or endpoint is disabled.
    NotConnected,
    /// Provided buffer overflowed.
    BufferOverflow,
}

impl core::fmt::Display for VendorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::NotConnected => f.write_str("NotConnected"),
            Self::BufferOverflow => f.write_str("BufferOverflow"),
        }
    }
}

impl core::error::Error for VendorError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for VendorError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match *self {
            Self::NotConnected => embedded_io_async::ErrorKind::NotConnected,
            Self::BufferOverflow => embedded_io_async::ErrorKind::OutOfMemory,
        }
    }
}

/// URL descriptor for WebUSB landing page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Url<'d>(&'d str, u8);

impl<'d> Url<'d> {
    /// Create a new WebUSB URL descriptor.
    pub fn new(url: &'d str) -> Self {
        let (prefix, stripped_url) = if let Some(stripped) = url.strip_prefix("https://") {
            (1, stripped)
        } else if let Some(stripped) = url.strip_prefix("http://") {
            (0, stripped)
        } else {
            (255, url)
        };
        assert!(
            stripped_url.len() <= 252,
            "URL too long ({} bytes). Maximum length is 252 bytes.",
            stripped_url.len()
        );
        Self(stripped_url, prefix)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub(crate) fn scheme(&self) -> u8 {
        self.1
    }
}

/// Configuration for the Vendor class.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<'d> {
    /// Vendor subclass code (default: `0x00`).
    pub subclass: u8,
    /// Vendor protocol code (default: `0x00`).
    pub protocol: u8,
    /// Optional interface string description.
    pub interface_name: Option<&'d str>,
    /// Optional Microsoft OS 2.0 WinUSB Device Interface GUIDs.
    ///
    /// When specified, Microsoft OS 2.0 descriptors are automatically generated with
    /// Compatible ID "WINUSB" and the provided GUIDs. Windows 8.1+ will automatically
    /// load `winusb.sys` without an INF file or manual driver installation.
    ///
    /// Example: `&["{CDB3B5AD-293B-4663-AA36-1AAE46463776}"]`
    pub msos_interface_guids: Option<&'d [&'d str]>,
    /// Whether WebUSB platform capability should be registered.
    pub webusb: bool,
    /// Optional WebUSB URL landing page.
    pub webusb_landing_url: Option<Url<'d>>,
    /// WebUSB vendor request code (default: `0x01`).
    pub webusb_vendor_code: u8,
    /// Options for the bulk OUT endpoint, which receives the data the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the bulk IN endpoint, which sends data to the host.
    pub in_endpoint_options: EndpointOptions,
}

impl<'d> Config<'d> {
    /// Create a new `Config` with default settings.
    pub const fn new() -> Self {
        Self {
            subclass: USB_SUBCLASS_NONE,
            protocol: USB_PROTOCOL_NONE,
            interface_name: None,
            msos_interface_guids: None,
            webusb: false,
            webusb_landing_url: None,
            webusb_vendor_code: 0x01,
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
        }
    }

    /// Set the interface string description.
    pub const fn with_interface_name(mut self, name: &'d str) -> Self {
        self.interface_name = Some(name);
        self
    }

    /// Set Microsoft OS 2.0 WinUSB device interface GUIDs.
    pub const fn with_winusb(mut self, guids: &'d [&'d str]) -> Self {
        self.msos_interface_guids = Some(guids);
        self
    }

    /// Enable WebUSB platform capability with an optional landing URL and vendor request code.
    pub const fn with_webusb(mut self, landing_url: Option<Url<'d>>, vendor_code: u8) -> Self {
        self.webusb = true;
        self.webusb_landing_url = landing_url;
        self.webusb_vendor_code = vendor_code;
        self
    }

    /// Set custom subclass and protocol codes.
    pub const fn with_subclass_protocol(mut self, subclass: u8, protocol: u8) -> Self {
        self.subclass = subclass;
        self.protocol = protocol;
        self
    }
}

impl<'d> Default for Config<'d> {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal state for the Vendor USB class.
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
    iface: InterfaceNumber,
    iface_string: Option<StringIndex>,
    interface_name: Option<&'d str>,
    webusb_vendor_code: u8,
    webusb_landing_url: Option<Url<'d>>,
    ep_buf: [u8; 128],
}

impl<'d> Handler for Control<'d> {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if self.iface_string == Some(index) {
            self.interface_name
        } else {
            None
        }
    }

    fn control_in(&mut self, req: Request, _data: &mut [u8]) -> Option<InResponse<'_>> {
        if let Some(ref url) = self.webusb_landing_url {
            if req.request_type == RequestType::Vendor
                && req.recipient == Recipient::Device
                && req.request == self.webusb_vendor_code
                && req.value == 1
                && req.index == WEB_USB_REQUEST_GET_URL
            {
                let url_bytes = url.as_bytes();
                let len = url_bytes.len();
                self.ep_buf[0] = len as u8 + 3;
                self.ep_buf[1] = WEB_USB_DESCRIPTOR_TYPE_URL;
                self.ep_buf[2] = url.scheme();
                self.ep_buf[3..3 + len].copy_from_slice(url_bytes);
                return Some(InResponse::Accepted(&self.ep_buf[..3 + len]));
            }
        }
        None
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface
    }
}

/// Vendor-specific bulk USB class.
///
/// Implements bidirectional communication over one Bulk IN and one Bulk OUT endpoint.
pub struct VendorClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    iface: InterfaceNumber,
}

impl<'d, D: Driver<'d>> VendorClass<'d, D> {
    /// Creates a new `VendorClass` with the provided configuration.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let iface_string = config.interface_name.map(|_| builder.string());

        if config.msos_interface_guids.is_some() {
            if builder.msos_writer().is_empty() {
                builder.msos_descriptor(msos::windows_version::WIN8_1, 0x02);
            }
        }

        let mut func = builder.function(USB_CLASS_VENDOR, config.subclass, config.protocol);

        if let Some(guids) = config.msos_interface_guids {
            func.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
            func.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
                "DeviceInterfaceGUIDs",
                msos::PropertyData::RegMultiSz(guids),
            ));
        }

        let mut iface = func.interface();
        let interface_number = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_VENDOR, config.subclass, config.protocol, iface_string);

        if config.webusb || config.webusb_landing_url.is_some() {
            let mut bos_cap = [0u8; 21];
            bos_cap[0] = 0x00; // bReserved
            bos_cap[1..17].copy_from_slice(&WEB_USB_PLATFORM_CAPABILITY_UUID);
            bos_cap[17] = 0x00; // bcdVersion (1.0) LSB
            bos_cap[18] = 0x01; // bcdVersion (1.0) MSB
            bos_cap[19] = config.webusb_vendor_code;
            bos_cap[20] = if config.webusb_landing_url.is_some() { 1 } else { 0 };

            alt.bos_capability(capability_type::PLATFORM, &bos_cap);
        }

        let read_ep = alt.endpoint_bulk_out(None, config.out_endpoint_options);
        let write_ep = alt.endpoint_bulk_in(None, config.in_endpoint_options);

        drop(func);

        let control = state.control.write(Control {
            iface: interface_number,
            iface_string,
            interface_name: config.interface_name,
            webusb_vendor_code: config.webusb_vendor_code,
            webusb_landing_url: config.webusb_landing_url,
            ep_buf: [0u8; 128],
        });
        builder.interface_handler(control);

        VendorClass {
            read_ep,
            write_ep,
            iface: interface_number,
        }
    }

    /// Gets the interface number.
    pub fn interface(&self) -> InterfaceNumber {
        self.iface
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
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

    /// Writes a multi-packet transfer into the IN endpoint.
    pub async fn write_transfer(&mut self, data: &[u8], needs_zlp: bool) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(data, needs_zlp).await
    }

    /// Reads a multi-packet transfer from the OUT endpoint until a short packet is received.
    pub async fn read_transfer(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(data).await
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Split the class into a sender and receiver for concurrent operation across tasks.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
                needs_zlp: false,
            },
            Receiver { read_ep: self.read_ep },
        )
    }
}

/// Vendor class packet sender.
///
/// You can obtain a `Sender` with [`VendorClass::split`].
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    needs_zlp: bool,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.write_ep.info().max_packet_size
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await?;
        self.needs_zlp = data.len() == self.max_packet_size() as usize;
        Ok(())
    }

    /// Writes a multi-packet transfer into the IN endpoint.
    pub async fn write_transfer(&mut self, data: &[u8], needs_zlp: bool) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(data, needs_zlp).await
    }

    /// Waits for the USB host to enable this endpoint.
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await;
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for Sender<'d, D> {
    type Error = VendorError;
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Write for Sender<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let max_packet_size = self.max_packet_size() as usize;
        match self.write_transfer(buf, false).await {
            Ok(()) => {
                self.needs_zlp = !buf.is_empty() && (buf.len() % max_packet_size == 0);
                Ok(buf.len())
            }
            Err(EndpointError::BufferOverflow) => unreachable!(),
            Err(EndpointError::Disabled) => Err(VendorError::NotConnected),
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        if self.needs_zlp {
            self.write_transfer(&[], false).await.map_err(|e| match e {
                EndpointError::BufferOverflow => unreachable!(),
                EndpointError::Disabled => VendorError::NotConnected,
            })?;
            self.needs_zlp = false;
        }
        Ok(())
    }
}

/// Vendor class packet receiver.
///
/// You can obtain a `Receiver` with [`VendorClass::split`].
pub struct Receiver<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.read_ep.info().max_packet_size
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Reads a multi-packet transfer from the OUT endpoint until a short packet is received.
    pub async fn read_transfer(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(data).await
    }

    /// Waits for the USB host to enable this endpoint.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Turn the `Receiver` into a [`BufferedReceiver`] to support arbitrary byte-length streaming.
    ///
    /// The supplied buffer must be large enough to hold `max_packet_size` bytes.
    pub fn into_buffered(self, buf: &'d mut [u8]) -> BufferedReceiver<'d, D> {
        BufferedReceiver {
            receiver: self,
            buffer: buf,
            start: 0,
            end: 0,
        }
    }
}

/// Vendor class buffered receiver for stream-like reads.
///
/// Implements [`embedded_io_async::Read`] allowing arbitrarily sized chunks of data to be read.
pub struct BufferedReceiver<'d, D: Driver<'d>> {
    receiver: Receiver<'d, D>,
    buffer: &'d mut [u8],
    start: usize,
    end: usize,
}

impl<'d, D: Driver<'d>> BufferedReceiver<'d, D> {
    fn read_from_buffer(&mut self, buf: &mut [u8]) -> usize {
        let available = &self.buffer[self.start..self.end];
        let len = core::cmp::min(available.len(), buf.len());
        buf[..len].copy_from_slice(&available[..len]);
        self.start += len;
        len
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.receiver.max_packet_size()
    }

    /// Waits for the USB host to enable this endpoint.
    pub async fn wait_connection(&mut self) {
        self.receiver.wait_connection().await;
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for BufferedReceiver<'d, D> {
    type Error = VendorError;
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Read for BufferedReceiver<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.start != self.end {
            return Ok(self.read_from_buffer(buf));
        }

        if buf.len() > self.receiver.max_packet_size() as usize {
            return match self.receiver.read_transfer(buf).await {
                Ok(n) => Ok(n),
                Err(EndpointError::BufferOverflow) => unreachable!(),
                Err(EndpointError::Disabled) => Err(VendorError::NotConnected),
            };
        }

        match self.receiver.read_packet(self.buffer).await {
            Ok(n) => self.end = n,
            Err(EndpointError::BufferOverflow) => unreachable!(),
            Err(EndpointError::Disabled) => return Err(VendorError::NotConnected),
        }
        self.start = 0;
        Ok(self.read_from_buffer(buf))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::format;

    use super::*;

    #[test]
    fn test_url_parsing() {
        let https_url = Url::new("https://example.com/test");
        assert_eq!(https_url.scheme(), 1);
        assert_eq!(https_url.as_bytes(), b"example.com/test");

        let http_url = Url::new("http://example.com/test");
        assert_eq!(http_url.scheme(), 0);
        assert_eq!(http_url.as_bytes(), b"example.com/test");

        let custom_url = Url::new("custom.url");
        assert_eq!(custom_url.scheme(), 255);
        assert_eq!(custom_url.as_bytes(), b"custom.url");
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_interface_name("Vendor Bulk")
            .with_subclass_protocol(0x01, 0x02)
            .with_winusb(&["{12345678-1234-1234-1234-123456789ABC}"])
            .with_webusb(Some(Url::new("https://example.com")), 0x22);

        assert_eq!(config.interface_name, Some("Vendor Bulk"));
        assert_eq!(config.subclass, 0x01);
        assert_eq!(config.protocol, 0x02);
        assert_eq!(
            config.msos_interface_guids,
            Some(&["{12345678-1234-1234-1234-123456789ABC}"][..])
        );
        assert!(config.webusb);
        assert_eq!(config.webusb_vendor_code, 0x22);
    }

    #[test]
    fn test_control_handler() {
        let mut control = Control {
            iface: InterfaceNumber(0),
            iface_string: Some(StringIndex::new(1)),
            interface_name: Some("Vendor Bulk"),
            webusb_vendor_code: 0x42,
            webusb_landing_url: Some(Url::new("https://example.com")),
            ep_buf: [0u8; 128],
        };

        // String lookup
        assert_eq!(control.get_string(StringIndex::new(1), 0x0409), Some("Vendor Bulk"));
        assert_eq!(control.get_string(StringIndex::new(2), 0x0409), None);

        // Interface ownership
        assert!(control.owns(InterfaceNumber(0)));
        assert!(!control.owns(InterfaceNumber(1)));

        // Non-vendor request rejected
        let std_req = Request {
            direction: crate::driver::Direction::In,
            request_type: RequestType::Standard,
            recipient: Recipient::Device,
            request: 0x42,
            value: 1,
            index: WEB_USB_REQUEST_GET_URL,
            length: 128,
        };
        let mut buf = [0u8; 128];
        assert!(control.control_in(std_req, &mut buf).is_none());

        // WebUSB URL vendor request accepted
        let webusb_req = Request {
            direction: crate::driver::Direction::In,
            request_type: RequestType::Vendor,
            recipient: Recipient::Device,
            request: 0x42,
            value: 1,
            index: WEB_USB_REQUEST_GET_URL,
            length: 128,
        };
        let resp = control.control_in(webusb_req, &mut buf);
        match resp {
            Some(InResponse::Accepted(data)) => {
                // Length byte = 3 + len("example.com") = 3 + 11 = 14
                assert_eq!(data[0], 14);
                assert_eq!(data[1], WEB_USB_DESCRIPTOR_TYPE_URL);
                assert_eq!(data[2], 1); // https prefix code
                assert_eq!(&data[3..], b"example.com");
            }
            _ => panic!("Expected Accepted InResponse"),
        }
    }

    #[test]
    fn test_vendor_error_traits() {
        assert_eq!(format!("{}", VendorError::NotConnected), "NotConnected");
        assert_eq!(format!("{}", VendorError::BufferOverflow), "BufferOverflow");

        #[cfg(feature = "embedded-io")]
        {
            use embedded_io_async::Error;
            assert_eq!(
                VendorError::NotConnected.kind(),
                embedded_io_async::ErrorKind::NotConnected
            );
            assert_eq!(
                VendorError::BufferOverflow.kind(),
                embedded_io_async::ErrorKind::OutOfMemory
            );
        }
    }
}
