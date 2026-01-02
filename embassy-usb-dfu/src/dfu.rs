//! DFU bootloader part of DFU logic
use embassy_boot::{AlignedBuffer, BlockingFirmwareUpdater, FirmwareUpdaterError};
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, FunctionBuilder, Handler};
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
    buf: AlignedBuffer<BLOCK_SIZE>,
    reset: RST,

    #[cfg(feature = "_verify")]
    public_key: &'static [u8; 32],
}

impl<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> Control<'d, DFU, STATE, RST, BLOCK_SIZE> {
    /// Create a new DFU instance to handle DFU transfers.
    pub fn new(
        updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
        attrs: DfuAttributes,
        reset: RST,
        #[cfg(feature = "_verify")] public_key: &'static [u8; 32],
    ) -> Self {
        Self {
            updater,
            attrs,
            state: State::DfuIdle,
            status: Status::Ok,
            offset: 0,
            buf: AlignedBuffer([0; BLOCK_SIZE]),
            reset,

            #[cfg(feature = "_verify")]
            public_key,
        }
    }

    fn reset_state(&mut self) {
        self.offset = 0;
        self.state = State::DfuIdle;
        self.status = Status::Ok;
    }
}

impl From<FirmwareUpdaterError> for Status {
    fn from(e: FirmwareUpdaterError) -> Self {
        match e {
            FirmwareUpdaterError::Flash(e) => match e {
                NorFlashErrorKind::NotAligned => Status::ErrWrite,
                NorFlashErrorKind::OutOfBounds => Status::ErrAddress,
                _ => Status::ErrUnknown,
            },
            FirmwareUpdaterError::Signature(_) => Status::ErrVerify,
            FirmwareUpdaterError::BadState => Status::ErrUnknown,
        }
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
            debug!("Unknown out request: {:?}", req);
            return None;
        }
        match Request::try_from(req.request) {
            Ok(Request::Abort) => {
                info!("Abort requested");
                self.reset_state();
                Some(OutResponse::Accepted)
            }
            Ok(Request::Dnload) if self.attrs.contains(DfuAttributes::CAN_DOWNLOAD) => {
                if req.value == 0 {
                    info!("Download starting");
                    self.state = State::Download;
                    self.offset = 0;
                }

                if self.state != State::Download {
                    error!("Unexpected DNLOAD while chip is waiting for a GETSTATUS");
                    self.status = Status::ErrUnknown;
                    self.state = State::Error;
                    return Some(OutResponse::Rejected);
                }

                if data.len() > BLOCK_SIZE {
                    error!("USB data len exceeded block size");
                    self.status = Status::ErrUnknown;
                    self.state = State::Error;
                    return Some(OutResponse::Rejected);
                }

                debug!("Copying {} bytes to buffer", data.len());
                self.buf.as_mut()[..data.len()].copy_from_slice(data);

                let final_transfer = req.length == 0;
                if final_transfer {
                    debug!("Receiving final transfer");

                    #[cfg(feature = "_verify")]
                    let update_res: Result<(), FirmwareUpdaterError> = {
                        const SIGNATURE_LEN: usize = 64;

                        let mut signature = [0; SIGNATURE_LEN];
                        let update_len = (self.offset - SIGNATURE_LEN) as u32;

                        self.updater.read_dfu(update_len, &mut signature).and_then(|_| {
                            self.updater
                                .verify_and_mark_updated(self.public_key, &signature, update_len)
                        })
                    };

                    #[cfg(not(feature = "_verify"))]
                    let update_res = self.updater.mark_updated();

                    match update_res {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::ManifestSync;
                            info!("Update complete");
                        }
                        Err(e) => {
                            error!("Error completing update: {}", e);
                            self.state = State::Error;
                            self.status = e.into();
                        }
                    }
                } else {
                    debug!("Writing {} bytes at {}", data.len(), self.offset);
                    match self.updater.write_firmware(self.offset, self.buf.as_ref()) {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::DlSync;
                            self.offset += data.len();
                        }
                        Err(e) => {
                            error!("Error writing firmware: {:?}", e);
                            self.state = State::Error;
                            self.status = e.into();
                        }
                    }
                }

                Some(OutResponse::Accepted)
            }
            Ok(Request::Detach) => Some(OutResponse::Accepted), // Device is already in DFU mode
            Ok(Request::ClrStatus) => {
                info!("Clear status requested");
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
            debug!("Unknown in request: {:?}", req);
            return None;
        }
        match Request::try_from(req.request) {
            Ok(Request::GetStatus) => {
                match self.state {
                    State::DlSync => self.state = State::Download,
                    State::ManifestSync => self.state = State::ManifestWaitReset,
                    _ => {}
                }

                //TODO: Configurable poll timeout, ability to add string for Vendor error
                buf[0..6].copy_from_slice(&[self.status as u8, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                Some(InResponse::Accepted(&buf[0..6]))
            }
            Ok(Request::GetState) => {
                buf[0] = self.state as u8;
                Some(InResponse::Accepted(&buf[0..1]))
            }
            Ok(Request::Upload) if self.attrs.contains(DfuAttributes::CAN_UPLOAD) => {
                //TODO: FirmwareUpdater provides a way of reading the active partition so we could in theory add functionality to upload firmware.
                Some(InResponse::Rejected)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        if matches!(self.state, State::ManifestSync | State::ManifestWaitReset) {
            self.reset.sys_reset()
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
    func_modifier: impl Fn(&mut FunctionBuilder<'_, 'd, D>),
) {
    let mut func = builder.function(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_DFU);

    // Here we give users the opportunity to add their own function level MSOS headers for instance.
    // This is useful when DFU functionality is part of a composite USB device.
    func_modifier(&mut func);

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
