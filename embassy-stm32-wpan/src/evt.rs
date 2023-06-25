use core::marker::PhantomData;
use core::{ptr, slice};

use super::PacketHeader;
use crate::consts::TL_EVT_HEADER_SIZE;

/**
 * The payload of `Evt` for a command status event
 */
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CsEvt {
    pub status: u8,
    pub num_cmd: u8,
    pub cmd_code: u16,
}

/**
 * The payload of `Evt` for a command complete event
 */
#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CcEvt {
    pub num_cmd: u8,
    pub cmd_code: u16,
    pub payload: [u8; 1],
}

impl CcEvt {
    pub fn write(&self, buf: &mut [u8]) {
        unsafe {
            let len = core::mem::size_of::<CcEvt>();
            assert!(buf.len() >= len);

            let self_ptr: *const CcEvt = self;
            let self_buf_ptr: *const u8 = self_ptr.cast();

            core::ptr::copy(self_buf_ptr, buf.as_mut_ptr(), len);
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct AsynchEvt {
    sub_evt_code: u16,
    payload: [u8; 1],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Evt {
    pub evt_code: u8,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EvtSerial {
    pub kind: u8,
    pub evt: Evt,
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct EvtStub {
    pub kind: u8,
    pub evt_code: u8,
}

/// This format shall be used for all events (asynchronous and command response) reported
/// by the CPU2 except for the command response of a system command where the header is not there
/// and the format to be used shall be `EvtSerial`.
///
/// ### Note:
/// Be careful that the asynchronous events reported by the CPU2 on the system channel do
/// include the header and shall use `EvtPacket` format. Only the command response format on the
/// system channel is different.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EvtPacket {
    pub header: PacketHeader,
    pub evt_serial: EvtSerial,
}

impl EvtPacket {
    pub fn kind(&self) -> u8 {
        self.evt_serial.kind
    }

    pub fn evt(&self) -> &Evt {
        &self.evt_serial.evt
    }
}

pub trait MemoryManager {
    unsafe fn drop_event_packet(evt: *mut EvtPacket);
}

/// smart pointer to the [`EvtPacket`] that will dispose of [`EvtPacket`] buffer automatically
/// on [`Drop`]
#[derive(Debug)]
pub struct EvtBox<T: MemoryManager> {
    ptr: *mut EvtPacket,
    mm: PhantomData<T>,
}

unsafe impl<T: MemoryManager> Send for EvtBox<T> {}
impl<T: MemoryManager> EvtBox<T> {
    pub(super) fn new(ptr: *mut EvtPacket) -> Self {
        Self { ptr, mm: PhantomData }
    }

    /// Returns information about the event
    pub fn stub(&self) -> EvtStub {
        unsafe {
            let p_evt_stub = &(*self.ptr).evt_serial as *const _ as *const EvtStub;

            ptr::read_volatile(p_evt_stub)
        }
    }

    pub fn payload<'a>(&'a self) -> &'a [u8] {
        unsafe {
            let p_payload_len = &(*self.ptr).evt_serial.evt.payload_len as *const u8;
            let p_payload = &(*self.ptr).evt_serial.evt.payload as *const u8;

            let payload_len = ptr::read_volatile(p_payload_len);

            slice::from_raw_parts(p_payload, payload_len as usize)
        }
    }

    pub fn serial<'a>(&'a self) -> &'a [u8] {
        unsafe {
            let evt_serial: *const EvtSerial = &(*self.ptr).evt_serial;
            let evt_serial_buf: *const u8 = evt_serial.cast();

            let len = (*evt_serial).evt.payload_len as usize + TL_EVT_HEADER_SIZE;

            slice::from_raw_parts(evt_serial_buf, len)
        }
    }
}

impl<T: MemoryManager> Drop for EvtBox<T> {
    fn drop(&mut self) {
        unsafe { T::drop_event_packet(self.ptr) };
    }
}
