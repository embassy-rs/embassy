//! USB DFU dfu mode.

use embassy_usb_driver::Driver;

use crate::{
    class::dfu::consts::STATUS_OK,
    control::{InResponse, OutResponse, Recipient, Request as ControlRequest, RequestType},
    Builder, FunctionBuilder,
};

use super::consts::{
    DfuAttributes, Error, Request, State, APPN_SPEC_SUBCLASS_DFU, DESC_DFU_FUNCTIONAL, DFU_PROTOCOL_DFU,
    USB_CLASS_APPN_SPEC,
};

/// TODO
pub trait Handler {
    /// TODO
    fn start(&mut self);
    /// TODO
    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    /// TODO
    fn finish(&mut self) -> Result<(), Error>;
    /// TODO
    fn switch_to_app(&mut self);
}

/// Internal state for USB DFU
pub struct DfuState<H: Handler> {
    handler: H,
    attrs: DfuAttributes,
    state: State,
    error: Option<Error>,
    next_block_num: usize,
}

impl<'d, H: Handler> DfuState<H> {
    /// Create a new DFU instance to handle DFU transfers.
    pub fn new(handler: H, attrs: DfuAttributes) -> Self {
        Self {
            handler,
            attrs,
            state: State::DfuIdle,
            error: None,
            next_block_num: 0,
        }
    }

    /// TODO
    #[inline]
    pub fn set_error(&mut self, error: Error) {
        self.state = State::Error;
        self.error = Some(error);
    }

    #[inline]
    fn set_state(&mut self, state: State) {
        assert_ne!(state, State::Error);
        self.state = state;
        self.error = None;
    }

    #[inline]
    fn reset_state(&mut self) {
        self.next_block_num = 0;
        self.set_state(State::DfuIdle);
    }
}

impl<H: Handler> crate::Handler for DfuState<H> {
    fn reset(&mut self) {
        self.handler.switch_to_app();
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
                    self.set_error(Error::Unknown);
                    return Some(OutResponse::Rejected);
                }

                if req.value == 0 {
                    self.handler.start();
                    self.set_state(State::Download);
                }

                if req.length == 0 {
                    match self.handler.finish() {
                        Ok(_) => self.set_state(State::ManifestSync),
                        Err(e) => self.set_error(e),
                    }
                } else {
                    if self.state != State::Download {
                        // Unexpected DNLOAD while chip is waiting for a GETSTATUS
                        self.set_error(Error::Unknown);
                        return Some(OutResponse::Rejected);
                    }
                    match self.handler.write(data) {
                        Ok(_) => {
                            self.set_state(State::DlSync);
                            self.next_block_num += 1;
                        }
                        Err(e) => self.set_error(e),
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
                let status = self.error.map(|e| e as u8).unwrap_or(STATUS_OK);
                //TODO: Configurable poll timeout, ability to add string for Vendor error
                buf[0..6].copy_from_slice(&[status, 0x32, 0x00, 0x00, self.state as u8, 0x00]);
                match self.state {
                    State::DlSync => self.state = State::Download,
                    State::ManifestSync => self.state = State::ManifestWaitReset,
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
    func_modifier: impl Fn(&mut FunctionBuilder<'_, 'd, D>),
) {
    let mut func = builder.function(0x00, 0x00, 0x00);

    // Here we give users the opportunity to add their own function level MSOS headers for instance.
    // This is useful when DFU functionality is part of a composite USB device.
    func_modifier(&mut func);

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
