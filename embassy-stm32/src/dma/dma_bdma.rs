use core::future::{Future, poll_fn};
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering, compiler_fence, fence};
use core::task::{Context, Poll, Waker};

use embassy_sync::waitqueue::AtomicWaker;

use super::ringbuffer::{DmaCtrl, Error, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use super::word::{Word, WordSize};
use super::{Channel, Dir, Increment, Request, STATE, info};
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::WakeGuard;
use crate::{interrupt, pac};

pub(crate) unsafe fn on_irq(id: u8) {
    let info = info(id);
    let state = &STATE[id as usize];
    match info.dma {
        #[cfg(dma)]
        DmaInfo::Dma(r) => {
            let cr = r.st(info.num).cr();
            let isr = r.isr(info.num / 4).read();

            if isr.teif(info.num % 4) {
                panic!("DMA: error on DMA@{:08x} channel {}", r.as_ptr() as u32, info.num);
            }

            if isr.htif(info.num % 4) && cr.read().htie() {
                // Acknowledge half transfer complete interrupt
                r.ifcr(info.num / 4).write(|w| w.set_htif(info.num % 4, true));
            } else if isr.tcif(info.num % 4) && cr.read().tcie() {
                // Acknowledge  transfer complete interrupt
                r.ifcr(info.num / 4).write(|w| w.set_tcif(info.num % 4, true));
                state.complete_count.fetch_add(1, Ordering::Release);
            } else {
                return;
            }
            state.waker.wake();
        }
        #[cfg(bdma)]
        DmaInfo::Bdma(r) => {
            let isr = r.isr().read();
            let cr = r.ch(info.num).cr();

            if isr.teif(info.num) {
                panic!("DMA: error on BDMA@{:08x} channel {}", r.as_ptr() as u32, info.num);
            }

            if isr.htif(info.num) && cr.read().htie() {
                // Acknowledge half transfer complete interrupt
                r.ifcr().write(|w| w.set_htif(info.num, true));
            } else if isr.tcif(info.num) && cr.read().tcie() {
                // Acknowledge transfer complete interrupt
                r.ifcr().write(|w| w.set_tcif(info.num, true));
                #[cfg(not(armv6m))]
                state.complete_count.fetch_add(1, Ordering::Release);
                #[cfg(armv6m)]
                critical_section::with(|_| {
                    let x = state.complete_count.load(Ordering::Relaxed);
                    state.complete_count.store(x + 1, Ordering::Release);
                })
            } else {
                return;
            }

            state.waker.wake();
        }
        #[cfg(mdma)]
        DmaInfo::Mdma(r) => {
            // If our bit in gisr0 is not set, then the interrupt is not for this channel
            if !r.gisr0().read().gif(info.num) {
                return;
            }

            let isr = r.ch(info.num).isr();
            let ifcr = r.ch(info.num).ifcr();

            if isr.read().teif() {
                panic!("DMA: error on MDMA@{:08x} channel {}", r.as_ptr() as u32, info.num);
            }

            if isr.read().ctcif() {
                // Channel Transfer complete
                state.complete_count.fetch_add(1, Ordering::Release);
                ifcr.write(|w| w.set_cctcif(true));
            }

            state.waker.wake();
        }
    }
}

pub(crate) struct ChannelInfo {
    pub(crate) dma: DmaInfo,
    pub(crate) num: usize,
    #[cfg(feature = "_dual-core")]
    pub(crate) irq: pac::Interrupt,
    #[cfg(dmamux)]
    pub(crate) dmamux: Option<super::DmamuxInfo>,
    #[cfg(feature = "low-power")]
    pub(crate) stop_mode: crate::rcc::StopMode,
}

impl ChannelInfo {
    fn wake_guard(&self) -> WakeGuard {
        WakeGuard::new(
            #[cfg(feature = "low-power")]
            self.stop_mode,
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DmaInfo {
    #[cfg(dma)]
    Dma(pac::dma::Dma),
    #[cfg(bdma)]
    Bdma(pac::bdma::Dma),
    #[cfg(mdma)]
    Mdma(pac::mdma::Mdma),
}

/// DMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Peripheral burst transfer configuration
    #[cfg(any(dma, mdma))]
    pub pburst: Burst,
    /// Memory burst transfer configuration
    #[cfg(any(dma, mdma))]
    pub mburst: Burst,
    /// Flow control configuration
    #[cfg(dma)]
    pub flow_ctrl: FlowControl,
    /// FIFO threshold for DMA FIFO mode. If none, direct mode is used.
    #[cfg(dma)]
    pub fifo_threshold: Option<FifoThreshold>,
    /// Request priority level
    pub priority: Priority,
    /// Enable circular DMA
    ///
    /// Note:
    /// If you enable circular mode manually, you may want to build and `.await` the `Transfer` in a separate task.
    /// Since DMA in circular mode need manually stop, `.await` in current task would block the task forever.
    pub circular: bool,
    /// Enable half transfer interrupt
    pub half_transfer_ir: bool,
    /// Enable transfer complete interrupt
    pub complete_transfer_ir: bool,
    #[cfg(mdma)]
    /// Max bytes to transfer at once, 1-64
    pub buffer_size: u8,
    #[cfg(mdma)]
    /// Swap bytes in each half-word
    pub byte_swap: bool,
    #[cfg(mdma)]
    /// Swap half-words in each word
    pub half_word_swap: bool,
    #[cfg(mdma)]
    /// Swap words in each double-word
    pub word_swap: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            #[cfg(dma)]
            pburst: Burst::Single,
            #[cfg(dma)]
            mburst: Burst::Single,
            #[cfg(dma)]
            flow_ctrl: FlowControl::Dma,
            #[cfg(dma)]
            fifo_threshold: None,
            priority: Priority::VeryHigh,
            circular: false,
            half_transfer_ir: false,
            complete_transfer_ir: true,
            #[cfg(mdma)]
            buffer_size: MDMA_MAX_BUFFER,
            #[cfg(mdma)]
            byte_swap: false,
            #[cfg(mdma)]
            half_word_swap: false,
            #[cfg(mdma)]
            word_swap: false,
        }
    }
}

/// DMA request priority
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Low Priority
    Low,
    /// Medium Priority
    Medium,
    /// High Priority
    High,
    /// Very High Priority
    VeryHigh,
}

#[cfg(dma)]
impl From<Priority> for pac::dma::vals::Pl {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => pac::dma::vals::Pl::LOW,
            Priority::Medium => pac::dma::vals::Pl::MEDIUM,
            Priority::High => pac::dma::vals::Pl::HIGH,
            Priority::VeryHigh => pac::dma::vals::Pl::VERY_HIGH,
        }
    }
}

#[cfg(bdma)]
impl From<Priority> for pac::bdma::vals::Pl {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => pac::bdma::vals::Pl::LOW,
            Priority::Medium => pac::bdma::vals::Pl::MEDIUM,
            Priority::High => pac::bdma::vals::Pl::HIGH,
            Priority::VeryHigh => pac::bdma::vals::Pl::VERY_HIGH,
        }
    }
}

#[cfg(dma)]
pub use dma_only::*;
#[cfg(dma)]
mod dma_only {
    use pac::dma::vals;

    use super::*;

    impl From<WordSize> for vals::Size {
        fn from(raw: WordSize) -> Self {
            match raw {
                WordSize::OneByte => Self::BITS8,
                WordSize::TwoBytes => Self::BITS16,
                WordSize::FourBytes => Self::BITS32,
                WordSize::EightBytes => unimplemented!(),
            }
        }
    }

    impl From<Dir> for vals::Dir {
        fn from(raw: Dir) -> Self {
            match raw {
                Dir::MemoryToPeripheral => Self::MEMORY_TO_PERIPHERAL,
                Dir::PeripheralToMemory => Self::PERIPHERAL_TO_MEMORY,
                Dir::MemoryToMemory => Self::MEMORY_TO_MEMORY,
            }
        }
    }

    /// DMA transfer burst setting.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Burst {
        /// Single transfer
        Single,
        /// Incremental burst of 4 beats
        Incr4,
        /// Incremental burst of 8 beats
        Incr8,
        /// Incremental burst of 16 beats
        Incr16,
        /// Incremental burst of 32 beats
        Incr32,
        /// Incremental burst of 64 beats
        Incr64,
        /// Incremental burst of 128 beats
        Incr128,
        /// Incremental burst of 256 beats
        Incr256,
    }

    impl From<Burst> for vals::Burst {
        fn from(burst: Burst) -> Self {
            match burst {
                Burst::Single => vals::Burst::SINGLE,
                Burst::Incr4 => vals::Burst::INCR4,
                Burst::Incr8 => vals::Burst::INCR8,
                Burst::Incr16 => vals::Burst::INCR16,
                _ => unimplemented!("invalid burst size"),
            }
        }
    }

    /// DMA flow control setting.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum FlowControl {
        /// Flow control by DMA
        Dma,
        /// Flow control by peripheral
        Peripheral,
    }

    impl From<FlowControl> for vals::Pfctrl {
        fn from(flow: FlowControl) -> Self {
            match flow {
                FlowControl::Dma => vals::Pfctrl::DMA,
                FlowControl::Peripheral => vals::Pfctrl::PERIPHERAL,
            }
        }
    }

    /// DMA FIFO threshold.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum FifoThreshold {
        /// 1/4 full FIFO
        Quarter,
        /// 1/2 full FIFO
        Half,
        /// 3/4 full FIFO
        ThreeQuarters,
        /// Full FIFO
        Full,
    }

    impl From<FifoThreshold> for vals::Fth {
        fn from(value: FifoThreshold) -> Self {
            match value {
                FifoThreshold::Quarter => vals::Fth::QUARTER,
                FifoThreshold::Half => vals::Fth::HALF,
                FifoThreshold::ThreeQuarters => vals::Fth::THREE_QUARTERS,
                FifoThreshold::Full => vals::Fth::FULL,
            }
        }
    }
}

#[cfg(mdma)]
pub use mdma_only::*;
#[cfg(mdma)]
mod mdma_only {
    use pac::mdma::vals;

    use super::*;

    /// Maximum Buffer Size
    pub const MDMA_MAX_BUFFER: u8 = 0x40;

    /// Max Block Size (bytes)
    pub const MDMA_MAX_BLOCK: usize = 0x10000;

    /// Max Number of Blocks
    pub const MDMA_MAX_BLOCK_COUNT: usize = 0x1000;

    impl From<WordSize> for vals::Wordsize {
        fn from(raw: WordSize) -> Self {
            match raw {
                WordSize::OneByte => Self::BYTE,
                WordSize::TwoBytes => Self::HALF_WORD,
                WordSize::FourBytes => Self::WORD,
                WordSize::EightBytes => Self::DOUBLE_WORD,
            }
        }
    }

    impl From<Burst> for vals::Burst {
        fn from(burst: Burst) -> Self {
            match burst {
                Burst::Single => vals::Burst::SINGLE,
                Burst::Incr4 => vals::Burst::INCR4,
                Burst::Incr8 => vals::Burst::INCR8,
                Burst::Incr16 => vals::Burst::INCR16,
                Burst::Incr32 => vals::Burst::INCR32,
                Burst::Incr64 => vals::Burst::INCR64,
                Burst::Incr128 => vals::Burst::INCR128,
                Burst::Incr256 => vals::Burst::INCR256,
            }
        }
    }

    impl From<Priority> for pac::mdma::vals::Pl {
        fn from(value: Priority) -> Self {
            match value {
                Priority::Low => pac::mdma::vals::Pl::LOW,
                Priority::Medium => pac::mdma::vals::Pl::MEDIUM,
                Priority::High => pac::mdma::vals::Pl::HIGH,
                Priority::VeryHigh => pac::mdma::vals::Pl::VERY_HIGH,
            }
        }
    }
}

#[cfg(bdma)]
mod bdma_only {
    use pac::bdma::vals;

    use super::*;

    impl From<WordSize> for vals::Size {
        fn from(raw: WordSize) -> Self {
            match raw {
                WordSize::OneByte => Self::BITS8,
                WordSize::TwoBytes => Self::BITS16,
                WordSize::FourBytes => Self::BITS32,
                WordSize::EightBytes => unimplemented!(),
            }
        }
    }

    impl From<Dir> for vals::Dir {
        fn from(raw: Dir) -> Self {
            match raw {
                Dir::MemoryToPeripheral => Self::FROM_MEMORY,
                Dir::PeripheralToMemory => Self::FROM_PERIPHERAL,
                Dir::MemoryToMemory => Self::FROM_MEMORY,
            }
        }
    }
}

pub(crate) struct ChannelState {
    waker: AtomicWaker,
    complete_count: AtomicUsize,
}

impl ChannelState {
    pub(crate) const NEW: Self = Self {
        waker: AtomicWaker::new(),
        complete_count: AtomicUsize::new(0),
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(
    cs: critical_section::CriticalSection,
    #[cfg(dma)] dma_priority: interrupt::Priority,
    #[cfg(bdma)] bdma_priority: interrupt::Priority,
    #[cfg(mdma)] mdma_priority: interrupt::Priority,
) {
    foreach_interrupt! {
        ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, dma_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
        ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, bdma_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    #[cfg(mdma)]
    foreach_interrupt! {
        ($peri:ident, mdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, mdma_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_dma();
    crate::_generated::init_bdma();
    #[cfg(mdma)]
    crate::_generated::init_mdma();
}

impl<'d> Channel<'d> {
    fn info(&self) -> &'static super::ChannelInfo {
        super::info(self.id)
    }

    unsafe fn configure(
        &self,
        _request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: Increment,
        mem_size: WordSize,
        peri_size: WordSize,
        options: TransferOptions,
    ) {
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let info = self.info();
        #[cfg(feature = "_dual-core")]
        {
            use embassy_hal_internal::interrupt::InterruptExt as _;
            info.irq.enable();
        }

        #[cfg(dmamux)]
        if let Some(ref dmamux) = info.dmamux {
            super::dmamux::configure_dmamux(dmamux, _request);
        }

        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                assert!(mem_len > 0 && mem_len <= 0xFFFF);
                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.st(info.num);

                state.complete_count.store(0, Ordering::Release);
                self.clear_irqs();

                // NDTR is the number of transfers in the *peripheral* word size.
                // ex: if mem_size=1, peri_size=4 and ndtr=3 it'll do 12 mem transfers, 3 peri transfers.
                let ndtr = match (mem_size, peri_size) {
                    (WordSize::FourBytes, WordSize::OneByte) => mem_len * 4,
                    (WordSize::FourBytes, WordSize::TwoBytes) | (WordSize::TwoBytes, WordSize::OneByte) => mem_len * 2,
                    (WordSize::FourBytes, WordSize::FourBytes)
                    | (WordSize::TwoBytes, WordSize::TwoBytes)
                    | (WordSize::OneByte, WordSize::OneByte) => mem_len,
                    (WordSize::TwoBytes, WordSize::FourBytes) | (WordSize::OneByte, WordSize::TwoBytes) => {
                        assert!(mem_len % 2 == 0);
                        mem_len / 2
                    }
                    (WordSize::OneByte, WordSize::FourBytes) => {
                        assert!(mem_len % 4 == 0);
                        mem_len / 4
                    }
                    (WordSize::EightBytes, _) | (_, WordSize::EightBytes) => unimplemented!("invalid word size"),
                };

                assert!(ndtr > 0 && ndtr <= 0xFFFF);

                ch.par().write_value(peri_addr as u32);
                ch.m0ar().write_value(mem_addr as u32);
                ch.ndtr().write_value(pac::dma::regs::Ndtr(ndtr as _));
                ch.fcr().write(|w| {
                    if let Some(fth) = options.fifo_threshold {
                        // FIFO mode
                        w.set_dmdis(pac::dma::vals::Dmdis::DISABLED);
                        w.set_fth(fth.into());
                    } else if mem_size != peri_size {
                        // force FIFO mode if msize != psize
                        // packing/unpacking doesn't work in direct mode.
                        w.set_dmdis(pac::dma::vals::Dmdis::DISABLED);
                        w.set_fth(FifoThreshold::Half.into());
                    } else {
                        // Direct mode
                        w.set_dmdis(pac::dma::vals::Dmdis::ENABLED);
                    }
                });
                ch.cr().write(|w| {
                    w.set_dir(dir.into());
                    w.set_msize(mem_size.into());
                    w.set_psize(peri_size.into());
                    w.set_pl(options.priority.into());
                    match incr_mem {
                        Increment::None => {
                            w.set_minc(false);
                            w.set_pinc(false);
                        }
                        Increment::Peripheral => {
                            w.set_minc(false);
                            w.set_pinc(true);
                        }
                        Increment::Memory => {
                            w.set_minc(true);
                            w.set_pinc(false);
                        }
                        Increment::Both => {
                            w.set_minc(true);
                            w.set_pinc(true);
                        }
                    }
                    w.set_teie(true);
                    w.set_htie(options.half_transfer_ir);
                    w.set_tcie(options.complete_transfer_ir);
                    w.set_circ(options.circular);
                    #[cfg(dma_v1)]
                    w.set_trbuff(true);
                    #[cfg(dma_v2)]
                    w.set_chsel(_request);
                    w.set_pburst(options.pburst.into());
                    w.set_mburst(options.mburst.into());
                    w.set_pfctrl(options.flow_ctrl.into());
                    w.set_en(false); // don't start yet
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                assert!(mem_len > 0 && mem_len <= 0xFFFF);

                #[cfg(bdma_v2)]
                critical_section::with(|_| r.cselr().modify(|w| w.set_cs(info.num, _request)));

                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.ch(info.num);

                state.complete_count.store(0, Ordering::Release);
                self.clear_irqs();

                ch.par().write_value(peri_addr as u32);
                ch.mar().write_value(mem_addr as u32);
                ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
                ch.cr().write(|w| {
                    w.set_psize(peri_size.into());
                    w.set_msize(mem_size.into());
                    match incr_mem {
                        Increment::None => {
                            w.set_minc(false);
                            w.set_pinc(false);
                        }
                        Increment::Peripheral => {
                            w.set_minc(false);
                            w.set_pinc(true);
                        }
                        Increment::Memory => {
                            w.set_minc(true);
                            w.set_pinc(false);
                        }
                        Increment::Both => {
                            w.set_minc(true);
                            w.set_pinc(true);
                        }
                    }
                    w.set_dir(dir.into());
                    w.set_teie(true);
                    w.set_tcie(options.complete_transfer_ir);
                    w.set_htie(options.half_transfer_ir);
                    w.set_circ(options.circular);
                    w.set_pl(options.priority.into());
                    w.set_en(false); // don't start yet
                });
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => {
                use pac::mdma::vals::Incmode;

                use crate::_generated::{MEMORY_REGION_DTCM, MEMORY_REGION_ITCM};

                assert!(mem_len > 0 && mem_len <= MDMA_MAX_BLOCK * MDMA_MAX_BLOCK_COUNT);

                // Circular mode is not supported
                assert!(!options.circular);

                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.ch(info.num);

                state.complete_count.store(0, Ordering::Release);
                self.clear_irqs();

                match dir {
                    Dir::MemoryToPeripheral => {
                        ch.sar().write(|w| w.set_sar(mem_addr as u32));
                        ch.dar().write(|w| w.set_dar(peri_addr as u32));
                    }
                    _ => {
                        ch.sar().write(|w| w.set_sar(peri_addr as u32));
                        ch.dar().write(|w| w.set_dar(mem_addr as u32));
                    }
                };

                // Find the best block size/count. This is essentially a factorisation problem
                // So it's best to avoid large prime number transfer sizes.
                let mut block_count = mem_len.div_ceil(MDMA_MAX_BLOCK);
                let mut block_size = mem_len.div_ceil(block_count);

                loop {
                    // Everything matches up so we're good to go
                    if block_count * block_size == mem_len {
                        break;
                    }

                    // Try a higher block count, lower block size
                    block_count += 1;
                    block_size = mem_len.div_ceil(block_count);

                    if block_count > MDMA_MAX_BLOCK_COUNT {
                        panic!("MDMA: max block count hit");
                    }
                }

                let (sinc, dinc) = match (incr_mem, dir) {
                    (Increment::None, _) => (Incmode::FIXED, Incmode::FIXED),
                    (Increment::Both, _) => (Incmode::INCREMENT, Incmode::INCREMENT),
                    (_, Dir::MemoryToMemory) => (Incmode::INCREMENT, Incmode::INCREMENT),
                    (Increment::Peripheral, Dir::PeripheralToMemory) => (Incmode::INCREMENT, Incmode::FIXED),
                    (Increment::Peripheral, Dir::MemoryToPeripheral) => (Incmode::FIXED, Incmode::INCREMENT),
                    (Increment::Memory, Dir::PeripheralToMemory) => (Incmode::FIXED, Incmode::INCREMENT),
                    (Increment::Memory, Dir::MemoryToPeripheral) => (Incmode::INCREMENT, Incmode::FIXED),
                };

                ch.tcr().write(|w| {
                    w.set_tlen((options.buffer_size - 1) as u8);
                    match dir {
                        Dir::MemoryToPeripheral => {
                            w.set_sincos(mem_size.into());
                            w.set_ssize(mem_size.into());
                            w.set_sinc(sinc);
                            w.set_dincos(peri_size.into());
                            w.set_dsize(peri_size.into());
                            w.set_dinc(dinc);
                        }
                        _ => {
                            w.set_sincos(peri_size.into());
                            w.set_ssize(peri_size.into());
                            w.set_sinc(sinc);
                            w.set_dincos(mem_size.into());
                            w.set_dsize(mem_size.into());
                            w.set_dinc(dinc);
                        }
                    };
                });

                ch.bndtr().write(|w| {
                    w.set_bndt(block_size as u32);
                    w.set_brc(block_count as u16 - 1);
                });

                let get_bus = |addr: u32| {
                    if MEMORY_REGION_ITCM.contains(&addr) || MEMORY_REGION_DTCM.contains(&addr) {
                        pac::mdma::vals::Bus::AHB
                    } else {
                        pac::mdma::vals::Bus::SYSTEM
                    }
                };

                let mem_bus = get_bus(mem_addr as u32);
                let peri_bus = get_bus(peri_addr as u32);

                ch.tbr().write(|w| {
                    match dir {
                        Dir::MemoryToPeripheral => {
                            w.set_sbus(mem_bus);
                            w.set_dbus(peri_bus);
                        }
                        _ => {
                            w.set_sbus(peri_bus);
                            w.set_dbus(mem_bus);
                        }
                    };

                    w.set_tsel(_request);
                });

                ch.cr().write(|w| {
                    w.set_teie(true);
                    w.set_ctcie(true);
                    w.set_tcie(false);
                    w.set_btie(false);
                    w.set_brtie(false);
                    w.set_pl(options.priority.into());
                    w.set_bex(options.byte_swap);
                    w.set_wex(options.word_swap);
                    w.set_hex(options.half_word_swap);
                    w.set_en(false); // don't start yet
                });
            }
        }
    }

    fn start(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let ch = r.st(info.num);
                ch.cr().modify(|w| w.set_en(true))
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                let ch = r.ch(info.num);
                ch.cr().modify(|w| w.set_en(true));
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => {
                let ch = r.ch(info.num);
                ch.cr().modify(|w| w.set_en(true));
            }
        }
    }

    fn clear_irqs(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let isrn = info.num / 4;
                let isrbit = info.num % 4;

                r.ifcr(isrn).write(|w| {
                    w.set_htif(isrbit, true);
                    w.set_tcif(isrbit, true);
                    w.set_teif(isrbit, true);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                r.ifcr().write(|w| {
                    w.set_htif(info.num, true);
                    w.set_tcif(info.num, true);
                    w.set_teif(info.num, true);
                });
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => {
                let ch = r.ch(info.num);
                ch.ifcr().write(|w| {
                    w.set_cbrtif(true);
                    w.set_cbtif(true);
                    w.set_cctcif(true);
                    w.set_cltcif(true);
                    w.set_cteif(true);
                });
            }
        }
    }

    fn request_pause(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                // Disable the channel without overwriting the existing configuration
                r.st(info.num).cr().modify(|w| {
                    w.set_en(false);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                // Disable the channel without overwriting the existing configuration
                r.ch(info.num).cr().modify(|w| {
                    w.set_en(false);
                });
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => {
                // Disable the channel without overwriting the existing configuration
                r.ch(info.num).cr().modify(|w| {
                    w.set_en(false);
                });
            }
        }
    }

    fn request_resume(&self) {
        self.start()
    }

    fn request_reset(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                // Disable the channel. Keep the IEs enabled so the irqs still fire.
                r.st(info.num).cr().write(|w| {
                    w.set_teie(true);
                    w.set_tcie(true);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                // Disable the channel. Keep the IEs enabled so the irqs still fire.
                r.ch(info.num).cr().write(|w| {
                    w.set_teie(true);
                    w.set_tcie(true);
                });
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => {
                // Disable the channel. Keep the IEs enabled so the irqs still fire.
                r.ch(info.num).cr().modify(|m| {
                    m.set_en(false);
                });
            }
        }

        while self.is_running() {}
    }

    fn is_running(&self) -> bool {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => r.st(info.num).cr().read().en(),
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.ch(info.num);
                let en = ch.cr().read().en();
                let circular = ch.cr().read().circ();
                let tcif = state.complete_count.load(Ordering::Acquire) != 0;
                en && (circular || !tcif)
            }
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => r.ch(info.num).cr().read().en(),
        }
    }

    fn get_remaining_transfers(&self) -> u32 {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => r.st(info.num).ndtr().read().ndt() as u32,
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => r.ch(info.num).ndtr().read().ndt() as u32,
            #[cfg(mdma)]
            DmaInfo::Mdma(r) => r.ch(info.num).bndtr().read().bndt(),
        }
    }

    fn disable_circular_mode(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(regs) => regs.st(info.num).cr().modify(|w| {
                w.set_circ(false);
            }),
            #[cfg(bdma)]
            DmaInfo::Bdma(regs) => regs.ch(info.num).cr().modify(|w| {
                w.set_circ(false);
            }),
            #[cfg(mdma)]
            DmaInfo::Mdma(_regs) => (),
        }
    }

    fn poll_stop(&self) -> Poll<()> {
        compiler_fence(Ordering::SeqCst);

        if !self.is_running() {
            fence(Ordering::Acquire);

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    /// Create a memory DMA transfer (memory to memory), using raw pointers.
    pub unsafe fn transfer<'a, MW: Word, PW: Word>(
        &'a mut self,
        request: Request,
        buf: *const [MW],
        dest_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.transfer_raw(request, buf as *const MW as *mut u32, buf.len(), dest_addr, options)
    }

    /// Create a memory DMA transfer (memory to memory), using raw pointers.
    pub unsafe fn transfer_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        request: Request,
        src_addr: *const MW,
        src_size: usize,
        dest_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        assert!(src_size > 0 && src_size <= 0xFFFF);

        self.configure(
            request,
            Dir::MemoryToMemory,
            src_addr as *mut u32,
            dest_addr as *mut u32,
            src_size,
            Increment::Both,
            MW::size(),
            PW::size(),
            options,
        );
        self.start();
        Transfer {
            _wake_guard: self.info().wake_guard(),
            channel: self.reborrow(),
        }
    }

    /// Create a read DMA transfer (peripheral to memory).
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        request: Request,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.read_raw(request, peri_addr, buf, options)
    }

    /// Create a read DMA transfer (peripheral to memory), using raw pointers.
    pub unsafe fn read_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        request: Request,
        peri_addr: *mut PW,
        buf: *mut [MW],
        options: TransferOptions,
    ) -> Transfer<'a> {
        let mem_len = buf.len();

        self.configure(
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut MW as *mut u32,
            mem_len,
            Increment::Memory,
            MW::size(),
            PW::size(),
            options,
        );
        self.start();
        Transfer {
            _wake_guard: self.info().wake_guard(),
            channel: self.reborrow(),
        }
    }

    /// Create a write DMA transfer (memory to peripheral).
    pub unsafe fn write<'a, MW: Word, PW: Word>(
        &'a mut self,
        request: Request,
        buf: &'a [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        self.write_raw(request, buf, peri_addr, options)
    }

    /// Create a write DMA transfer (memory to peripheral), using raw pointers.
    pub unsafe fn write_raw<'a, MW: Word, PW: Word>(
        &'a mut self,
        request: Request,
        buf: *const [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Transfer<'a> {
        let mem_len = buf.len();

        self.configure(
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            buf as *const MW as *mut u32,
            mem_len,
            Increment::Memory,
            MW::size(),
            PW::size(),
            options,
        );
        self.start();
        Transfer {
            _wake_guard: self.info().wake_guard(),
            channel: self.reborrow(),
        }
    }

    /// Create a write DMA transfer (memory to peripheral), writing the same value repeatedly.
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        request: Request,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        assert!(count > 0 && count <= 0xFFFF);

        self.configure(
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            repeated as *const W as *mut u32,
            count,
            Increment::None,
            W::size(),
            W::size(),
            options,
        );
        self.start();
        Transfer {
            _wake_guard: self.info().wake_guard(),
            channel: self.reborrow(),
        }
    }
}

/// DMA transfer.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
}

impl<'a> Transfer<'a> {
    /// Request the transfer to pause, keeping the existing configuration for this channel.
    ///
    /// To resume the transfer, call [`request_resume`](Self::request_resume) again.
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Request the transfer to resume after having been paused.
    pub fn request_resume(&mut self) {
        self.channel.request_resume()
    }

    /// Request the DMA to reset.
    ///
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    pub fn request_reset(&mut self) {
        self.channel.request_reset()
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_pause`](Self::request_pause).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u32 {
        self.channel.get_remaining_transfers()
    }

    /// Blocking wait until the transfer finishes.
    pub fn blocking_wait(mut self) {
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);

        core::mem::forget(self);
    }

    pub(crate) unsafe fn unchecked_extend_lifetime(self) -> Transfer<'static> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        self.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

impl<'a> Unpin for Transfer<'a> {}
impl<'a> Future for Transfer<'a> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state: &ChannelState = &STATE[self.channel.id as usize];

        state.waker.register(cx.waker());

        compiler_fence(Ordering::SeqCst);
        if self.is_running() {
            Poll::Pending
        } else {
            fence(Ordering::Acquire);

            Poll::Ready(())
        }
    }
}

// ==============================

struct DmaCtrlImpl<'a>(Channel<'a>);

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        self.0.get_remaining_transfers() as _
    }

    fn reset_complete_count(&mut self) -> usize {
        let state = &STATE[self.0.id as usize];
        #[cfg(not(armv6m))]
        return state.complete_count.swap(0, Ordering::AcqRel);
        #[cfg(armv6m)]
        return critical_section::with(|_| {
            let x = state.complete_count.load(Ordering::Acquire);
            state.complete_count.store(0, Ordering::Release);
            x
        });
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.0.id as usize].waker.register(waker);
    }
}

/// Ringbuffer for receiving data using DMA circular mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Channel<'a>,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let channel: Channel<'a> = channel.into();

        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len();
        let dir = Dir::PeripheralToMemory;
        let data_size = W::size();

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            Increment::Memory,
            data_size,
            data_size,
            options,
        );

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Set the frame alignment for the ring buffer.
    ///
    /// See [`ReadableDmaRingBuffer::set_alignment`] for details.
    pub fn set_alignment(&mut self, alignment: usize) {
        self.ringbuf.set_alignment(alignment);
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.ringbuf.read(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, Error> {
        self.ringbuf
            .read_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Request the transfer to resume after having been paused.
    pub fn request_resume(&mut self) {
        self.channel.request_resume()
    }

    /// Request the DMA to reset.
    ///
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    pub fn request_reset(&mut self) {
        self.channel.request_reset()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_reset`](Self::request_reset).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is full.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when the buffer is full.
    ///
    /// This is designed to be used with streaming input data such as the
    /// I2S/SAI or ADC.
    ///
    /// When using the UART, you probably want `request_reset()`.
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for ReadableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

/// Ringbuffer for writing data using DMA circular mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Channel<'a>,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let channel: Channel<'a> = channel.into();

        let len = buffer.len();
        let dir = Dir::MemoryToPeripheral;
        let data_size = W::size();
        let buffer_ptr = buffer.as_mut_ptr();

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            Increment::Memory,
            data_size,
            data_size,
            options,
        );

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Write elements directly to the raw buffer.
    /// This can be used to fill the buffer before starting the DMA transfer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write_immediate(buf)
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        self.ringbuf
            .write_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ringbuf
            .wait_write_error(&mut DmaCtrlImpl(self.channel.reborrow()))
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the DMA to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_reset(&mut self) {
        self.channel.request_reset()
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_reset`](Self::request_reset).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is empty.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when all available data has been written.
    ///
    /// This is designed to be used with streaming output data such as the
    /// I2S/SAI or DAC.
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for WritableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}
