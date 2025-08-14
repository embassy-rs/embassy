use embassy_usb_driver::Driver;

use crate::{
    control::{InResponse, OutResponse, Recipient, Request as ControlRequest, RequestType},
    Builder,
};

use super::consts::{
    DfuAttributes, Request, State, Status, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_RT,
    USB_CLASS_APPN_SPEC,
};

pub trait Handler {
    fn detach(&mut self) {}
    fn reset(&mut self) {}
}

/// Internal state for the DFU class
pub struct DfuState<H: Handler> {
    handler: H,
    state: State,
    attrs: DfuAttributes,
}

impl<H: Handler> DfuState<H> {
    /// Create a new DFU instance to expose a DFU interface.
    pub fn new(handler: H, attrs: DfuAttributes) -> Self {
        DfuState {
            handler,
            state: State::AppIdle,
            attrs,
        }
    }
}

impl<H: Handler> crate::Handler for DfuState<H> {
    fn reset(&mut self) {
        self.handler.reset()
    }

    fn control_out(&mut self, req: ControlRequest, _: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        trace!("Received out request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::Detach) => {
                trace!("Received DETACH");
                self.state = State::AppDetach;
                self.handler.detach();
                Some(OutResponse::Accepted)
            }
            _ => None,
        }
    }

    fn control_in<'a>(&'a mut self, req: ControlRequest, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        trace!("Received in request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::GetStatus) => {
                buf[0..6].copy_from_slice(&[Status::Ok as u8, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                Some(InResponse::Accepted(buf))
            }
            _ => None,
        }
    }
}

/// An implementation of the USB DFU 1.1 runtime protocol
///
/// This function will add a DFU interface descriptor to the provided Builder, and register the provided Control as a handler for the USB device. The USB builder can be used as normal once this is complete.
/// The handler is responsive to DFU GetStatus and Detach commands.
///
/// Once a detach command, followed by a USB reset is received by the host, a magic number will be written into the bootloader state partition to indicate that
/// it should expose a DFU device, and a software reset will be issued.
///
/// To apply USB DFU updates, the bootloader must be capable of recognizing the DFU magic and exposing a device to handle the full DFU transaction with the host.
pub fn usb_dfu<'d, D: Driver<'d>, H: Handler>(builder: &mut Builder<'d, D>, state: &'d mut DfuState<H>) {
    let mut func = builder.function(0x00, 0x00, 0x00);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_RT, None);
    let timeout_ms = 1000;
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            state.attrs.bits(),
            (timeout_ms & 0xff) as u8,
            ((timeout_ms >> 8) & 0xff) as u8,
            0x40,
            0x00, // 64B control buffer size for application side
            0x10,
            0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(state);
}
