use super::word::Word;
use super::{AnyChannel, Request, Transfer, TransferOptions};
use crate::Peri;

/// Convenience wrapper, contains a channel and a request number.
///
/// Commonly used in peripheral drivers that own DMA channels.
pub(crate) struct ChannelAndRequest<'d> {
    pub channel: Peri<'d, AnyChannel>,
    pub request: Request,
}

impl<'d> ChannelAndRequest<'d> {
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read(self.channel.reborrow(), self.request, peri_addr, buf, options)
    }

    pub unsafe fn read_raw<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read_raw(self.channel.reborrow(), self.request, peri_addr, buf, options)
    }

    pub unsafe fn write<'a, W: Word>(
        &'a mut self,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write(self.channel.reborrow(), self.request, buf, peri_addr, options)
    }

    pub unsafe fn write_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        buf: *const [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_raw(self.channel.reborrow(), self.request, buf, peri_addr, options)
    }

    #[allow(dead_code)]
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_repeated(
            self.channel.reborrow(),
            self.request,
            repeated,
            count,
            peri_addr,
            options,
        )
    }
}
