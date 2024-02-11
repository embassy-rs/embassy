use core::marker::PhantomData;

use embassy_boot::{AlignedBuffer, BlockingFirmwareUpdater};
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, Handler};
use embedded_storage::nor_flash::{NorFlash, NorFlashErrorKind};

use crate::consts::{
    DfuAttributes, Request, State, Status, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_DFU,
    USB_CLASS_APPN_SPEC,
};
use crate::Reset;

/// Internal state for USB DFU
pub struct Control<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> {
    updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
    attrs: DfuAttributes,
    state: State,
    status: Status,
    offset: usize,
    _rst: PhantomData<RST>,
}

impl<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> Control<'d, DFU, STATE, RST, BLOCK_SIZE> {
    /// Create a new DFU instance to handle DFU transfers.
    pub fn new(updater: BlockingFirmwareUpdater<'d, DFU, STATE>, attrs: DfuAttributes) -> Self {
        Self {
            updater,
            attrs,
            state: State::DfuIdle,
            status: Status::Ok,
            offset: 0,
            _rst: PhantomData,
        }
    }

    fn reset_state(&mut self) {
        self.offset = 0;
        self.state = State::DfuIdle;
        self.status = Status::Ok;
    }
}

impl<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> Handler
    for Control<'d, DFU, STATE, RST, BLOCK_SIZE>
{
    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        data: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }
        match Request::try_from(req.request) {
            Ok(Request::Abort) => {
                self.reset_state();
                Some(OutResponse::Accepted)
            }
            Ok(Request::Dnload) if self.attrs.contains(DfuAttributes::CAN_DOWNLOAD) => {
                if req.value == 0 {
                    self.state = State::Download;
                    self.offset = 0;
                }

                let mut buf = AlignedBuffer([0; BLOCK_SIZE]);
                buf.as_mut()[..data.len()].copy_from_slice(data);

                if req.length == 0 {
                    match self.updater.mark_updated() {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::ManifestSync;
                        }
                        Err(e) => {
                            self.state = State::Error;
                            match e {
                                embassy_boot::FirmwareUpdaterError::Flash(e) => match e {
                                    NorFlashErrorKind::NotAligned => self.status = Status::ErrWrite,
                                    NorFlashErrorKind::OutOfBounds => self.status = Status::ErrAddress,
                                    _ => self.status = Status::ErrUnknown,
                                },
                                embassy_boot::FirmwareUpdaterError::Signature(_) => self.status = Status::ErrVerify,
                                embassy_boot::FirmwareUpdaterError::BadState => self.status = Status::ErrUnknown,
                            }
                        }
                    }
                } else {
                    if self.state != State::Download {
                        // Unexpected DNLOAD while chip is waiting for a GETSTATUS
                        self.status = Status::ErrUnknown;
                        self.state = State::Error;
                        return Some(OutResponse::Rejected);
                    }
                    match self.updater.write_firmware(self.offset, buf.as_ref()) {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::DlSync;
                            self.offset += data.len();
                        }
                        Err(e) => {
                            self.state = State::Error;
                            match e {
                                embassy_boot::FirmwareUpdaterError::Flash(e) => match e {
                                    NorFlashErrorKind::NotAligned => self.status = Status::ErrWrite,
                                    NorFlashErrorKind::OutOfBounds => self.status = Status::ErrAddress,
                                    _ => self.status = Status::ErrUnknown,
                                },
                                embassy_boot::FirmwareUpdaterError::Signature(_) => self.status = Status::ErrVerify,
                                embassy_boot::FirmwareUpdaterError::BadState => self.status = Status::ErrUnknown,
                            }
                        }
                    }
                }

                Some(OutResponse::Accepted)
            }
            Ok(Request::Detach) => Some(OutResponse::Accepted), // Device is already in DFU mode
            Ok(Request::ClrStatus) => {
                self.reset_state();
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
        match Request::try_from(req.request) {
            Ok(Request::GetStatus) => {
                //TODO: Configurable poll timeout, ability to add string for Vendor error
                buf[0..6].copy_from_slice(&[self.status as u8, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                match self.state {
                    State::DlSync => self.state = State::Download,
                    State::ManifestSync => RST::sys_reset(),
                    _ => {}
                }

                Some(InResponse::Accepted(&buf[0..6]))
            }
            Ok(Request::GetState) => {
                buf[0] = self.state as u8;
                Some(InResponse::Accepted(&buf[0..1]))
            }
            Ok(Request::Upload) if self.attrs.contains(DfuAttributes::CAN_UPLOAD) => {
                //TODO: FirmwareUpdater does not provide a way of reading the active partition, can't upload.
                Some(InResponse::Rejected)
            }
            _ => None,
        }
    }
}

/// An implementation of the USB DFU 1.1 protocol
///
/// This function will add a DFU interface descriptor to the provided Builder, and register the provided Control as a handler for the USB device
/// The handler is responsive to DFU GetState, GetStatus, Abort, and ClrStatus commands, as well as Download if configured by the user.
///
/// Once the host has initiated a DFU download operation, the chunks sent by the host will be written to the DFU partition.
/// Once the final sync in the manifestation phase has been received, the handler will trigger a system reset to swap the new firmware.
pub fn usb_dfu<'d, D: Driver<'d>, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize>(
    builder: &mut Builder<'d, D>,
    handler: &'d mut Control<'d, DFU, STATE, RST, BLOCK_SIZE>,
) {
    let mut func = builder.function(0x00, 0x00, 0x00);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_DFU, None);
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            handler.attrs.bits(),
            0xc4,
            0x09, // 2500ms timeout, doesn't affect operation as DETACH not necessary in bootloader code
            (BLOCK_SIZE & 0xff) as u8,
            ((BLOCK_SIZE & 0xff00) >> 8) as u8,
            0x10,
            0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(handler);
}
