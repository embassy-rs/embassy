//! Class-specific control requests (MSC BBB r1.0 §3).

use core::cell::Cell;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::super::{BOT_REQ_GET_MAX_LUN, BOT_REQ_RESET};
use crate::Handler;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::types::InterfaceNumber;

/// Internal state for the MSC class.
pub struct State<'a> {
    pub(super) control: MaybeUninit<Control<'a>>,
    pub(super) reset_requested: CriticalSectionMutex<Cell<bool>>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> State<'a> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            reset_requested: CriticalSectionMutex::new(Cell::new(false)),
        }
    }
}

pub(super) struct Control<'a> {
    pub(super) interface: InterfaceNumber,
    pub(super) reset_requested: &'a CriticalSectionMutex<Cell<bool>>,
}

impl<'a> Handler for Control<'a> {
    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.interface.0 as u16)
        {
            return None;
        }

        match req.request {
            BOT_REQ_RESET if req.length == 0 && req.value == 0 && data.is_empty() => {
                self.reset_requested.lock(|flag| flag.set(true));
                Some(OutResponse::Accepted)
            }
            BOT_REQ_RESET => Some(OutResponse::Rejected),
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'b>(&'b mut self, req: Request, buf: &'b mut [u8]) -> Option<InResponse<'b>> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.interface.0 as u16)
        {
            return None;
        }

        match req.request {
            BOT_REQ_GET_MAX_LUN if req.length == 1 && req.value == 0 && !buf.is_empty() => {
                buf[0] = 0;
                Some(InResponse::Accepted(&buf[..1]))
            }
            BOT_REQ_GET_MAX_LUN => Some(InResponse::Rejected),
            _ => Some(InResponse::Rejected),
        }
    }
}
