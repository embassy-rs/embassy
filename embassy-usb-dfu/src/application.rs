use core::marker::PhantomData;

use embassy_boot::BlockingFirmwareState;
use embassy_time::{Duration, Instant};
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, Handler};
use embedded_storage::nor_flash::NorFlash;

use crate::consts::{
    DfuAttributes, Request, State, Status, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_RT,
    USB_CLASS_APPN_SPEC,
};
use crate::Reset;

/// Internal state for the DFU class
pub struct Control<'d, STATE: NorFlash, RST: Reset> {
    firmware_state: BlockingFirmwareState<'d, STATE>,
    attrs: DfuAttributes,
    state: State,
    timeout: Option<Duration>,
    detach_start: Option<Instant>,
    _rst: PhantomData<RST>,
}

impl<'d, STATE: NorFlash, RST: Reset> Control<'d, STATE, RST> {
    /// Create a new DFU instance to expose a DFU interface.
    pub fn new(firmware_state: BlockingFirmwareState<'d, STATE>, attrs: DfuAttributes) -> Self {
        Control {
            firmware_state,
            attrs,
            state: State::AppIdle,
            detach_start: None,
            timeout: None,
            _rst: PhantomData,
        }
    }
}

impl<'d, STATE: NorFlash, RST: Reset> Handler for Control<'d, STATE, RST> {
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
                self.firmware_state
                    .mark_dfu()
                    .expect("Failed to mark DFU mode in bootloader");
                RST::sys_reset()
            }
        }
    }

    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        _: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        trace!("Received request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::Detach) => {
                trace!("Received DETACH, awaiting USB reset");
                self.detach_start = Some(Instant::now());
                self.timeout = Some(Duration::from_millis(req.value as u64));
                self.state = State::AppDetach;
                Some(OutResponse::Accepted)
            }
            _ => None,
        }
    }

    fn control_in<'a>(
        &'a mut self,
        req: embassy_usb::control::Request,
        buf: &'a mut [u8],
    ) -> Option<embassy_usb::control::InResponse<'a>> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        trace!("Received request {}", req);

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
pub fn usb_dfu<'d, D: Driver<'d>, STATE: NorFlash, RST: Reset>(
    builder: &mut Builder<'d, D>,
    handler: &'d mut Control<'d, STATE, RST>,
    timeout: Duration,
) {
    let mut func = builder.function(0x00, 0x00, 0x00);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_RT, None);
    let timeout = timeout.as_millis() as u16;
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            handler.attrs.bits(),
            (timeout & 0xff) as u8,
            ((timeout >> 8) & 0xff) as u8,
            0x40,
            0x00, // 64B control buffer size for application side
            0x10,
            0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(handler);
}
