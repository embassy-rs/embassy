//! Direct Memory Access (DMA)
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering, fence};
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::peripherals::DMA;

const U23_MAX: u32 = 0xff_ffff;

/// DMA interrupt handler binding.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Acking the interrupt clears both the ERROR and DONE flags.
        // In poll, we can check the BUSY flag to know if we are done,
        // but still need a way to check for bus error.
        //
        // So we cache the ERROR flag before clearing it.
        let err = T::info().reg.ctrl().read().dma_ctrl_error().bit_is_set();
        T::info().err_flag.store(err, Ordering::Release);
        T::info().reg.ctrl().modify(|_, w| w.dma_ctrl_ack().set_bit());

        T::info().waker.wake();
    }
}

/// DMA error.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The NEORV32 configuration does not support DMA.
    NotSupported,
    /// Indicates a bus error occurred during transfer.
    BusError,
}

enum DataConfig {
    ConstantByte,
    ConstantWord,
    IncrementingByte,
    IncrementingWord,
}

impl From<DataConfig> for u32 {
    fn from(config: DataConfig) -> Self {
        match config {
            DataConfig::ConstantByte => 0b00,
            DataConfig::ConstantWord => 0b01,
            DataConfig::IncrementingByte => 0b10,
            DataConfig::IncrementingWord => 0b11,
        }
    }
}

struct TransferConfig {
    num_elems: u32,
    swap_byte_order: bool,
    src_cfg: DataConfig,
    dst_cfg: DataConfig,
}

impl TransferConfig {
    fn new(num_elems: u32, swap_byte_order: bool, src_cfg: DataConfig, dst_cfg: DataConfig) -> Self {
        // Hardware only supports 23 bits for num elements
        assert!(num_elems > 0 && num_elems < U23_MAX);
        Self {
            num_elems,
            swap_byte_order,
            src_cfg,
            dst_cfg,
        }
    }
}

impl From<TransferConfig> for u32 {
    fn from(config: TransferConfig) -> Self {
        (u32::from(config.dst_cfg) << 30)
            | (u32::from(config.src_cfg) << 28)
            | ((config.swap_byte_order as u32) << 27)
            | (config.num_elems & U23_MAX)
    }
}

enum Descriptor {
    BaseAddress(u32),
    Config(TransferConfig),
}

impl From<Descriptor> for u32 {
    fn from(descriptor: Descriptor) -> Self {
        match descriptor {
            Descriptor::BaseAddress(addr) => addr,
            Descriptor::Config(cfg) => cfg.into(),
        }
    }
}

/// DMA driver.
///
/// DMA is single-channel only so so the entire peripheral may only have a single owner.
pub struct Dma<'d> {
    info: Info,
    _phantom: PhantomData<&'d ()>,
}

// Allows for use in a Mutex (to share safely between harts and tasks)
unsafe impl<'d> Send for Dma<'d> {}

impl<'d> Dma<'d> {
    fn enable(&mut self) {
        self.info.reg.ctrl().modify(|_, w| w.dma_ctrl_en().set_bit());
    }

    fn disable(&mut self) {
        self.info.reg.ctrl().modify(|_, w| w.dma_ctrl_en().clear_bit());
    }

    fn start(&mut self) {
        self.info.reg.ctrl().modify(|_, w| w.dma_ctrl_start().set_bit());
    }

    fn write_descriptor(&mut self, descriptor: Descriptor) {
        // SAFETY: We are writing a valid descriptor
        self.info.reg.desc().write(|w| unsafe { w.bits(descriptor.into()) });
    }

    fn busy(&self) -> bool {
        self.info.reg.ctrl().read().dma_ctrl_busy().bit_is_set()
    }

    fn abort(&mut self) {
        // Disable DMA and flush cache to ensure CPU sees most recent main memory
        self.disable();
        fence(Ordering::SeqCst);
    }

    /// Creates a new instance of a DMA driver.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if DMA is not supported.
    pub fn new<T: Instance>(
        _instance: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        if !crate::sysinfo::SysInfo::soc_config().has_dma() {
            return Err(Error::NotSupported);
        }

        // SAFETY: Enabling DMA interrupts at this point is valid
        unsafe { T::Interrupt::enable() }

        Ok(Self {
            info: T::info(),
            _phantom: PhantomData,
        })
    }

    /// Starts a transfer which reads from `src` until the `dst` buffer is filled.
    ///
    /// # Panics
    ///
    /// Panics if the `dst` buffer length can not be represented in 23 bits.
    pub fn read<'t, W: Word>(&'t mut self, src: &W, dst: &mut [W], swap_byte_order: bool) -> Transfer<'d, 't> {
        Transfer::new(
            self,
            src as *const W as *const u32,
            W::cfg_constant(),
            dst.as_mut_ptr() as *mut u32,
            W::cfg_increment(),
            dst.len() as u32,
            swap_byte_order,
        )
    }

    /// Starts a transfer which writes all elements from the `src` buffer to `dst`.
    ///
    /// # Panics
    ///
    /// Panics if the `src` buffer length can not be represented in 23 bits.
    pub fn write<'t, W: Word>(&'t mut self, src: &[W], dst: &mut W, swap_byte_order: bool) -> Transfer<'d, 't> {
        Transfer::new(
            self,
            src.as_ptr() as *const u32,
            W::cfg_increment(),
            dst as *mut W as *mut u32,
            W::cfg_constant(),
            src.len() as u32,
            swap_byte_order,
        )
    }

    /// Starts a transfer which copies all elements from the `src` buffer to the `dst` buffer.
    ///
    /// # Panics
    ///
    /// Panics if the `src` buffer length does not match the `dst` buffer length,
    /// or if the buffer length can not be represented in 23 bits.
    pub fn copy<'t, W: Word>(&'t mut self, src: &[W], dst: &mut [W], swap_byte_order: bool) -> Transfer<'d, 't> {
        assert!(src.len() == dst.len());
        Transfer::new(
            self,
            src.as_ptr() as *const u32,
            W::cfg_increment(),
            dst.as_mut_ptr() as *mut u32,
            W::cfg_increment(),
            src.len() as u32,
            swap_byte_order,
        )
    }
}

/// A DMA transfer.
///
/// The transfer should be awaited to ensure completion.
///
/// **Note**: The transfer will be aborted if cancelled/dropped before completion.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'d, 't> {
    // Note: We use 2 unique lifetimes here so that Transfer holds a mutable reference to Dma
    // (to prevent other transfers from being simultaneously created) for AT MOST as long as it lives.
    //
    // If we use a single lifetime, the borrow checker assumes the Transfer lives as long as Dma lives.
    //
    // Can think of it as 't represents Transfer lifetime and 'd represents Dma lifetime.
    dma: &'t mut Dma<'d>,
}

impl<'d, 't> Transfer<'d, 't> {
    fn new(
        dma: &'t mut Dma<'d>,
        src: *const u32,
        src_cfg: DataConfig,
        dst: *mut u32,
        dst_cfg: DataConfig,
        len: u32,
        swap_byte_order: bool,
    ) -> Self {
        // Clear error flag and enable DMA
        dma.info.err_flag.store(false, Ordering::Release);
        dma.enable();

        // Configure the transfer
        let config = TransferConfig::new(len, swap_byte_order, src_cfg, dst_cfg);
        let descriptors = [
            Descriptor::BaseAddress(src as u32),
            Descriptor::BaseAddress(dst as u32),
            Descriptor::Config(config),
        ];

        // Write each descriptor
        // We are assuming the descriptor FIFO is empty because this HAL does not allow partial transfers in the FIFO
        for descriptor in descriptors {
            dma.write_descriptor(descriptor);
        }

        // Flush cache to ensure DMA sees most recent main memory, then start transfer
        fence(Ordering::SeqCst);
        dma.start();
        Self { dma }
    }
}

impl<'d, 't> Drop for Transfer<'d, 't> {
    // When the transfer is completed, or otherwise dropped or cancelled, always get here
    // Regardless, we ensure the DMA is disabled (aborting the transfer if in progress) and flush cache
    fn drop(&mut self) {
        self.dma.abort();
    }
}

impl<'d, 't> Future for Transfer<'d, 't> {
    type Output = Result<(), Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.dma.info.waker.register(cx.waker());

        if self.dma.busy() {
            Poll::Pending
        } else if self.dma.info.err_flag.load(Ordering::Acquire) {
            Poll::Ready(Err(Error::BusError))
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

trait SealedWord {
    fn cfg_constant() -> DataConfig;
    fn cfg_increment() -> DataConfig;
}

/// A DMA transfer word.
///
/// **Note**: The hardware supports transferring `u8` to `u32` (by zero-extending the `u8`),
/// but does not seem to support transferring `u32` to `u8` (ideally it would truncate the 24 MSB).
///
/// So, for ease of use, the driver only supports `u8` <-> `u8` and `u32` <-> `u32` transfers.
#[allow(private_bounds)]
pub trait Word: SealedWord {}

impl SealedWord for u8 {
    #[inline(always)]
    fn cfg_constant() -> DataConfig {
        DataConfig::ConstantByte
    }

    #[inline(always)]
    fn cfg_increment() -> DataConfig {
        DataConfig::IncrementingByte
    }
}
impl Word for u8 {}

impl SealedWord for u32 {
    #[inline(always)]
    fn cfg_constant() -> DataConfig {
        DataConfig::ConstantWord
    }

    #[inline(always)]
    fn cfg_increment() -> DataConfig {
        DataConfig::IncrementingWord
    }
}
impl Word for u32 {}

struct Info {
    reg: &'static crate::pac::dma::RegisterBlock,
    waker: &'static AtomicWaker,
    err_flag: &'static AtomicBool,
}

trait SealedInstance {
    fn info() -> Info;
}

/// A valid DMA peripheral.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: Interrupt;
}

impl SealedInstance for DMA {
    fn info() -> Info {
        static WAKER: AtomicWaker = AtomicWaker::new();
        static ERR_FLAG: AtomicBool = AtomicBool::new(false);

        Info {
            // SAFETY: We are the sole users of the pointer and are sure to use it safely
            reg: unsafe { &*crate::pac::Dma::ptr() },
            waker: &WAKER,
            err_flag: &ERR_FLAG,
        }
    }
}
impl Instance for DMA {
    type Interrupt = crate::interrupt::typelevel::DMA;
}
