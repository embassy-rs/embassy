use embassy_usb_driver::Driver;

use crate::{
    control::{InResponse, OutResponse, Recipient, Request as ControlRequest, RequestType},
    Builder,
};

use super::consts::{
    DfuAttributes, Request, State, Status, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_DFU,
    DFU_PROTOCOL_RT, USB_CLASS_APPN_SPEC,
};

pub trait Handler {
    fn start(&mut self);
    fn write(&mut self, data: &[u8]) -> Result<(), Status>;
    fn finish(&mut self) -> Result<(), Status>;
    fn reset(&mut self);
}

/// Internal state for USB DFU
pub struct DfuState<H: Handler> {
    handler: H,
    attrs: DfuAttributes,
    state: State,
    status: Status,
    next_block_num: usize,
}

impl<'d, H: Handler> DfuState<H> {
    /// Create a new DFU instance to handle DFU transfers.
    pub fn new(handler: H, attrs: DfuAttributes) -> Self {
        Self {
            handler,
            attrs,
            state: State::DfuIdle,
            status: Status::Ok,
            next_block_num: 0,
        }
    }

    fn reset_state(&mut self) {
        self.next_block_num = 0;
        self.state = State::DfuIdle;
        self.status = Status::Ok;
    }
}

impl<H: Handler> crate::Handler for DfuState<H> {
    fn reset(&mut self) {
        self.handler.reset();
    }

    fn control_out(&mut self, req: ControlRequest, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }
        match Request::try_from(req.request) {
            Ok(Request::Abort) => {
                self.reset_state();
                Some(OutResponse::Accepted)
            }
            Ok(Request::Dnload) if self.attrs.contains(DfuAttributes::CAN_DOWNLOAD) => {
                if req.value as usize != self.next_block_num {
                    error!("expected next block num {}, got {}", self.next_block_num, req.value);
                    self.state = State::Error;
                    self.status = Status::ErrUnknown;
                    return Some(OutResponse::Rejected);
                }

                if req.value == 0 {
                    self.state = State::Download;
                    self.handler.start();
                }

                if req.length == 0 {
                    match self.handler.finish() {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::ManifestSync;
                        }
                        Err(e) => {
                            self.state = State::Error;
                            self.status = e;
                        }
                    }
                } else {
                    if self.state != State::Download {
                        // Unexpected DNLOAD while chip is waiting for a GETSTATUS
                        self.status = Status::ErrUnknown;
                        self.state = State::Error;
                        return Some(OutResponse::Rejected);
                    }
                    match self.handler.write(data) {
                        Ok(_) => {
                            self.status = Status::Ok;
                            self.state = State::DlSync;
                            self.next_block_num += 1;
                        }
                        Err(e) => {
                            self.state = State::Error;
                            self.status = e;
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

    fn control_in<'a>(&'a mut self, req: ControlRequest, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface) {
            return None;
        }
        match Request::try_from(req.request) {
            Ok(Request::GetStatus) => {
                //TODO: Configurable poll timeout, ability to add string for Vendor error
                buf[0..6].copy_from_slice(&[self.status as u8, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                match self.state {
                    State::DlSync => self.state = State::Download,
                    State::ManifestSync => self.state = State::DfuIdle,
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
pub fn usb_dfu<'d, D: Driver<'d>, H: Handler>(
    builder: &mut Builder<'d, D>,
    state: &'d mut DfuState<H>,
    max_write_size: usize,
) {
    let mut func = builder.function(0x00, 0x00, 0x00);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(USB_CLASS_APPN_SPEC, APPN_SPEC_SUBCLASS_DFU, DFU_PROTOCOL_DFU, None);
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            state.attrs.bits(),
            0xc4,
            0x09, // 2500ms timeout, doesn't affect operation as DETACH not necessary in bootloader code
            (max_write_size & 0xff) as u8,
            ((max_write_size & 0xff00) >> 8) as u8,
            0x10,
            0x01, // DFU 1.1
        ],
    );

    drop(func);
    builder.handler(state);
}
