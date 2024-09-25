// This module contains a more or less full abstraction for message ram,
// we don't want to remove dead code as it might be used in future
// feature implementations.
#![allow(dead_code)]

use core::marker::PhantomData;

use element::tx_event::TxEventElement;
use volatile_register::RW;

mod element;
pub(crate) use element::enums::{DataLength, FilterElementConfig, FilterType, FrameFormat, IdType};
pub(crate) use element::{
    filter_extended::ExtendedFilter, filter_standard::StandardFilter, rx_buffer::RxFifoElementHeader,
    tx_buffer::TxBufferElementHeader,
};

use crate::can::config::MessageRamConfig;

/// Configuration for MessageRam layout.
pub struct MessageRam {
    // 32 bit words
    pub(crate) base_ptr: *mut RW<u32>,

    // Full: 0-128 elements / 0-128 words
    // Simplified: 28 elements / 28 words
    pub(crate) standard_filter: Elements<SimpleElement<StandardFilter>>,

    // Full: 0-64 elements / 0-128 words
    // Simplified: 8 elements / 16 words
    pub(crate) extended_filter: Elements<SimpleElement<ExtendedFilter>>,

    // Full: 0-64 elements / 0-1152 words
    // Simplified: 3 elements / 54 words
    // x 2
    pub(crate) rx_fifos: [Elements<HeaderElement<RxFifoElementHeader>>; 2],

    // Full: 0-64 elements / 0-1152 words
    // Simplified: Does not exist in the simplified peripheral variant.
    #[cfg(can_fdcan_h7)]
    pub(crate) rx_buffer: Elements<HeaderElement<RxFifoElementHeader>>,

    // Full: 0-32 elements / 0-64 words
    // Simplified: 3 elements / 6 words
    pub(crate) tx_event_fifo: Elements<SimpleElement<TxEventElement>>,

    // Full: 0-32 elements / 0-576 words
    // Simplified: 3 elements / 54 words
    // Simplified variant does not support TX buffer, only FIFO/Queue.
    pub(crate) tx_elements: Elements<HeaderElement<TxBufferElementHeader>>,
    #[cfg(can_fdcan_h7)]
    pub(crate) tx_buffer_len: usize,
    pub(crate) tx_queue_len: usize,

    // Full: 0-64 elements / 0-128 words
    // Simplified: Does not exist in the simplified peripheral variant.
    #[cfg(can_fdcan_h7)]
    pub(crate) trigger_memory: Elements<()>,
}

unsafe impl Sync for MessageRam {}

impl MessageRam {
    pub(crate) const DEFAULT: MessageRam = MessageRam {
        base_ptr: core::ptr::null_mut(),
        standard_filter: Elements::EMPTY,
        extended_filter: Elements::EMPTY,
        rx_fifos: [Elements::EMPTY, Elements::EMPTY],
        #[cfg(can_fdcan_h7)]
        rx_buffer: Elements::EMPTY,
        tx_event_fifo: Elements::EMPTY,
        tx_elements: Elements::EMPTY,
        #[cfg(can_fdcan_h7)]
        tx_buffer_len: 0,
        tx_queue_len: 0,
        #[cfg(can_fdcan_h7)]
        trigger_memory: Elements::EMPTY,
    };
}

#[repr(C)]
pub(crate) struct SimpleElement<H: Sized> {
    pub(crate) data: H,
}

#[repr(C)]
pub(crate) struct HeaderElement<H: Sized> {
    pub(crate) header: H,
    pub(crate) data: [RW<u32>],
}

pub(crate) struct Elements<E: ?Sized> {
    _phantom: PhantomData<E>,
    base: *mut RW<u32>,
    element_size: usize,
    element_len: usize,
}

impl<E: ?Sized> Elements<E> {
    const EMPTY: Self = Elements {
        _phantom: PhantomData,
        base: core::ptr::null_mut(),
        element_size: 8,
        element_len: 0,
    };

    unsafe fn new(base: *mut RW<u32>, element_len: usize, element_size: usize) -> Self {
        Self {
            _phantom: PhantomData,
            base,
            element_len,
            element_size,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.element_len
    }
}

impl<H: Sized> Elements<SimpleElement<H>> {
    pub(crate) fn get_mut(&self, index: usize) -> &mut SimpleElement<H> {
        assert!(index < self.element_len);

        // Offset of the first item that belons to the element.
        assert!(self.element_size == size_of::<H>());
        let item_index = index * self.element_size;

        unsafe {
            let start = self.base.add(item_index);
            &mut *(start as *mut SimpleElement<H>)
        }
    }
}

impl<H: Sized> Elements<HeaderElement<H>> {
    pub(crate) fn get_mut(&self, index: usize) -> &mut HeaderElement<H> {
        assert!(index < self.element_len);

        // Offset of the first item that belons to the element.
        let item_index = index * self.element_size;

        // Number of items which belong to the header
        assert!(size_of::<H>() % size_of::<RW<u32>>() == 0);
        let header_items_size = size_of::<H>() / size_of::<RW<u32>>();

        // Number of items which belong to the data, the DST length
        let dst_size = self.element_size - header_items_size;

        unsafe {
            let start = self.base.add(item_index);
            let slice = core::slice::from_raw_parts_mut(start as *mut (), dst_size);
            &mut *(slice as *mut [()] as *mut HeaderElement<H>)
        }
    }
}

struct ElementAllocator(usize);
impl ElementAllocator {
    fn new(offset: usize) -> Self {
        ElementAllocator(offset)
    }
    fn next(&mut self, element_size: usize, num_elements: usize) -> ElementSizing {
        let sizing = ElementSizing {
            offset: self.0,
            element_size,
            num_elements,
        };
        self.0 += sizing.total_size_words();
        sizing
    }
}

#[derive(Clone, Copy)]
struct ElementSizing {
    /// Base offset in words.
    offset: usize,
    /// Size of each element in words.
    element_size: usize,
    /// Number of elements.
    num_elements: usize,
}

impl ElementSizing {
    const NONE: Self = Self::new(0, 0, 0);
    const fn new(offset: usize, element_size: usize, num_elements: usize) -> Self {
        Self {
            offset,
            element_size,
            num_elements,
        }
    }
    fn total_size_words(&self) -> usize {
        self.element_size * self.num_elements
    }
    fn end_offset_words(&self) -> usize {
        self.offset + self.total_size_words()
    }
    unsafe fn make<E: ?Sized>(&self, base_ptr: *mut RW<u32>) -> Elements<E> {
        Elements::new(base_ptr.add(self.offset), self.num_elements, self.element_size)
    }
}

struct ElementsSizing {
    start_offset: usize,
    end_offset: usize,

    standard_id: ElementSizing,
    extended_id: ElementSizing,
    rx_fifo_0: ElementSizing,
    rx_fifo_1: ElementSizing,
    rx_buffer: ElementSizing,
    tx_event: ElementSizing,
    tx_elements: ElementSizing,
    /// Number of leading elements in `tx_elements` which are dedicated tx buffers.
    dedicated_len: usize,
    trigger: ElementSizing,
}

pub(crate) struct MessageRamSegment {
    /// Base offset of the Message RAM region allocated to this
    /// peripheral instance.
    /// In bytes.
    pub base_offset: usize,
    /// Available space allocated for this peripheral instance in bytes.
    /// If present, it will be validated that everything fits into the
    /// allocated space.
    /// In bytes.
    pub available_space: Option<usize>,
}

impl ElementsSizing {
    /// Calculates the sizing used for the simplified variant of the M_CAN peripheral.
    fn calculate_element_sizing_simplified() -> ElementsSizing {
        let mut a = ElementAllocator::new(0);

        ElementsSizing {
            start_offset: 0,
            standard_id: a.next(1, 28),
            extended_id: a.next(2, 8),
            rx_fifo_0: a.next(18, 3),
            rx_fifo_1: a.next(18, 3),
            rx_buffer: ElementSizing::NONE,
            tx_event: a.next(2, 3),
            tx_elements: a.next(18, 3),
            // Simplified variant supports no dedicated buffers
            dedicated_len: 0,
            trigger: ElementSizing::NONE,
            end_offset: a.0,
        }
    }

    /// Calculates the sizing used for the full variant of the M_CAN peripheral.
    /// This mapping is customizable, a configuration is needed to instantiate it.
    fn calculate_element_sizing(config: &MessageRamConfig, segment: &MessageRamSegment) -> ElementsSizing {
        assert!(
            config.standard_id_filter_size <= 128,
            "more than 128 standard id filters not supported"
        );
        assert!(
            config.extended_id_filter_size <= 64,
            "more than 64 extended id filters not supported"
        );
        assert!(
            config.rx_fifo_0.fifo_size <= 64,
            "more than 64 rx fifo 0 elements not supported"
        );
        assert!(
            config.rx_fifo_1.fifo_size <= 64,
            "more than 64 rx fifo 1 elements not supported"
        );
        assert!(
            config.rx_buffer.size <= 64,
            "more than 64 rx buffer elements not supported"
        );
        assert!(
            config.tx.dedicated_size + config.tx.queue_size <= 32,
            "total TX elements can not be larger than 32"
        );

        let base_offset = segment.base_offset;
        let base_offset_words = base_offset >> 2;
        let mut a = ElementAllocator::new(base_offset_words);

        ElementsSizing {
            start_offset: a.0,
            standard_id: a.next(1, config.standard_id_filter_size as usize),
            extended_id: a.next(2, config.extended_id_filter_size as usize),
            rx_fifo_0: a.next(
                2 + config.rx_fifo_0.data_field_size.word_size(),
                config.rx_fifo_0.fifo_size as usize,
            ),
            rx_fifo_1: a.next(
                2 + config.rx_fifo_1.data_field_size.word_size(),
                config.rx_fifo_1.fifo_size as usize,
            ),
            rx_buffer: a.next(
                2 + config.rx_buffer.data_field_size.word_size(),
                config.rx_buffer.size as usize,
            ),
            // Hard code size 16 for TX Event FIFO.
            // Closely coupled to the driver implementation, no reason to
            // allow for customization for now.
            tx_event: a.next(2, 16),
            tx_elements: a.next(
                2 + config.tx.data_field_size.word_size(),
                (config.tx.dedicated_size + config.tx.queue_size) as usize,
            ),
            dedicated_len: config.tx.dedicated_size as usize,
            // Driver does not support TTCAN for now, zero triggers.
            trigger: a.next(2, 0),
            end_offset: a.0,
        }
    }

    unsafe fn make(&self, ram_base: *mut RW<u32>) -> MessageRam {
        MessageRam {
            base_ptr: ram_base.add(self.start_offset),
            standard_filter: self.standard_id.make(ram_base),
            extended_filter: self.extended_id.make(ram_base),
            rx_fifos: [self.rx_fifo_0.make(ram_base), self.rx_fifo_1.make(ram_base)],
            #[cfg(can_fdcan_h7)]
            rx_buffer: self.rx_buffer.make(ram_base),
            tx_event_fifo: self.tx_event.make(ram_base),
            tx_elements: self.tx_elements.make(ram_base),
            #[cfg(can_fdcan_h7)]
            tx_buffer_len: self.dedicated_len,
            tx_queue_len: self.tx_elements.num_elements - self.dedicated_len,
            #[cfg(can_fdcan_h7)]
            trigger_memory: self.trigger.make(ram_base),
        }
    }
}

#[cfg(can_fdcan_h7)]
impl MessageRamConfig {
    // This constant is a H7 thing, not a limitation of M_CAN.
    const H7_MSG_RAM_SIZE: usize = 0x2800;

    /// Configures message ram for the peripheral according to the supplied
    /// config and returns a struct which can be used to interact with the
    /// message RAM.
    pub(crate) unsafe fn apply_config(
        &self,
        segment: &MessageRamSegment,
        regs: &crate::pac::can::Fdcan,
        ram: &crate::pac::fdcanram::Fdcanram,
    ) -> MessageRam {
        let sizing = ElementsSizing::calculate_element_sizing(self, segment);

        let total_size_words = sizing.end_offset - sizing.start_offset;
        let total_size_bytes = total_size_words << 2;

        if let Some(avail) = segment.available_space {
            assert!(
                total_size_bytes <= avail,
                "CAN RAM config exceeded available space! ({} allocated, {} available)",
                total_size_bytes,
                avail
            );
        }

        // Standard ID filter config
        // Fully managed
        regs.sidfc().modify(|v| {
            v.set_flssa(sizing.standard_id.offset as u16);
            v.set_lss(sizing.standard_id.num_elements as u8);
        });

        // Extended ID filter config
        // Fully managed
        regs.xidfc().modify(|v| {
            v.set_flesa(sizing.extended_id.offset as u16);
            v.set_lse(sizing.extended_id.num_elements as u8);
        });

        // RX FIFO 0 config
        regs.rxfc(0).modify(|v| {
            // F0OM - RX FIFO Operating Mode
            // F0WM - RX FIFO Water Mark
            v.set_fsa(sizing.rx_fifo_0.offset as u16);
            v.set_fs(sizing.rx_fifo_0.num_elements as u8);
        });

        // RX FIFO 1 config
        regs.rxfc(1).modify(|v| {
            // F1OM - RX FIFO Operating Mode
            // F1WM - RX FIFO Water Mark
            v.set_fsa(sizing.rx_fifo_1.offset as u16);
            v.set_fs(sizing.rx_fifo_1.num_elements as u8);
        });

        // RX buffer config
        // Fully managed
        regs.rxbc().modify(|v| {
            v.set_rbsa(sizing.rx_buffer.offset as u16);
        });

        // Rx buffer / queue element size config
        // Fully managed
        regs.rxesc().modify(|v| {
            v.set_rbds(self.rx_buffer.data_field_size.reg_value());
            v.set_fds(0, self.rx_fifo_0.data_field_size.reg_value());
            v.set_fds(1, self.rx_fifo_1.data_field_size.reg_value());
        });

        // TX event FIFO config
        regs.txefc().modify(|v| {
            // EFWM - Event FIFO Water Mark
            v.set_efsa(sizing.tx_event.offset as u16);
            v.set_efs(sizing.tx_event.num_elements as u8);
        });

        // Tx buffer / queue element size config
        // Fully managed
        regs.txesc().modify(|v| {
            v.set_tbds(self.tx.data_field_size.reg_value());
        });

        // TX queue configuration
        // Fully managed
        regs.txbc().modify(|v| {
            //v.set_tfqm(self.tx.queue_operation_mode.reg_value());
            v.set_tbsa(sizing.tx_elements.offset as u16);
            v.set_ndtb(sizing.dedicated_len as u8);
            v.set_tfqs((sizing.tx_elements.num_elements - sizing.dedicated_len) as u8);
        });

        // TT Trigger memory config
        // Fully managed
        regs.tttmc().modify(|v| {
            v.set_tmsa(sizing.trigger.offset as u16);
            v.set_tme(sizing.trigger.num_elements as u8);
        });

        let ram_ptr = ram.as_ptr() as *mut RW<u32>;

        assert!(unsafe {
            (ram_ptr.add(sizing.end_offset) as usize) < ((ram_ptr as *mut u8).add(Self::H7_MSG_RAM_SIZE) as usize)
        });

        unsafe { sizing.make(ram_ptr) }
    }
}
