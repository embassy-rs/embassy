//! DMA transfer management

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::dma::channel::Channel;

/// DMA transfer options
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Transfer data width
    pub width: Width,

    /// Transfer priority level
    pub priority: Priority,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            width: Width::Bit8,
            priority: Priority::Priority0,
        }
    }
}

/// DMA transfer priority
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Priority 7 (lowest priority)
    Priority7,
    /// Priority 6
    Priority6,
    /// Priority 5
    Priority5,
    /// Priority 4
    Priority4,
    /// Priority 3
    Priority3,
    /// Priority 2
    Priority2,
    /// Priority 1
    Priority1,
    /// Priority 0 (highest priority)
    Priority0,
}

/// DMA transfer width
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Width {
    /// 8 bits
    Bit8,
    /// 16 bits
    Bit16,
    /// 32 bits
    Bit32,
}

impl From<Width> for u8 {
    fn from(w: Width) -> Self {
        match w {
            Width::Bit8 => 0,
            Width::Bit16 => 1,
            Width::Bit32 => 2,
        }
    }
}

impl Width {
    /// Width in bytes
    pub fn byte_width(self) -> usize {
        match self {
            Width::Bit8 => 1,
            Width::Bit16 => 2,
            Width::Bit32 => 4,
        }
    }
}

/// DMA transfer direction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Memory-to-memory
    MemoryToMemory,
    /// Memory-to-peripheral
    MemoryToPeripheral,
    /// Peripheral-to-memory
    PeripheralToMemory,
}

/// DMA transfer
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Transfer<'d> {
    _inner: &'d Channel<'d>,
}

impl<'d> Transfer<'d> {
    /// Reads from a peripheral register into a memory buffer using DMA
    pub fn new_read(
        channel: &'d Channel<'d>,
        peri_addr: *const u8,
        buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Self {
        Self::new_inner_transfer(
            channel,
            Direction::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut [u8] as *mut u32,
            buf.len(),
            options,
        )
    }

    /// Writes a memory buffer into a peripheral register using DMA
    pub fn new_write(channel: &'d Channel<'d>, buf: &'d [u8], peri_addr: *mut u8, options: TransferOptions) -> Self {
        Self::new_inner_transfer(
            channel,
            Direction::MemoryToPeripheral,
            buf as *const [u8] as *const u32,
            peri_addr as *mut u32,
            buf.len(),
            options,
        )
    }

    /// Writes a memory buffer into another memory buffer using DMA
    pub fn new_write_mem(
        channel: &'d Channel<'d>,
        src_buf: &'d [u8],
        dst_buf: &'d mut [u8],
        options: TransferOptions,
    ) -> Self {
        Self::new_inner_transfer(
            channel,
            Direction::MemoryToMemory,
            src_buf as *const [u8] as *const u32,
            dst_buf as *mut [u8] as *mut u32,
            src_buf.len(),
            options,
        )
    }

    /// Configures the channel and initiates the DMA transfer
    fn new_inner_transfer(
        channel: &'d Channel<'d>,
        dir: Direction,
        src_buf: *const u32,
        dst_buf: *mut u32,
        mem_len: usize,
        options: TransferOptions,
    ) -> Self {
        // Configure the DMA channel descriptor and registers
        channel.configure_channel(dir, src_buf, dst_buf, mem_len, options);

        // Enable the channel
        channel.enable_channel();

        // Generate a software channel trigger to start the transfer
        channel.trigger_channel();

        Self { _inner: channel }
    }
}

impl Unpin for Transfer<'_> {}
impl Future for Transfer<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let channel = self._inner.info.ch_num;

        // Re-register the waker on each call to poll() because any calls to
        // wake will deregister the waker.
        super::DMA_WAKERS[channel].register(cx.waker());

        if self._inner.info.regs.active0().read().act().bits() & (1 << channel) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Transfer<'_> {
    fn drop(&mut self) {
        self._inner.abort()
    }
}
