use embassy_hal_internal::PeripheralRef;

use super::word::Word;
use super::{AnyChannel, Request, Transfer, TransferOptions};

/// Convenience wrapper, contains a DMA channel and a DMA request number.
///
/// Commonly used in peripheral drivers that own DMA channels.
pub struct ChannelAndRequest<'d> {
    /// DMA channel.
    pub channel: PeripheralRef<'d, AnyChannel>,
    /// DMA request.
    pub request: Request,
}

impl<'d> ChannelAndRequest<'d> {
    /// See [`Transfer::new_read()`].
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read(&mut self.channel, self.request, peri_addr, buf, options)
    }

    /// See [`Transfer::new_read_raw()`].
    pub unsafe fn read_raw<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read_raw(&mut self.channel, self.request, peri_addr, buf, options)
    }

    /// See [`Transfer::new_write()`].
    pub unsafe fn write<'a, W: Word>(
        &'a mut self,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write(&mut self.channel, self.request, buf, peri_addr, options)
    }

    /// See [`Transfer::new_write_raw()`].
    pub unsafe fn write_raw<'a, W: Word>(
        &'a mut self,
        buf: *const [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_raw(&mut self.channel, self.request, buf, peri_addr, options)
    }

    /// See [`Transfer::new_write_repeated()`].
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_repeated(&mut self.channel, self.request, repeated, count, peri_addr, options)
    }
}
