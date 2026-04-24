//! Variable-length Packet Buffer

use core::slice;

/// Defines a variable-length packet buffer
///
/// This is like a ringbuffer, but it doesn't wrap packets.
/// The length of the packet is a `u16`. `start` points to
/// where the first readable packet starts and `end` points
/// to after where the last readable packet ends.
///
/// The `PacketBuffer` is not irq-safe. It should be called in a thread-mode
/// context only.
#[allow(dead_code)]
struct PacketBuffer<const N: usize> {
    start: usize,
    wrap: usize,
    end: usize,
    buffer: [u8; N],
}

impl<const N: usize> PacketBuffer<N> {
    /// Create a new `PacketBuffer`
    pub const fn new() -> Self {
        Self {
            start: 0,
            wrap: 0,
            end: 0,
            buffer: [0u8; N],
        }
    }

    /// Write a packet with a given length. This method will
    /// return `None` if there is not enough space available.
    #[allow(dead_code)]
    pub fn try_write_packet<'a>(&'a mut self, len: u16) -> Option<&'a mut [u8]> {
        let header = len.to_le_bytes();
        let size: usize = len as usize + size_of::<u16>();
        let needs_wrap = self.end + size > N;

        if len == 0 {
            Some(unsafe { slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), 0) })
        } else if self.start > self.end && self.end + size >= self.start {
            None
        } else if self.start <= self.end && needs_wrap && size >= self.start {
            None
        } else if !needs_wrap {
            let buf = &mut self.buffer[self.end..][size_of::<u16>()..];
            let buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), len.into()) };

            self.buffer[self.end..][..size_of::<u16>()].copy_from_slice(&header);
            self.end += size;

            Some(buf)
        } else {
            let buf = &mut self.buffer[0..][size_of::<u16>()..];
            let buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), len.into()) };

            self.buffer[0..][..size_of::<u16>()].copy_from_slice(&header);
            self.wrap = self.end;
            self.end = size;

            Some(buf)
        }
    }

    /// Read a packet with a given length. This method will
    /// return `None` if there is nothing to read
    #[allow(dead_code)]
    pub fn try_read_packet<'a>(&'a mut self) -> Option<&'a [u8]> {
        if self.start == 0 && self.end == 0 {
            None
        } else {
            if self.start == self.wrap {
                self.start = 0;
            }

            let len = u16::from_le_bytes((&self.buffer[self.start..][..size_of::<u16>()]).try_into().unwrap());
            let size: usize = len as usize + size_of::<u16>();

            let buf = &self.buffer[0..][size_of::<u16>()..];
            let buf = unsafe { slice::from_raw_parts(buf.as_ptr(), len.into()) };

            self.start += size;
            if self.start == self.wrap {
                // Move start to buffer len to allow use of the full buffer space
                self.start = N;
            }
            if self.start == self.end {
                // Reset only in the empty condition in order to disambiguate buffer full
                self.start = 0;
                self.end = 0;
                self.wrap = 0;
            }

            Some(buf)
        }
    }
}
