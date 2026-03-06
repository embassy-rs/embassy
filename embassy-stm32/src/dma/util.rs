use super::word::Word;
use super::{Channel, Request, Transfer, TransferOptions};
use crate::Peri;
use crate::dma::{ChannelInstance, InterruptHandler};
use crate::interrupt::typelevel::Binding;

/// Convenience wrapper, contains a channel and a request number.
///
/// Commonly used in peripheral drivers that own DMA channels.
pub(crate) struct ChannelAndRequest<'d> {
    pub channel: Channel<'d>,
    pub request: Request,
}

impl<'d> ChannelAndRequest<'d> {
    pub fn new<T: ChannelInstance>(
        request: Request,
        ch: Peri<'d, T>,
        irqs: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        Self {
            channel: Channel::new(ch, irqs),
            request,
        }
    }

    /// Create a read DMA transfer (peripheral to memory).
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel.read(self.request, peri_addr, buf, options)
    }

    #[cfg(not(stm32c5))]
    /// Create a read DMA transfer (peripheral to memory), using raw pointers.
    pub unsafe fn read_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        peri_addr: *mut PW,
        buf: *mut [MW],
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel.read_raw(self.request, peri_addr, buf, options)
    }

    #[allow(dead_code)]
    /// Create a read DMA transfer (peripheral to memory), writing the same value repeatedly.
    pub unsafe fn read_raw_repeated<'a, MW: Word, PW: Word>(
        &'a mut self,
        repeated: *mut MW,
        count: usize,
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel
            .read_raw_repeated(self.request, repeated, count, peri_addr, options)
    }

    /// Create a write DMA transfer (memory to peripheral).
    pub unsafe fn write<'a, W: Word>(
        &'a mut self,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel.write(self.request, buf, peri_addr, options)
    }

    #[cfg(not(stm32c5))]
    /// Create a write DMA transfer (memory to peripheral), using raw pointers.
    pub unsafe fn write_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        buf: *const [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel.write_raw(self.request, buf, peri_addr, options)
    }

    #[allow(dead_code)]
    /// Create a write DMA transfer (memory to peripheral), writing the same value repeatedly.
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.channel
            .write_repeated(self.request, repeated, count, peri_addr, options)
    }

    #[allow(dead_code)]
    /// Reborrow the channel and request, allowing it to be used in multiple places.
    pub fn reborrow(&mut self) -> ChannelAndRequest<'_> {
        ChannelAndRequest {
            channel: self.channel.reborrow(),
            request: self.request,
        }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn clone_unchecked(&self) -> ChannelAndRequest<'d> {
        ChannelAndRequest {
            channel: self.channel.clone_unchecked(),
            request: self.request,
        }
    }
}
