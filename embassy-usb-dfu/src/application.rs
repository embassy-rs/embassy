
use embassy_boot::BlockingFirmwareUpdater;
use embassy_time::{Instant, Duration};
use embassy_usb::{Handler, control::{RequestType, Recipient, OutResponse, InResponse}, Builder, driver::Driver};
use embedded_storage::nor_flash::NorFlash;

use crate::consts::{DfuAttributes, Request, Status, State, USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_RT};

/// Internal state for the DFU class
pub struct Control<'d, DFU: NorFlash, STATE: NorFlash> {
    updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
    attrs: DfuAttributes,
    state: State,
    timeout: Option<Duration>,
    detach_start: Option<Instant>,
}

impl<'d, DFU: NorFlash, STATE: NorFlash> Control<'d, DFU, STATE> {
    pub fn new(updater: BlockingFirmwareUpdater<'d, DFU, STATE>, attrs: DfuAttributes) -> Self {
        Control { updater, attrs, state: State::AppIdle, detach_start: None, timeout: None }
    }
}

impl<'d, DFU: NorFlash, STATE: NorFlash> Handler for Control<'d, DFU, STATE> {
    fn reset(&mut self) {
        if let Some(start) = self.detach_start {
            let delta = Instant::now() - start;
            let timeout = self.timeout.unwrap();
            #[cfg(feature = "defmt")]
            defmt::info!("Received RESET with delta = {}, timeout = {}", delta.as_millis(), timeout.as_millis());
            if delta < timeout {
                self.updater.mark_dfu().expect("Failed to mark DFU mode in bootloader");
                cortex_m::asm::dsb();
                cortex_m::peripheral::SCB::sys_reset();
            }
        }
    }

    fn control_out(&mut self, req: embassy_usb::control::Request, _: &[u8]) -> Option<embassy_usb::control::OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Received request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::Detach) => {
                #[cfg(feature = "defmt")]
                defmt::info!("Received DETACH, awaiting USB reset");
                self.detach_start = Some(Instant::now());
                self.timeout = Some(Duration::from_millis(req.value as u64));
                self.state = State::AppDetach;
                Some(OutResponse::Accepted)
            }
            _ => {
                None
            }
        }
    }

    fn control_in<'a>(&'a mut self, req: embassy_usb::control::Request, buf: &'a mut [u8]) -> Option<embassy_usb::control::InResponse<'a>> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Received request {}", req);

        match Request::try_from(req.request) {
            Ok(Request::GetStatus) => {
                buf[0..6].copy_from_slice(&[Status::Ok as u8, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                Some(InResponse::Accepted(buf))
            }
            _ => None
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
pub fn usb_dfu<'d, D: Driver<'d>, DFU: NorFlash, STATE: NorFlash>(builder: &mut Builder<'d, D>, handler: &'d mut Control<'d, DFU, STATE>, timeout: Duration) {
    #[cfg(feature = "defmt")]
    defmt::info!("Application USB DFU initializing");
    let mut func = builder.function(0x00, 0x00, 0x00);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(
        USB_CLASS_APPN_SPEC,
        APPN_SPEC_SUBCLASS_DFU,
        DFU_PROTOCOL_RT,
        None,
    );
    let timeout = timeout.as_millis() as u16;
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            handler.attrs.bits(),
            (timeout & 0xff) as u8,
            ((timeout >> 8) & 0xff) as u8,
            0x40, 0x00, // 64B control buffer size for application side
            0x10, 0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(handler); 
}