//! WebUSB API capability implementation.
//!
//! See https://wicg.github.io/webusb

use core::mem::MaybeUninit;

use crate::control::{InResponse, Recipient, Request, RequestType};
use crate::descriptor::capability_type;
use crate::driver::Driver;
use crate::{Builder, Handler};

const USB_CLASS_VENDOR: u8 = 0xff;
const USB_SUBCLASS_NONE: u8 = 0x00;
const USB_PROTOCOL_NONE: u8 = 0x00;

const WEB_USB_REQUEST_GET_URL: u16 = 0x02;
const WEB_USB_DESCRIPTOR_TYPE_URL: u8 = 0x03;

/// URL descriptor for WebUSB landing page.
///
/// An ecoded URL descriptor to point to a website that is suggested to the user when the device is connected.
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
            "URL too long. ({} bytes). Maximum length is 252 bytes.",
            stripped_url.len()
        );
        Self(stripped_url, prefix)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn scheme(&self) -> u8 {
        self.1
    }
}

/// Configuration for WebUSB.
pub struct Config<'d> {
    /// Maximum packet size in bytes for the data endpoints.
    ///
    /// Valid values depend on the speed at which the bus is enumerated.
    /// - low speed: 8
    /// - full speed: 8, 16, 32, or 64
    /// - high speed: 64
    pub max_packet_size: u16,
    /// URL to navigate to when the device is connected.
    ///
    /// If defined, shows a landing page which the device manufacturer would like the user to visit in order to control their device.
    pub landing_url: Option<Url<'d>>,
    /// Vendor code for the WebUSB request.
    ///
    /// This value defines the request id (bRequest) the device expects the host to use when issuing control transfers these requests. This can be an arbitrary u8 and is not to be confused with the USB Vendor ID.
    pub vendor_code: u8,
}

struct Control<'d> {
    ep_buf: [u8; 128],
    vendor_code: u8,
    landing_url: Option<&'d Url<'d>>,
}

impl<'d> Control<'d> {
    fn new(config: &'d Config<'d>) -> Self {
        Control {
            ep_buf: [0u8; 128],
            vendor_code: config.vendor_code,
            landing_url: config.landing_url.as_ref(),
        }
    }
}

impl<'d> Handler for Control<'d> {
    fn control_in(&mut self, req: Request, _data: &mut [u8]) -> Option<InResponse> {
        let landing_value = if self.landing_url.is_some() { 1 } else { 0 };
        if req.request_type == RequestType::Vendor
            && req.recipient == Recipient::Device
            && req.request == self.vendor_code
            && req.value == landing_value
            && req.index == WEB_USB_REQUEST_GET_URL
        {
            if let Some(url) = self.landing_url {
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

/// Internal state for WebUSB
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
        State {
            control: MaybeUninit::uninit(),
        }
    }
}

/// WebUSB capability implementation.
///
/// WebUSB is a W3C standard that allows a web page to communicate with USB devices.
/// See See https://wicg.github.io/webusb for more information and the browser API.
/// This implementation provides one read and one write endpoint.
pub struct WebUsb<'d, D: Driver<'d>> {
    _driver: core::marker::PhantomData<&'d D>,
}

impl<'d, D: Driver<'d>> WebUsb<'d, D> {
    /// Builder for the WebUSB capability implementation.
    ///
    /// Pass in a USB `Builder`, a `State`, which holds the control endpoint state, and a `Config` for the WebUSB configuration.
    pub fn configure(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: &'d Config<'d>) {
        let mut func = builder.function(USB_CLASS_VENDOR, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE);
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(USB_CLASS_VENDOR, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE, None);

        alt.bos_capability(
            capability_type::PLATFORM,
            &[
                // PlatformCapabilityUUID (3408b638-09a9-47a0-8bfd-a0768815b665)
                0x0,
                0x38,
                0xb6,
                0x08,
                0x34,
                0xa9,
                0x09,
                0xa0,
                0x47,
                0x8b,
                0xfd,
                0xa0,
                0x76,
                0x88,
                0x15,
                0xb6,
                0x65,
                // bcdVersion of WebUSB (1.0)
                0x00,
                0x01,
                // bVendorCode
                config.vendor_code,
                // iLandingPage
                if config.landing_url.is_some() { 1 } else { 0 },
            ],
        );

        let control = state.control.write(Control::new(config));

        drop(func);

        builder.handler(control);
    }
}
