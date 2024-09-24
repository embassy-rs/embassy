use core::marker::PhantomData;

use element::tx_event::TxEventElement;
use stm32_metapac::can::vals::Tfqm;
use volatile_register::RW;

mod element;
pub(crate) use element::enums::{DataLength, Event, FilterElementConfig, FilterType, FrameFormat, IdType};
pub(crate) use element::{
    filter_extended::ExtendedFilter, filter_standard::StandardFilter, rx_buffer::RxFifoElementHeader,
    tx_buffer::TxBufferElementHeader,
};

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

    pub fn debug_print(&self, regs: &crate::pac::can::Fdcan) {
        defmt::info!("msgram seg base: {}", self.base_ptr);
        defmt::info!("msgram tx base: {}", self.tx_elements.base);
        //defmt::info!(
        //    "msgram tx calc offet: {}",
        //    ((self.tx_elements.base as usize) - (self.base_ptr as usize)) >> 2
        //);
        defmt::info!("msgram tx periph offset: {}", regs.txbc().read().tbsa());
        let ram_ptr = crate::pac::FDCANRAM.as_ptr();
        defmt::info!("msgram tx periph ptr: {}", unsafe {
            ram_ptr.byte_add(regs.txbc().read().tbsa() as usize * 4)
        });
        //defmt::info!("elem 0 start {}", self.tx_elements.get_mut(0) as *mut _);
        //defmt::info!("elem 1 start {}", self.tx_elements.get_mut(1) as *mut _);
        //defmt::info!("elem 0 data start {}", (&self.tx_elements.get_mut(0).data).as_ptr());
        //defmt::info!("elem 1 data start {}", (&self.tx_elements.get_mut(1).data).as_ptr());
    }
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

impl MessageRam {}

#[derive(Debug, Clone, Copy)]
pub enum RxFifoOperationMode {
    Blocking = 0,
    Overwrite = 1,
}

/// For RX:
/// Excess data is IGNORED, only the number of bytes which fit
/// into the element are stored.
///
/// For TX:
/// If DLC is higher than the data field size, excess bytes are
/// transmitted as 0xCC (padding bytes).
#[derive(Debug, Clone, Copy)]
pub enum DataFieldSize {
    B8 = 0b000,
    B12 = 0b001,
    B16 = 0b010,
    B20 = 0b011,
    B24 = 0b100,
    B32 = 0b101,
    B48 = 0b110,
    B64 = 0b111,
}

impl DataFieldSize {
    fn reg_value(self) -> u8 {
        self as u8
    }

    fn byte_size(self) -> usize {
        match self {
            DataFieldSize::B8 => 8,
            DataFieldSize::B12 => 12,
            DataFieldSize::B16 => 16,
            DataFieldSize::B20 => 20,
            DataFieldSize::B24 => 24,
            DataFieldSize::B32 => 32,
            DataFieldSize::B48 => 48,
            DataFieldSize::B64 => 64,
        }
    }

    fn word_size(self) -> usize {
        self.byte_size() / 4
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RxFifoConfig {
    pub(crate) operation_mode: RxFifoOperationMode,
    /// 0: Disabled
    /// 1-64: Watermark interrupt level
    /// >64: Disabled
    pub(crate) watermark_interrupt_level: u8,
    /// 0: RX FIFO disabled
    /// 1-64: Number of RX FIFO elements
    /// >64: Interpreted as 64
    pub(crate) fifo_size: u8,
    pub(crate) data_field_size: DataFieldSize,
}

impl RxFifoConfig {
    pub const DISABLED: Self = RxFifoConfig {
        operation_mode: RxFifoOperationMode::Blocking,
        watermark_interrupt_level: 0,
        fifo_size: 0,
        data_field_size: DataFieldSize::B8,
    };
}

pub struct RxBufferConfig {
    /// 0-64
    pub(crate) size: u8,
    pub(crate) data_field_size: DataFieldSize,
}

impl RxBufferConfig {
    pub const DISABLED: Self = RxBufferConfig {
        size: 0,
        data_field_size: DataFieldSize::B8,
    };
}

#[derive(Debug, Clone, Copy)]
pub enum TxQueueOperationMode {
    /// Operates as a strict FIFO.
    /// First element entered into the FIFO is always the first element sent.
    FIFO = 0,
    /// Operates as a priority queue.
    /// Element with highest priority (lowest ID) is always the first element sent.
    Priority = 1,
}

impl TxQueueOperationMode {
    fn reg_value(self) -> Tfqm {
        match self {
            TxQueueOperationMode::FIFO => Tfqm::FIFO,
            TxQueueOperationMode::Priority => Tfqm::QUEUE,
        }
    }
}

#[cfg(can_fdcan_h7)]
pub struct TxConfig {
    pub(crate) queue_operation_mode: TxQueueOperationMode,
    /// Number of elements reserved for TX Queue.
    /// NOTE: queue_size + dedicated_size may not be greater than 32.
    ///
    /// 0: No TX FIFO/Priority queue
    /// 1-32: Number of TX buffers used for TX FIFO/Priority queue
    /// >32: Interpreted as 32
    pub(crate) queue_size: u8,
    /// Number of elements reserved for Dedicated TX buffers.
    /// NOTE: queue_size + dedicated_size may not be greater than 32.
    ///
    /// 0: No TX dedicated buffers
    /// 1-32: Number of TX buffers used for TX dedicated buffers
    /// >32: Interpreted as 32
    pub(crate) dedicated_size: u8,
    pub(crate) data_field_size: DataFieldSize,
}

#[cfg(can_fdcan_h7)]
pub struct MessageRamConfig {
    /// Base offset of the Message RAM region allocated to this
    /// peripheral instance.
    /// In bytes.
    pub(crate) base_offset: usize,
    /// Available space allocated for this peripheral instance in bytes.
    /// If present, it will be validated that everything fits into the
    /// allocated space.
    /// In bytes.
    pub(crate) available_space: Option<usize>,

    /// 0: No standard Message ID filter
    /// 1-128: Number of standard Message ID filter elements
    /// >128: Interpreted as 128
    pub(crate) standard_id_filter_size: u8,
    /// 0: No extended Message ID filter
    /// 1-64: Number of extended Message ID filter elements
    /// >64: Interpreted as 64
    pub(crate) extended_id_filter_size: u8,

    pub(crate) rx_fifo_0: RxFifoConfig,
    pub(crate) rx_fifo_1: RxFifoConfig,
    pub(crate) rx_buffer: RxBufferConfig,
    pub(crate) tx: TxConfig,
}

#[cfg(can_fdcan_h7)]
const MSG_RAM_SIZE: usize = 0x2800;

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

impl ElementsSizing {
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

    fn calculate_element_sizing(config: &MessageRamConfig) -> ElementsSizing {
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

        let base_offset = config.base_offset;
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
            // TODO implement TX events
            tx_event: a.next(2, 0),
            tx_elements: a.next(
                2 + config.tx.data_field_size.word_size(),
                (config.tx.dedicated_size + config.tx.queue_size) as usize,
            ),
            dedicated_len: config.tx.dedicated_size as usize,
            // TODO implement triggers
            trigger: a.next(2, 0),
            end_offset: a.0,
        }
    }
}

#[cfg(can_fdcan_h7)]
impl MessageRamConfig {
    /// Configures message ram for the peripheral according to the supplied
    /// config and returns a struct which can be used to interact with the
    /// message RAM.
    #[cfg(can_fdcan_h7)]
    pub fn apply_config(&self, regs: &crate::pac::can::Fdcan, ram: &crate::pac::fdcanram::Fdcanram) -> MessageRam {
        let base_offset = self.base_offset;
        let base_offset_words = base_offset >> 2;

        // Abbreviations:
        // sa: start address
        // es: element size
        // en: element num
        // ts: total size

        assert!(
            self.standard_id_filter_size <= 128,
            "more than 128 standard id filters not supported"
        );
        let sid_sa = base_offset_words;
        let sid_es = 1;
        let sid_en = self.standard_id_filter_size as usize;
        let sid_ts = sid_es * sid_en;

        assert!(
            self.extended_id_filter_size <= 64,
            "more than 64 extended id filters not supported"
        );
        let xid_sa = sid_sa + sid_ts;
        let xid_es = 2;
        let xid_en = self.extended_id_filter_size as usize;
        let xid_ts = xid_es * xid_en;

        assert!(
            self.rx_fifo_0.fifo_size <= 64,
            "more than 64 rx fifo 0 elements not supported"
        );
        let rx0_sa = xid_sa + xid_ts;
        let rx0_es = 2 + self.rx_fifo_0.data_field_size.word_size();
        let rx0_en = self.rx_fifo_0.fifo_size as usize;
        let rx0_ts = rx0_es * rx0_en;

        assert!(
            self.rx_fifo_1.fifo_size <= 64,
            "more than 64 rx fifo 1 elements not supported"
        );
        let rx1_sa = rx0_sa + rx0_ts;
        let rx1_es = 2 + self.rx_fifo_1.data_field_size.word_size();
        let rx1_en = self.rx_fifo_1.fifo_size as usize;
        let rx1_ts = rx1_es * rx1_en;

        assert!(
            self.rx_buffer.size <= 64,
            "more than 64 rx buffer elements not supported"
        );
        let rxb_sa = rx1_sa + rx1_ts;
        let rxb_es = 2 + self.rx_buffer.data_field_size.word_size();
        let rxb_en = self.rx_buffer.size as usize;
        let rxb_ts = rxb_es * rxb_en;

        let txe_sa = rxb_sa + rxb_ts;
        let txe_es = 2;
        let txe_en = 0; // TODO implement TX events
        let txe_ts = txe_es * txe_en;

        assert!(
            self.tx.dedicated_size + self.tx.queue_size <= 32,
            "total TX elements can not be larger than 32"
        );
        let txx_es = 2 + self.tx.data_field_size.word_size();

        let txq_sa = txe_sa + txe_ts;
        let txq_en = self.tx.queue_size as usize;
        let txq_ts = txx_es * txq_en;

        let txd_sa = txq_sa + txq_ts;
        let txd_en = self.tx.dedicated_size as usize;
        let txd_ts = txx_es * txd_en;

        let tmc_sa = txd_sa + txd_ts;
        let tmc_es = 2;
        let tmc_en = 0; // TODO implement trigger stuff
        let tmc_ts = tmc_es * tmc_en;

        let end_offset_words = tmc_sa + tmc_ts;
        let total_size_words = end_offset_words - base_offset_words;
        let total_size_bytes = total_size_words << 2;
        //defmt::info!("total calculated word size: {:#x}", total_size_words);

        if let Some(avail) = self.available_space {
            assert!(total_size_bytes <= avail, "CAN RAM config exceeded available space!");
        }

        // Standard ID filter config
        // Fully managed
        regs.sidfc().modify(|v| {
            v.set_flssa(sid_sa as u16);
            v.set_lss(sid_en as u8);
        });

        // Extended ID filter config
        // Fully managed
        regs.xidfc().modify(|v| {
            v.set_flesa(xid_sa as u16);
            v.set_lse(xid_en as u8);
        });

        // RX FIFO 0 config
        regs.rxfc(0).modify(|v| {
            // F0OM - RX FIFO Operating Mode
            // F0WM - RX FIFO Water Mark
            v.set_fsa(rx0_sa as u16);
            v.set_fs(rx0_en as u8);
        });

        // RX FIFO 1 config
        regs.rxfc(1).modify(|v| {
            // F1OM - RX FIFO Operating Mode
            // F1WM - RX FIFO Water Mark
            v.set_fsa(rx1_sa as u16);
            v.set_fs(rx1_en as u8);
        });

        // RX buffer config
        // Fully managed
        regs.rxbc().modify(|v| {
            v.set_rbsa(rxb_sa as u16);
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
            v.set_efsa(txe_sa as u16);
            v.set_efs(txe_en as u8);
        });

        // Tx buffer / queue element size config
        // Fully managed
        regs.txesc().modify(|v| {
            v.set_tbds(self.tx.data_field_size.reg_value());
        });

        // TX queue configuration
        // Fully managed
        regs.txbc().modify(|v| {
            v.set_tfqm(self.tx.queue_operation_mode.reg_value());
            v.set_tbsa(txq_sa as u16);
            v.set_ndtb(txd_en as u8);
            v.set_tfqs(txq_en as u8);
        });

        // TT Trigger memory config
        // Fully managed
        regs.tttmc().modify(|v| {
            v.set_tmsa(tmc_sa as u16);
            v.set_tme(tmc_en as u8);
        });

        let ram_ptr = ram.as_ptr() as *mut RW<u32>;
        let base_ptr = unsafe { ram_ptr.add(base_offset_words) };

        // let mut test_ptr = base_ptr as *const u32;
        // while test_ptr < unsafe { base_ptr.byte_add(self.available_space.unwrap()) as *const u32 } {
        //     let val = unsafe { test_ptr.read() };
        //     if val == 0xaa95fd71 || val == 0x71fd95aa {
        //         defmt::info!("FOUND BYTE MATCH: {}", test_ptr);
        //     }
        //     test_ptr = unsafe { test_ptr.byte_add(4) };
        // }

        // if let Some(avail) = self.available_space {
        //     unsafe { core::ptr::write_bytes(ram_ptr as *mut u8, 0, avail) };
        // }

        // assert!(unsafe { base_ptr as usize == (ram_ptr as *mut u8).add(self.base_offset) as usize });

        assert!(unsafe {
            (ram_ptr.add(end_offset_words) as usize) < ((ram_ptr as *mut u8).add(MSG_RAM_SIZE) as usize)
        });

        unsafe {
            MessageRam {
                base_ptr,
                standard_filter: Elements::new(ram_ptr.add(sid_sa), sid_en, sid_es),
                extended_filter: Elements::new(ram_ptr.add(xid_sa), xid_en, xid_es),
                rx_fifos: [
                    Elements::new(ram_ptr.add(rx0_sa), rx0_en, rx0_es),
                    Elements::new(ram_ptr.add(rx1_sa), rx1_en, rx1_es),
                ],
                rx_buffer: Elements::new(ram_ptr.add(rxb_sa), rxb_en, rxb_es),
                tx_event_fifo: Elements::new(ram_ptr.add(txe_sa), txe_en, txe_es),
                tx_elements: Elements::new(ram_ptr.add(txq_sa), txd_en + txq_en, txx_es),
                tx_buffer_len: txd_en,
                tx_queue_len: txq_en,
                trigger_memory: Elements::new(ram_ptr.add(tmc_sa), tmc_en, tmc_es),
            }
        }
    }
}
