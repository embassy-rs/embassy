//! 2D GPDMA linked-list items.
//!
//! Provides [`TwoDItem`] and [`TwoDConfig`] for GPDMA channels that support
//! 2D addressing (block repeat with per-burst and per-block address offsets).

use core::mem::size_of;

use stm32_metapac::gpdma::regs;
use stm32_metapac::gpdma::vals::Dec;

use super::linked_list::{Item, ItemConfig, LinkedListItem};
use crate::dma::word::{Word, WordSize};
use crate::dma::{Dir, Request};

/// Configuration for 2D (block-repeat with address offsets) DMA transfers.
///
/// These parameters control how addresses are stepped after each burst
/// and each block repeat.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TwoDConfig {
    /// Base linear configuration (transfer complete mode, etc.).
    pub linear: ItemConfig,
    /// Number of block repeats (0 = single block, no repeat).
    pub block_repeat_count: u16,
    /// Per-burst source address offset (13-bit signed, in bytes).
    pub src_addr_offset: i16,
    /// Per-burst destination address offset (13-bit signed, in bytes).
    pub dst_addr_offset: i16,
    /// Per-block-repeat source address offset (range: -65535..=65535, in bytes).
    /// Applied at the end of each block.
    pub block_src_addr_offset: i32,
    /// Per-block-repeat destination address offset (range: -65535..=65535, in bytes).
    /// Applied at the end of each block.
    pub block_dst_addr_offset: i32,
}

impl Default for TwoDConfig {
    fn default() -> Self {
        Self {
            linear: ItemConfig::default(),
            block_repeat_count: 0,
            src_addr_offset: 0,
            dst_addr_offset: 0,
            block_src_addr_offset: 0,
            block_dst_addr_offset: 0,
        }
    }
}

/// A linked-list item for 2D GPDMA transfers (block repeat with address offsets).
///
/// This is an 8-word (32 byte) descriptor that includes TR3 and BR2
/// registers for per-burst and per-block-repeat address stepping.
/// Only usable on GPDMA channels that support 2D addressing.
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct TwoDItem {
    /// Common item fields (TR1, TR2, BR1, SAR, DAR).
    pub item: Item,
    /// Transfer register 3 (per-burst address offsets).
    pub tr3: regs::ChTr3,
    /// Block register 2 (per-block-repeat address offsets).
    pub br2: regs::ChBr2,
    /// Linked-list address register.
    pub llr: regs::ChLlr,
}

const _: () = core::assert!(size_of::<TwoDItem>() == 32);

impl LinkedListItem for TwoDItem {
    type Config = TwoDConfig;
    const IS_2D: bool = true;

    unsafe fn new_read<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buf: &mut [MW],
        config: Self::Config,
    ) -> Self {
        Self::new_inner(
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut [MW] as *mut MW as *mut u32,
            buf.len(),
            true,
            PW::size(),
            MW::size(),
            config,
        )
    }

    unsafe fn new_write<MW: Word, PW: Word>(
        request: Request,
        buf: &[MW],
        peri_addr: *mut PW,
        config: Self::Config,
    ) -> Self {
        Self::new_inner(
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            buf as *const [MW] as *const MW as *mut u32,
            buf.len(),
            true,
            MW::size(),
            PW::size(),
            config,
        )
    }

    fn link_to(&mut self, next: u16) {
        let mut llr = regs::ChLlr(0);

        llr.set_ut1(true);
        llr.set_ut2(true);
        llr.set_ub1(true);
        llr.set_usa(true);
        llr.set_uda(true);
        llr.set_ut3(true);
        llr.set_ub2(true);
        llr.set_ull(true);

        // Lower two bits are ignored: 32 bit aligned.
        llr.set_la(next >> 2);

        self.llr = llr;
    }

    fn unlink(&mut self) {
        self.llr = regs::ChLlr(0);
    }

    fn transfer_count(&self) -> usize {
        self.item.transfer_count()
    }
}

impl TwoDItem {
    #[allow(clippy::too_many_arguments)]
    unsafe fn new_inner(
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        dst_size: WordSize,
        config: TwoDConfig,
    ) -> Self {
        let mut item = Item::new(
            request, dir, peri_addr, mem_addr, mem_len, incr_mem, data_size, dst_size,
        );
        item.apply_config(&config.linear);

        assert!(
            config.block_repeat_count <= 0x7FF,
            "block_repeat_count must fit in 11 bits (0..=2047)"
        );
        assert!(
            config.src_addr_offset.unsigned_abs() <= 0x1FFF,
            "src_addr_offset magnitude must fit in 13 bits (0..=8191)"
        );
        assert!(
            config.dst_addr_offset.unsigned_abs() <= 0x1FFF,
            "dst_addr_offset magnitude must fit in 13 bits (0..=8191)"
        );
        assert!(
            config.block_src_addr_offset.unsigned_abs() <= 0xFFFF,
            "block_src_addr_offset magnitude must fit in 16 bits (0..=65535)"
        );
        assert!(
            config.block_dst_addr_offset.unsigned_abs() <= 0xFFFF,
            "block_dst_addr_offset magnitude must fit in 16 bits (0..=65535)"
        );

        item.br1.set_brc(config.block_repeat_count);

        let dec = |negative: bool| if negative { Dec::Subtract } else { Dec::Add };

        item.br1.set_sdec(dec(config.src_addr_offset < 0));
        item.br1.set_ddec(dec(config.dst_addr_offset < 0));
        item.br1.set_brsdec(dec(config.block_src_addr_offset < 0));
        item.br1.set_brddec(dec(config.block_dst_addr_offset < 0));

        let mut tr3 = regs::ChTr3(0);
        tr3.set_sao(config.src_addr_offset.unsigned_abs());
        tr3.set_dao(config.dst_addr_offset.unsigned_abs());

        let mut br2 = regs::ChBr2(0);
        br2.set_brsao(config.block_src_addr_offset.unsigned_abs() as u16);
        br2.set_brdao(config.block_dst_addr_offset.unsigned_abs() as u16);

        Self {
            item,
            tr3,
            br2,
            llr: regs::ChLlr(0),
        }
    }
}
