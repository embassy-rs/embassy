use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::{ptr, slice};

use crate::wb::PacketHeader;

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

#[allow(dead_code)]
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
    pub payload_len: u8,
}

/// This format shall be used for all events (asynchronous and command response) reported
/// by the CPU2 except for the command response of a system command where the header is not there
/// and the format to be used shall be `EvtSerial`.
///
/// ### Note:
/// Be careful that the asynchronous events reported by the CPU2 on the system channel do
/// include the header and shall use `EvtPacket` format. Only the command response format on the
/// system channel is different.
///
/// ```rust,ignore
/// struct EvtPacket {
///     header: [u8; 8],
///     kind: [u8; 1],
///     evt_code: [u8; 1], // `CmdPacket` two byte code
///     payload_len: [u8; 1]
///     payload [u8; 255]
/// }
/// ```
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EvtPacket {
    pub header: PacketHeader,
    pub evt_serial: EvtSerial,
}

impl EvtPacket {
    pub unsafe fn read_stub(ptr: *const EvtPacket) -> EvtStub {
        unsafe { ptr::read_volatile(&raw const (*ptr).evt_serial as *const EvtStub) }
    }

    pub unsafe fn read_serial(ptr: *const EvtPacket) -> (*const u8, usize) {
        let payload_len = Self::read_stub(ptr).payload_len as usize;

        (
            &raw const (*ptr).evt_serial as *const u8,
            payload_len + size_of::<PacketHeader>(),
        )
    }

    pub unsafe fn read_payload(ptr: *const EvtPacket) -> (*const u8, usize) {
        let payload_len = Self::read_stub(ptr).payload_len as usize;

        (&raw const (*ptr).evt_serial.evt.payload as *const u8, payload_len)
    }
}

pub trait MemoryManager {
    unsafe fn new_event_packet(_evt: *mut EvtPacket) {}
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
        unsafe { T::new_event_packet(ptr) };

        Self { ptr, mm: PhantomData }
    }

    /// Returns information about the event
    pub fn stub(&self) -> EvtStub {
        unsafe { EvtPacket::read_stub(self.ptr) }
    }

    pub fn serial<'a>(&'a self) -> &'a [u8] {
        unsafe {
            let (serial, len) = EvtPacket::read_serial(self.ptr);

            compiler_fence(Ordering::Acquire);

            slice::from_raw_parts(serial, len)
        }
    }

    pub unsafe fn serial_unchecked(&self) -> &'static [u8] {
        unsafe {
            let (serial, len) = EvtPacket::read_serial(self.ptr);

            compiler_fence(Ordering::Acquire);

            slice::from_raw_parts(serial, len)
        }
    }

    pub fn payload<'a>(&'a self) -> &'a [u8] {
        unsafe {
            let (payload, len) = EvtPacket::read_payload(self.ptr);

            compiler_fence(Ordering::Acquire);

            slice::from_raw_parts(payload, len)
        }
    }

    pub unsafe fn payload_unchecked(&self) -> &'static [u8] {
        unsafe {
            let (payload, len) = EvtPacket::read_payload(self.ptr);

            compiler_fence(Ordering::Acquire);

            slice::from_raw_parts(payload, len)
        }
    }
}

impl<T: MemoryManager> Drop for EvtBox<T> {
    fn drop(&mut self) {
        unsafe { T::drop_event_packet(self.ptr) };
    }
}
