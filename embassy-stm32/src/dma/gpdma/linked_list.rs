//! Implementation of the GPDMA linked list and linked list items.
#![macro_use]

#[cfg(gpdma)]
use stm32_metapac::gpdma::regs;
#[cfg(gpdma)]
use stm32_metapac::gpdma::vals::Dreq;
#[cfg(not(gpdma))]
use stm32_metapac::lpdma::regs;
#[cfg(not(gpdma))]
use stm32_metapac::lpdma::vals::Dreq;

use crate::dma::word::{Word, WordSize};
use crate::dma::{Dir, Request};

/// The mode in which to run the linked list.
#[derive(Debug)]
pub enum RunMode {
    /// List items are not linked together.
    Unlinked,
    /// The list is linked sequentially and only run once.
    Once,
    /// The list is linked sequentially, and the end of the list is linked to the beginning.
    Circular,
}

/// Configuration common to all linked-list item types.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ItemConfig {
    /// Transfer complete event mode for this item.
    pub transfer_complete_mode: super::TransferCompleteMode,
}

impl Default for ItemConfig {
    fn default() -> Self {
        Self {
            transfer_complete_mode: super::TransferCompleteMode::EachBlock,
        }
    }
}

/// Backwards-compatible alias.
pub type LinearItemConfig = ItemConfig;

/// Trait for linked-list item types usable in a generic [`Table`].
pub trait LinkedListItem: Copy + Default + Sized {
    /// Per-item configuration passed at construction time.
    type Config: Default + Copy;

    /// Whether this item type requires a 2D-capable channel.
    const IS_2D: bool;

    /// Create a new read DMA transfer item (peripheral to memory).
    ///
    /// # Safety
    /// The caller must ensure addresses and buffer remain valid for the transfer duration.
    unsafe fn new_read<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buf: &mut [MW],
        config: Self::Config,
    ) -> Self;

    /// Create a new write DMA transfer item (memory to peripheral).
    ///
    /// # Safety
    /// The caller must ensure addresses and buffer remain valid for the transfer duration.
    unsafe fn new_write<MW: Word, PW: Word>(
        request: Request,
        buf: &[MW],
        peri_addr: *mut PW,
        config: Self::Config,
    ) -> Self;

    /// Link to the next item at the given offset address.
    fn link_to(&mut self, next: u16);

    /// Unlink the next item (disables channel update bits).
    fn unlink(&mut self);

    /// The item's transfer count in number of destination words.
    fn transfer_count(&self) -> usize;
}

/// The common fields shared by all linked-list item types (TR1, TR2, BR1, SAR, DAR).
///
/// This is the first 5 words of every GPDMA linked-list descriptor.
/// Both [`LinearItem`] and [`TwoDItem`](super::two_d::TwoDItem) embed
/// this as their first field so the `#[repr(C)]` hardware layout is preserved.
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct Item {
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
}

impl Item {
    /// Build a new item from raw transfer parameters.
    ///
    /// # Safety
    /// The caller must ensure `peri_addr` and `mem_addr` are valid for the transfer.
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn new(
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        dst_size: WordSize,
    ) -> Self {
        let Ok(bndt) = (mem_len * data_size.bytes()).try_into() else {
            panic!("DMA transfers may not be larger than 65535 bytes.");
        };

        let mut br1 = regs::ChBr1(0);
        br1.set_bndt(bndt);

        let mut tr1 = regs::ChTr1(0);
        tr1.set_sdw(data_size.into());
        tr1.set_ddw(dst_size.into());
        tr1.set_sinc(dir == Dir::MemoryToPeripheral && incr_mem);
        tr1.set_dinc(dir == Dir::PeripheralToMemory && incr_mem);

        #[cfg(gpdma)]
        {
            use stm32_metapac::gpdma::vals::Ap;
            tr1.set_sap(match dir {
                Dir::MemoryToPeripheral => Ap::Port0,
                Dir::PeripheralToMemory => Ap::Port1,
                Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
            });
            tr1.set_dap(match dir {
                Dir::MemoryToPeripheral => Ap::Port1,
                Dir::PeripheralToMemory => Ap::Port0,
                Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
            });
        }

        let mut tr2 = regs::ChTr2(0);
        tr2.set_dreq(match dir {
            Dir::MemoryToPeripheral => Dreq::DestinationPeripheral,
            Dir::PeripheralToMemory => Dreq::SourcePeripheral,
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
        });
        tr2.set_reqsel(request);

        let (sar, dar) = match dir {
            Dir::MemoryToPeripheral => (mem_addr as _, peri_addr as _),
            Dir::PeripheralToMemory => (peri_addr as _, mem_addr as _),
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
        };

        Self {
            tr1,
            tr2,
            br1,
            sar,
            dar,
        }
    }

    /// Apply the common item configuration fields.
    ///
    /// This is the single place to expand when new common config fields are added.
    pub(super) fn apply_config(&mut self, config: &ItemConfig) {
        self.tr2.set_tcem(config.transfer_complete_mode.into());
    }

    /// The item's transfer count in number of destination words.
    pub fn transfer_count(&self) -> usize {
        let word_size: WordSize = self.tr1.ddw().into();
        self.br1.bndt() as usize / word_size.bytes()
    }
}

/// A linked-list item for linear GPDMA transfers.
///
/// Also works for 2D-capable GPDMA channels, but does not use 2D capabilities.
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct LinearItem {
    /// Common item fields (TR1, TR2, BR1, SAR, DAR).
    pub item: Item,
    /// Linked-list address register.
    pub llr: regs::ChLlr,
}

impl LinkedListItem for LinearItem {
    type Config = ItemConfig;
    const IS_2D: bool = false;

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

impl LinearItem {
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
        config: ItemConfig,
    ) -> Self {
        let mut item = Item::new(
            request, dir, peri_addr, mem_addr, mem_len, incr_mem, data_size, dst_size,
        );
        item.apply_config(&config);

        Self {
            item,
            llr: regs::ChLlr(0),
        }
    }
}

/// A table of linked list items.
#[repr(C)]
pub struct Table<T: LinkedListItem, const N: usize> {
    /// The items.
    pub items: [T; N],
}

impl<T: LinkedListItem, const N: usize> Table<T, N> {
    /// Create a new table.
    pub fn new(items: [T; N]) -> Self {
        assert!(N > 0);

        Self { items }
    }

    /// Create a single-LLI circular linked-list table.
    ///
    /// Uses one linked-list item covering the entire buffer, linked to itself.
    /// This avoids multi-LLI race conditions in position tracking while still
    /// providing half-transfer and transfer-complete interrupts for wakeups.
    pub unsafe fn new_circular<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buffer: &mut [MW],
        direction: Dir,
        config: T::Config,
    ) -> Table<T, 1> {
        let item = match direction {
            Dir::MemoryToPeripheral => T::new_write(request, &buffer[..], peri_addr, config),
            Dir::PeripheralToMemory => T::new_read(request, peri_addr, &mut buffer[..], config),
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
        };

        Table::new([item])
    }

    /// Create a ping-pong linked-list table.
    ///
    /// This uses two linked-list items, one for each half of the buffer.
    pub unsafe fn new_ping_pong<MW: Word, PW: Word>(
        request: Request,
        peri_addr: *mut PW,
        buffer: &mut [MW],
        direction: Dir,
        config: T::Config,
    ) -> Table<T, 2> {
        let half_len = buffer.len() / 2;
        assert_eq!(half_len * 2, buffer.len());

        let items = match direction {
            Dir::MemoryToPeripheral => [
                T::new_write(request, &mut buffer[..half_len], peri_addr, config),
                T::new_write(request, &mut buffer[half_len..], peri_addr, config),
            ],
            Dir::PeripheralToMemory => [
                T::new_read(request, peri_addr, &mut buffer[..half_len], config),
                T::new_read(request, peri_addr, &mut buffer[half_len..], config),
            ],
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for linked-list items"),
        };

        Table::new(items)
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
