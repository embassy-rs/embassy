//! USB DFU application mode.

use embassy_time::{Duration, Instant};
use embassy_usb_driver::Driver;

use crate::{
    class::dfu::consts::STATUS_OK,
    control::{InResponse, OutResponse, Recipient, Request as ControlRequest, RequestType},
    Builder, FunctionBuilder,
};

use super::consts::{
    DfuAttributes, Request, State, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_RT, USB_CLASS_APPN_SPEC,
};

/// TODO
pub trait Handler {
    /// TODO
    fn switch_to_dfu(&mut self) {}
}

/// Internal state for the DFU class
pub struct DfuState<H: Handler> {
    handler: H,
    state: State,
    attrs: DfuAttributes,
    timeout: Option<Duration>,
    detach_start: Option<Instant>,
}

impl<H: Handler> DfuState<H> {
    /// Create a new DFU instance to expose a DFU interface.
    pub fn new(handler: H, attrs: DfuAttributes) -> Self {
        DfuState {
            handler,
            state: State::AppIdle,
            attrs,
            timeout: None,
            detach_start: None,
        }
    }
}

impl<H: Handler> crate::Handler for DfuState<H> {
    fn reset(&mut self) {
        if let Some(start) = self.detach_start {
            let delta = Instant::now() - start;
            let timeout = self.timeout.unwrap();
            trace!(
                "Received RESET with delta = {}, timeout = {}",
                delta.as_millis(),
                timeout.as_millis()
            );
            if delta < timeout {
                self.handler.switch_to_dfu();
            }
        }
    }

    fn control_out(&mut self, req: ControlRequest, _: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        trace!("Received out request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::Detach) => {
                self.detach_start = Some(Instant::now());
                self.timeout = Some(Duration::from_millis(req.value as u64));
                self.state = State::AppDetach;
                if self.attrs.contains(DfuAttributes::WILL_DETACH) {
                    trace!("Received DETACH, performing reset");
                    self.reset();
                } else {
                    trace!("Received DETACH, awaiting USB reset");
                }
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
                buf[0..6].copy_from_slice(&[STATUS_OK, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
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
pub fn usb_dfu<'d, D: Driver<'d>, H: Handler>(
    builder: &mut Builder<'d, D>,
    state: &'d mut DfuState<H>,
    timeout: Duration,
    func_modifier: impl Fn(&mut FunctionBuilder<'_, 'd, D>),
) {
    let mut func = builder.function(0x00, 0x00, 0x00);

    // Here we give users the opportunity to add their own function level MSOS headers for instance.
    // This is useful when DFU functionality is part of a composite USB device.
    func_modifier(&mut func);

    let mut iface = func.interface();
    let mut alt = iface.alt_setting(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_RT, None);
    let timeout = timeout.as_millis() as u16;
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            state.attrs.bits(),
            (timeout & 0xff) as u8,
            ((timeout >> 8) & 0xff) as u8,
            0x40,
            0x00, // 64B control buffer size for application side
            0x10,
            0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(state);
}
