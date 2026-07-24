//! 2D GPDMA linked-list items and tables.
//!
//! Provides [`TwoDItem`] and [`TwoDTable`] for GPDMA channels that support
//! 2D addressing (block repeat with per-burst and per-block address offsets).

use core::mem::size_of;

use stm32_metapac::gpdma::regs;
use stm32_metapac::gpdma::vals::Dec;

use super::linked_list::{RunMode, build_common_fields};
use crate::dma::word::{Word, WordSize};
use crate::dma::{Dir, Request};

/// Configuration for 2D (block-repeat with address offsets) DMA transfers.
///
/// These parameters control how addresses are stepped after each burst
/// and each block repeat.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TwoDConfig {
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

/// A linked-list item for 2D GPDMA transfers (block repeat with address offsets).
///
/// This is an 8-word (32 byte) descriptor that includes TR3 and BR2
/// registers for per-burst and per-block-repeat address stepping.
/// Only usable on GPDMA channels that support 2D addressing.
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct TwoDItem {
    /// Transfer register 1.
    pub tr1: regs::ChTr1,
    /// Transfer register 2.
    pub tr2: regs::ChTr2,
    /// Block register 1.
    pub br1: regs::ChBr1,
    /// Source address register.
    pub sar: u32,
    /// Destination address register.
    pub dar: u32,
    /// Transfer register 3 (per-burst address offsets).
    pub tr3: regs::ChTr3,
    /// Block register 2 (per-block-repeat address offsets).
    pub br2: regs::ChBr2,
    /// Linked-list address register.
    pub llr: regs::ChLlr,
}

const _: () = core::assert!(size_of::<TwoDItem>() == 32);

impl TwoDItem {
    /// Create a new 2D read DMA transfer (peripheral to memory).
    pub unsafe fn new_read<'d, MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buf: &'d mut [MW],
        config: TwoDConfig,
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

    /// Create a new 2D write DMA transfer (memory to peripheral).
    pub unsafe fn new_write<'d, MW: Word, PW: Word>(
        request: Request,
        buf: &'d [MW],
        peri_addr: *mut PW,
        config: TwoDConfig,
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
        let (tr1, tr2, mut br1, sar, dar) = build_common_fields(
            request, dir, peri_addr, mem_addr, mem_len, incr_mem, data_size, dst_size,
        );

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

        br1.set_brc(config.block_repeat_count);

        let dec = |negative: bool| if negative { Dec::Subtract } else { Dec::Add };

        br1.set_sdec(dec(config.src_addr_offset < 0));
        br1.set_ddec(dec(config.dst_addr_offset < 0));
        br1.set_brsdec(dec(config.block_src_addr_offset < 0));
        br1.set_brddec(dec(config.block_dst_addr_offset < 0));

        let mut tr3 = regs::ChTr3(0);
        tr3.set_sao(config.src_addr_offset.unsigned_abs());
        tr3.set_dao(config.dst_addr_offset.unsigned_abs());

        let mut br2 = regs::ChBr2(0);
        br2.set_brsao(config.block_src_addr_offset.unsigned_abs() as u16);
        br2.set_brdao(config.block_dst_addr_offset.unsigned_abs() as u16);

        Self {
            tr1,
            tr2,
            br1,
            sar,
            dar,
            tr3,
            br2,
            llr: regs::ChLlr(0),
        }
    }

    /// Link to the next 2D item at the given offset address.
    ///
    /// Enables update bits for all 8 fields (UT1, UT2, UB1, USA, UDA, UT3, UB2, ULL).
    pub fn link_to(&mut self, next: u16) {
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

    /// Unlink the next 2D item.
    ///
    /// Disables channel update bits.
    pub fn unlink(&mut self) {
        self.llr = regs::ChLlr(0);
    }

    /// Set the transfer complete event mode (`TR2.TCEM`) for this item.
    ///
    /// In linked-list mode, the channel's TR2 register is overwritten by
    /// each LLI when `UT2` is set, so TCEM must be configured per-item.
    ///
    /// See [`TransferCompleteMode`](super::TransferCompleteMode) for details.
    pub fn set_transfer_complete_mode(&mut self, mode: super::TransferCompleteMode) {
        self.tr2.set_tcem(mode.into());
    }

    /// The item's transfer count in number of destination words.
    pub fn transfer_count(&self) -> usize {
        let word_size: WordSize = self.tr1.ddw().into();
        self.br1.bndt() as usize / word_size.bytes()
    }
}

/// A table of 2D linked list items.
#[repr(C)]
pub struct TwoDTable<const ITEM_COUNT: usize> {
    /// The items.
    pub items: [TwoDItem; ITEM_COUNT],
}

impl<const ITEM_COUNT: usize> TwoDTable<ITEM_COUNT> {
    /// Create a new table.
    pub fn new(items: [TwoDItem; ITEM_COUNT]) -> Self {
        assert!(!items.is_empty());

        Self { items }
    }

    /// Create a single-LLI circular 2D linked-list table.
    ///
    /// Uses one 2D item covering the entire buffer, linked to itself.
    pub unsafe fn new_circular<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buffer: &mut [MW],
        direction: Dir,
        config: TwoDConfig,
    ) -> TwoDTable<1> {
        let item = match direction {
            Dir::MemoryToPeripheral => TwoDItem::new_write(request, &buffer[..], peri_addr, config),
            Dir::PeripheralToMemory => TwoDItem::new_read(request, peri_addr, &mut buffer[..], config),
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for TwoDItem"),
        };

        TwoDTable::new([item])
    }

    /// Create a ping-pong 2D linked-list table.
    ///
    /// This uses two 2D items, one for each half of the buffer.
    pub unsafe fn new_ping_pong<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buffer: &mut [MW],
        direction: Dir,
        config: TwoDConfig,
    ) -> TwoDTable<2> {
        let half_len = buffer.len() / 2;
        assert_eq!(half_len * 2, buffer.len());

        let items = match direction {
            Dir::MemoryToPeripheral => [
                TwoDItem::new_write(request, &mut buffer[..half_len], peri_addr, config),
                TwoDItem::new_write(request, &mut buffer[half_len..], peri_addr, config),
            ],
            Dir::PeripheralToMemory => [
                TwoDItem::new_read(request, peri_addr, &mut buffer[..half_len], config),
                TwoDItem::new_read(request, peri_addr, &mut buffer[half_len..], config),
            ],
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for TwoDItem"),
        };

        TwoDTable::new(items)
    }

    /// Link the table as given by the run mode.
    pub fn link(&mut self, run_mode: RunMode) {
        if matches!(run_mode, RunMode::Once | RunMode::Circular) {
            self.link_sequential();
        }

        if matches!(run_mode, RunMode::Circular) {
            self.link_repeat();
        }
    }

    /// The number of linked list items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The total transfer count of the table in number of words.
    pub fn transfer_count(&self) -> usize {
        let mut count = 0;
        for item in self.items {
            count += item.transfer_count() as usize
        }

        count
    }

    /// Link items of given indices together: first -> second.
    pub fn link_indices(&mut self, first: usize, second: usize) {
        assert!(first < self.len());
        assert!(second < self.len());

        let second_item = self.offset_address(second);
        self.items[first].link_to(second_item);
    }

    /// Link items sequentially.
    pub fn link_sequential(&mut self) {
        if self.len() > 1 {
            for index in 0..(self.items.len() - 1) {
                let next = self.offset_address(index + 1);
                self.items[index].link_to(next);
            }
        }
    }

    /// Link last to first item.
    pub fn link_repeat(&mut self) {
        let first_address = self.offset_address(0);
        self.items.last_mut().unwrap().link_to(first_address);
    }

    /// Unlink all items.
    pub fn unlink(&mut self) {
        for item in self.items.iter_mut() {
            item.unlink();
        }
    }

    /// Linked list base address (upper 16 address bits).
    pub fn base_address(&self) -> u16 {
        ((&raw const self.items as u32) >> 16) as _
    }

    /// Linked list offset address (lower 16 address bits) at the selected index.
    pub fn offset_address(&self, index: usize) -> u16 {
        assert!(self.items.len() > index);

        let address = &raw const self.items[index] as _;

        // Ensure 32 bit address alignment.
        assert_eq!(address & 0b11, 0);

        address
    }
}
