//! Implementation of the GPDMA linked list and linked list items.
#![macro_use]

use stm32_metapac::gpdma::regs;
use stm32_metapac::gpdma::vals::Dreq;

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

/// A linked-list item for linear GPDMA transfers.
///
/// Also works for 2D-capable GPDMA channels, but does not use 2D capabilities.
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct LinearItem {
    /// Transfer register 1.
    pub tr1: regs::ChTr1,
    /// Transfer register 2.
    pub tr2: regs::ChTr2,
    /// Block register 2.
    pub br1: regs::ChBr1,
    /// Source address register.
    pub sar: u32,
    /// Destination address register.
    pub dar: u32,
    /// Linked-list address register.
    pub llr: regs::ChLlr,
}

impl LinearItem {
    /// Create a new read DMA transfer (peripheral to memory).
    pub unsafe fn new_read<'d, W: Word>(request: Request, peri_addr: *mut W, buf: &'d mut [W]) -> Self {
        Self::new_inner(
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut [W] as *mut W as *mut u32,
            buf.len(),
            true,
            W::size(),
            W::size(),
        )
    }

    /// Create a new write DMA transfer (memory to peripheral).
    pub unsafe fn new_write<'d, MW: Word, PW: Word>(request: Request, buf: &'d [MW], peri_addr: *mut PW) -> Self {
        Self::new_inner(
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            buf as *const [MW] as *const MW as *mut u32,
            buf.len(),
            true,
            MW::size(),
            PW::size(),
        )
    }

    unsafe fn new_inner(
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        dst_size: WordSize,
    ) -> Self {
        // BNDT is specified as bytes, not as number of transfers.
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

        let mut tr2 = regs::ChTr2(0);
        tr2.set_dreq(match dir {
            Dir::MemoryToPeripheral => Dreq::DESTINATION_PERIPHERAL,
            Dir::PeripheralToMemory => Dreq::SOURCE_PERIPHERAL,
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for LinearItem"),
        });
        tr2.set_reqsel(request);

        let (sar, dar) = match dir {
            Dir::MemoryToPeripheral => (mem_addr as _, peri_addr as _),
            Dir::PeripheralToMemory => (peri_addr as _, mem_addr as _),
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for LinearItem"),
        };

        let llr = regs::ChLlr(0);

        Self {
            tr1,
            tr2,
            br1,
            sar,
            dar,
            llr,
        }
    }

    /// Link to the next linear item at the given address.
    ///
    /// Enables channel update bits.
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

    /// Unlink the next linear item.
    ///
    /// Disables channel update bits.
    fn unlink(&mut self) {
        self.llr = regs::ChLlr(0);
    }

    /// The item's transfer count in number of words.
    fn transfer_count(&self) -> usize {
        let word_size: WordSize = self.tr1.ddw().into();
        self.br1.bndt() as usize / word_size.bytes()
    }
}

/// A table of linked list items.
#[repr(C)]
pub struct Table<const ITEM_COUNT: usize> {
    /// The items.
    pub items: [LinearItem; ITEM_COUNT],
}

impl<const ITEM_COUNT: usize> Table<ITEM_COUNT> {
    /// Create a new table.
    pub fn new(items: [LinearItem; ITEM_COUNT]) -> Self {
        assert!(!items.is_empty());

        Self { items }
    }

    /// Create a ping-pong linked-list table.
    ///
    /// This uses two linked-list items, one for each half of the buffer.
    pub unsafe fn new_ping_pong<W: Word>(
        request: Request,
        peri_addr: *mut W,
        buffer: &mut [W],
        direction: Dir,
    ) -> Table<2> {
        // Buffer halves should be the same length.
        let half_len = buffer.len() / 2;
        assert_eq!(half_len * 2, buffer.len());

        let items = match direction {
            Dir::MemoryToPeripheral => [
                LinearItem::new_write(request, &mut buffer[..half_len], peri_addr),
                LinearItem::new_write(request, &mut buffer[half_len..], peri_addr),
            ],
            Dir::PeripheralToMemory => [
                LinearItem::new_read(request, peri_addr, &mut buffer[..half_len]),
                LinearItem::new_read(request, peri_addr, &mut buffer[half_len..]),
            ],
            Dir::MemoryToMemory => panic!("memory-to-memory transfers are not valid for LinearItem"),
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
