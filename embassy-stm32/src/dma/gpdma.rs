#![macro_use]

use core::future::Future;
use core::pin::Pin;
use core::ptr;
use core::sync::atomic::{fence, AtomicBool, AtomicU8, Ordering};
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use super::word::{Word, WordSize};
use super::{AnyChannel, Channel, Dir, Request, STATE};
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::Priority;
use crate::pac;
use crate::pac::gpdma::vals;

pub(crate) struct ChannelInfo {
    pub(crate) dma: pac::gpdma::Gpdma,
    pub(crate) num: usize,
}

/// GPDMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {}
    }
}

impl From<WordSize> for vals::ChTr1Dw {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BYTE,
            WordSize::TwoBytes => Self::HALFWORD,
            WordSize::FourBytes => Self::WORD,
        }
    }
}

pub(crate) struct ChannelState {
    waker: AtomicWaker,
    is_lli_mode: AtomicBool,
    transfer_in: AtomicU8,
    transfer_out: AtomicU8,
    transfer_size: AtomicU8,
}

impl ChannelState {
    pub(crate) const NEW: Self = Self {
        waker: AtomicWaker::new(),
        is_lli_mode: AtomicBool::new(false),
        transfer_in: AtomicU8::new(0),
        transfer_out: AtomicU8::new(0),
        transfer_size: AtomicU8::new(0),
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(cs: critical_section::CriticalSection, irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, gpdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, irq_priority);
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_gpdma();
}

/// Linked List Table
#[derive(Debug)]
pub struct LliTable<W: Word, const MEMS: usize, const BUFLEN: usize> {
    addrs: [&'static [W; BUFLEN]; MEMS],
    items: &'static mut [LliItem; MEMS],
    option: LliOption,
}

impl<W: Word, const MEMS: usize, const BUFLEN: usize> LliTable<W, MEMS, BUFLEN> {
    /// sets the buffer addresses
    ///
    /// const ADC_BUFFERS: usize = 2;
    /// const ADC1_CHANNELS: usize = 8;
    /// static mut ADC1_BUF: [[u16;8];ADC_BUFFERS] = [[0u16;ADC1_CHANNELS];ADC_BUFFERS];
    /// static mut ADC1_LLIITEMS: [LliItem;ADC_BUFFERS] = [LliItem { dar: 0, llr: 0 }; ADC_BUFFERS];
    /// static ADC1_LLITABLE: StaticCell<LliTable<u16, ADC_BUFFERS, ADC1_CHANNELS>> = StaticCell::new();
    /// // creates an array of buffer addresses
    /// let adc1_lli = unsafe { ADC1_BUF.iter().map(|w| *addr_of!(w)).collect::<Vec<&[u16;ADC1_CHANNELS]>>().as_slice().try_into().unwrap() };  
    ///
    /// let lli_table = unsafe { ADC1_LLITABLE.init(LliTable::new(adc1_lli, &mut *addr_of_mut!(ADC1_LLIITEMS), LliOption::Repeated)) };
    ///
    /// // start the dma with
    /// let dma = Transfer::new_read_with_lli(...)
    ///
    /// // the data can read with get_next_buffer(..)
    ///
    pub fn new(
        addrs: [&'static [W; BUFLEN]; MEMS],
        lli_items: &'static mut [LliItem; MEMS],
        option: LliOption,
    ) -> Self {
        assert!(MEMS > 1);
        assert!(BUFLEN > 0);
        //map the buffer startaddr to lli
        for (index, buf) in addrs.iter().enumerate() {
            let (ptr, mut len) = super::slice_ptr_parts(*buf);
            len *= W::size().bytes();
            assert!(len > 0 && len <= 0xFFFF);
            lli_items[index].dar = ptr as u32;
        }
        let mut this = Self {
            items: lli_items,
            addrs,
            option,
        };
        this.fixing_memory();
        this
    }

    /// Create the Linked List
    fn fixing_memory(&mut self) {
        // create linked list
        for i in 0..MEMS - 1 {
            let lli_plus_one = ptr::addr_of!(self.items[i + 1]) as u16;
            self.items[i].set_llr(lli_plus_one as u16);
        }
        match self.option {
            LliOption::Repeated => self.items[MEMS - 1].set_llr(ptr::addr_of!(self.items[0]) as u16), // Connect the end and the beginning
            LliOption::Single => self.items[MEMS - 1].llr = 0,
        }
    }

    /// get the next Buffer
    pub fn get_next_buffer(&self, dma: &Transfer) -> Option<&&[W; BUFLEN]> {
        let state = dma.get_lli_state();
        let i = state.transfer_in.load(Ordering::Relaxed);
        let o = state.transfer_out.load(Ordering::Relaxed);
        match i == o {
            true => None,
            false => {
                //defmt::info!("DMA GET BUF {}", o);
                let result = self.addrs.get((o as usize) % MEMS);
                let _ = state
                    .transfer_out
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                        if v >= state.transfer_size.load(Ordering::Relaxed) {
                            Some(0)
                        } else {
                            Some(v + 1)
                        }
                    });
                result
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
/// Linked List Item
pub struct LliItem {
    /// Data Start Address
    pub dar: u32,
    /// Linked List
    pub llr: u32,
}
#[allow(unused)]
impl LliItem {
    fn set_llr(&mut self, la: u16) {
        // set la, uda and ull
        self.llr = (la as u32) | 1u32 << 27 | 1u32 << 16;
    }
}

#[derive(Debug)]
/// Definition for the end of the linked list
pub enum LliOption {
    /// The end of the list is linked to the beginning
    Repeated,
    /// the list is only processed once
    Single,
}

impl AnyChannel {
    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub(crate) unsafe fn on_irq(&self) {
        let info = self.info();
        let state = &STATE[self.id as usize];

        let ch = info.dma.ch(info.num);
        let sr = ch.sr().read();

        if sr.dtef() {
            panic!(
                "DMA: data transfer error on DMA@{:08x} channel {}",
                info.dma.as_ptr() as u32,
                info.num
            );
        }
        if sr.usef() {
            panic!(
                "DMA: user settings error on DMA@{:08x} channel {}",
                info.dma.as_ptr() as u32,
                info.num
            );
        }

        if sr.suspf() || sr.tcf() {
            if state.is_lli_mode.load(Ordering::Relaxed) {
                let _ = state
                    .transfer_in
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                        if v >= state.transfer_size.load(Ordering::Relaxed) {
                            Some(0)
                        } else {
                            Some(v + 1)
                        }
                    });
                ch.fcr().modify(|reg| reg.set_tcf(true));
                //defmt::info!("DMA SET BUF {}", state.transfer_in.load(Ordering::Relaxed));
            } else {
                // disable all xxIEs to prevent the irq from firing again.
                ch.cr().write(|_| {});

                // Wake the future. It'll look at tcf and see it's set.
                state.waker.wake();
            }
        }
    }
}

/// DMA transfer.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: PeripheralRef<'a, AnyChannel>,
}

impl<'a> Transfer<'a> {
    /// Create a new read DMA transfer (peripheral to memory). with linked list
    /// The transfer starts at buf0 and moves to the next buffer after a transfer, etc
    /// The LLI Table can be configured as a single or repeated transfer
    /// the buffer switching is done in Hardware
    pub unsafe fn new_read_with_lli<W: Word, const M: usize, const N: usize>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        peri_addr: *mut W,
        llit: &LliTable<W, M, N>,
        _options: TransferOptions,
    ) -> Self {
        into_ref!(channel);
        let channel: PeripheralRef<'a, AnyChannel> = channel.map_into();
        let data_size = W::size();
        let info = channel.info();
        let ch = info.dma.ch(info.num);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let this = Self { channel };

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&*this.channel, request);

        ch.cr().write(|w| w.set_reset(true));
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF); // clear all irqs
        ch.tr1().write(|w| {
            w.set_sdw(data_size.into());
            w.set_ddw(data_size.into());
            w.set_sinc(false);
            w.set_dinc(true);
        });
        ch.tr2().write(|w| {
            w.set_dreq(vals::ChTr2Dreq::SOURCEPERIPHERAL);
            w.set_reqsel(request);
        });

        ch.sar().write_value(peri_addr as _); // Peripheral Addr
        let llis_base_addr = ptr::addr_of!(llit.items) as u32;
        ch.lbar().write(|reg| reg.set_lba((llis_base_addr >> 16) as u16)); // linked high addr
        ch.br1().write(|reg| reg.set_bndt((N * W::size().bytes()) as u16));
        ch.dar().write(|reg| *reg = llit.items[0].dar);
        ch.llr().write(|reg| reg.0 = llit.items[0].llr); // Set Start llr

        let state = &STATE[this.channel.id as usize];
        state.is_lli_mode.store(true, Ordering::Relaxed);
        state.transfer_in.store(0, Ordering::Relaxed);
        state.transfer_out.store(0, Ordering::Relaxed);
        state.transfer_size.store(M as _, Ordering::Relaxed);

        ch.cr().write(|w| {
            // Enable interrupts
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);

            // Start it
            w.set_en(true);
        });

        this
    }

    /// get the channel state
    fn get_lli_state(&self) -> &ChannelState {
        &STATE[self.channel.id as usize]
    }

    /// Create a new read DMA transfer (peripheral to memory).
    pub unsafe fn new_read<W: Word>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Self {
        Self::new_read_raw(channel, request, peri_addr, buf, options)
    }

    /// Create a new read DMA transfer (peripheral to memory), using raw pointers.
    pub unsafe fn new_read_raw<W: Word>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let (ptr, len) = super::slice_ptr_parts_mut(buf);
        assert!(len > 0 && len <= 0xFFFF);

        Self::new_inner(
            channel.map_into(),
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            ptr as *mut u32,
            len,
            true,
            W::size(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to peripheral).
    pub unsafe fn new_write<W: Word>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        Self::new_write_raw(channel, request, buf, peri_addr, options)
    }

    /// Create a new write DMA transfer (memory to peripheral), using raw pointers.
    pub unsafe fn new_write_raw<W: Word>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        buf: *const [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let (ptr, len) = super::slice_ptr_parts(buf);
        assert!(len > 0 && len <= 0xFFFF);

        Self::new_inner(
            channel.map_into(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            ptr as *mut u32,
            len,
            true,
            W::size(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to peripheral), writing the same value repeatedly.
    pub unsafe fn new_write_repeated<W: Word>(
        channel: impl Peripheral<P = impl Channel> + 'a,
        request: Request,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        Self::new_inner(
            channel.map_into(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            repeated as *const W as *mut u32,
            count,
            false,
            W::size(),
            options,
        )
    }

    unsafe fn new_inner(
        channel: PeripheralRef<'a, AnyChannel>,
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        _options: TransferOptions,
    ) -> Self {
        let info = channel.info();
        let ch = info.dma.ch(info.num);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let this = Self { channel };

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&*this.channel, request);

        ch.cr().write(|w| w.set_reset(true));
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF); // clear all irqs
        ch.llr().write(|_| {}); // no linked list
        ch.tr1().write(|w| {
            w.set_sdw(data_size.into());
            w.set_ddw(data_size.into());
            w.set_sinc(dir == Dir::MemoryToPeripheral && incr_mem);
            w.set_dinc(dir == Dir::PeripheralToMemory && incr_mem);
        });
        ch.tr2().write(|w| {
            w.set_dreq(match dir {
                Dir::MemoryToPeripheral => vals::ChTr2Dreq::DESTINATIONPERIPHERAL,
                Dir::PeripheralToMemory => vals::ChTr2Dreq::SOURCEPERIPHERAL,
            });
            w.set_reqsel(request);
        });
        ch.br1().write(|w| {
            // BNDT is specified as bytes, not as number of transfers.
            w.set_bndt((mem_len * data_size.bytes()) as u16)
        });

        match dir {
            Dir::MemoryToPeripheral => {
                ch.sar().write_value(mem_addr as _);
                ch.dar().write_value(peri_addr as _);
            }
            Dir::PeripheralToMemory => {
                ch.sar().write_value(peri_addr as _);
                ch.dar().write_value(mem_addr as _);
            }
        }

        ch.cr().write(|w| {
            // Enable interrupts
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);

            // Start it
            w.set_en(true);
        });

        this
    }

    /// Request the transfer to stop.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        ch.cr().modify(|w| w.set_susp(true))
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        let sr = ch.sr().read();
        !sr.tcf() && !sr.suspf()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        ch.br1().read().bndt()
    }

    /// Blocking wait until the transfer finishes.
    pub fn blocking_wait(mut self) {
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);

        core::mem::forget(self);
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

impl<'a> Unpin for Transfer<'a> {}
impl<'a> Future for Transfer<'a> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = &STATE[self.channel.id as usize];
        state.waker.register(cx.waker());

        if self.is_running() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
