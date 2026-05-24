use core::ptr;
use core::sync::atomic::{Ordering, compiler_fence};

use embedded_io::ErrorKind;

use crate::wb::PacketHeader;
use crate::wb::consts::TlPacketType;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Cmd {
    pub cmd_code: u16,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

impl Default for Cmd {
    fn default() -> Self {
        Self {
            cmd_code: 0,
            payload_len: 0,
            payload: [0u8; 255],
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdSerial {
    pub ty: u8,
    pub cmd: Cmd,
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdSerialStub {
    pub ty: u8,
    pub cmd_code: u16,
    pub payload_len: u8,
}

/// ```rust,ignore
/// struct CmdPacket {
///     header: [u8; 8],
///     ty: [u8; 1],
///     cmd_code: [u8; 2], // `EvtPacket` one byte code
///     payload_len: [u8; 1]
///     payload [u8; 255]
/// }
/// ```
#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdPacket {
    pub header: PacketHeader,
    pub cmdserial: CmdSerial,
}

impl CmdPacket {
    pub unsafe fn read_stub(ptr: *const CmdPacket) -> CmdSerialStub {
        unsafe { ptr::read_unaligned(&raw const (*ptr).cmdserial as *const CmdSerialStub) }
    }

    pub unsafe fn write_stub(ptr: *mut CmdPacket, stub: CmdSerialStub) {
        unsafe { ptr::write_unaligned(&raw mut (*ptr).cmdserial as *mut CmdSerialStub, stub) }
    }

    pub unsafe fn read_serial(ptr: *const CmdPacket) -> (*const u8, usize) {
        let payload_len = Self::read_stub(ptr).payload_len as usize;

        (
            &raw const (*ptr).cmdserial as *const u8,
            payload_len + size_of::<PacketHeader>(),
        )
    }

    pub unsafe fn write_serial(ptr: *mut CmdPacket) -> (*mut u8, usize) {
        (&raw mut (*ptr).cmdserial as *mut u8, size_of::<CmdSerial>())
    }

    pub unsafe fn read_payload(ptr: *const CmdPacket) -> (*const u8, usize) {
        let payload_len = Self::read_stub(ptr).payload_len as usize;

        (&raw const (*ptr).cmdserial.cmd.payload as *const u8, payload_len)
    }

    pub unsafe fn write_payload(ptr: *mut CmdPacket) -> (*mut u8, usize) {
        (&raw mut (*ptr).cmdserial.cmd.payload as *mut u8, size_of::<[u8; 255]>())
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataSerial {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
    pub acl_data: [u8; 1],
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataSerialStub {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}

impl AclDataPacket {
    #[allow(dead_code)]
    pub unsafe fn write_into(cmd_buf: *mut AclDataPacket, packet_type: TlPacketType, handle: u16, payload: &[u8]) {
        let p_cmd_serial = (cmd_buf as *mut u8).add(size_of::<PacketHeader>());
        let p_payload = p_cmd_serial.add(size_of::<AclDataSerialStub>());

        ptr::write_unaligned(
            p_cmd_serial as *mut _,
            AclDataSerialStub {
                ty: packet_type as u8,
                handle: handle,
                length: payload.len() as u16,
            },
        );

        ptr::copy_nonoverlapping(payload as *const _ as *const u8, p_payload, payload.len());

        compiler_fence(Ordering::Release);
    }
}

pub struct VolatileWriter {
    start: *mut u8,
    len: usize,
}

impl VolatileWriter {
    #[allow(dead_code)]
    pub unsafe fn from_serial(ptr: *mut CmdPacket) -> Self {
        unsafe {
            let (ptr, len) = CmdPacket::write_serial(ptr);

            Self { start: ptr, len }
        }
    }

    pub unsafe fn from_payload(ptr: *mut CmdPacket) -> Self {
        unsafe {
            let (ptr, len) = CmdPacket::write_payload(ptr);

            Self { start: ptr, len }
        }
    }
}

impl<'d> embedded_io::ErrorType for VolatileWriter {
    type Error = ErrorKind;
}

impl embedded_io::Write for VolatileWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self.len == 0 {
            return Err(ErrorKind::WriteZero);
        }

        unsafe {
            let count = self.len.min(buf.len());

            ptr::copy_nonoverlapping(buf as *const _ as *const u8, self.start, count);

            self.start = self.start.add(count);
            self.len -= count;

            Ok(count)
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        compiler_fence(Ordering::Release);

        Ok(())
    }
}
