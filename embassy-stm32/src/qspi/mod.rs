//! Quad Serial Peripheral Interface (QSPI)

#![macro_use]

pub mod enums;

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use enums::*;

use crate::dma::ChannelAndRequest;
use crate::gpio::{AfType, AnyPin, OutputType, Pull, Speed};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::quadspi::Quadspi as Regs;
use crate::rcc::{self, RccPeripheral};
use crate::{Peri, interrupt};

/// QSPI transfer configuration.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TransferConfig {
    /// Instruction width (IMODE)
    pub iwidth: QspiWidth,
    /// Address width (ADMODE)
    pub awidth: QspiWidth,
    /// Data width (DMODE)
    pub dwidth: QspiWidth,
    /// Instruction Id
    pub instruction: u8,
    /// Flash memory address
    pub address: Option<u32>,
    /// Number of dummy cycles (DCYC)
    pub dummy: DummyCycles,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            iwidth: QspiWidth::NONE,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: 0,
            address: None,
            dummy: DummyCycles::_0,
        }
    }
}

/// QSPI driver configuration.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Flash memory size representend as 2^[0-32], as reasonable minimum 1KiB(9) was chosen.
    /// If you need other value the whose predefined use `Other` variant.
    pub memory_size: MemorySize,
    /// Address size (8/16/24/32-bit)
    pub address_size: AddressSize,
    /// Scalar factor for generating CLK [0-255]
    pub prescaler: u8,
    /// Number of bytes to trigger FIFO threshold flag.
    pub fifo_threshold: FIFOThresholdLevel,
    /// Minimum number of cycles that chip select must be high between issued commands
    pub cs_high_time: ChipSelectHighTime,
    /// Shift sampling point of input data (none, or half-cycle)
    pub sample_shifting: SampleShifting,
    /// GPIO Speed
    pub gpio_speed: Speed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory_size: MemorySize::Other(0),
            address_size: AddressSize::_24bit,
            prescaler: 128,
            fifo_threshold: FIFOThresholdLevel::_17Bytes,
            cs_high_time: ChipSelectHighTime::_5Cycle,
            sample_shifting: SampleShifting::None,
            gpio_speed: Speed::VeryHigh,
        }
    }
}

/// QSPI driver.
#[allow(dead_code)]
pub struct Qspi<'d, T: Instance, M: PeriMode> {
    _peri: Peri<'d, T>,
    sck: Option<Peri<'d, AnyPin>>,
    d0: Option<Peri<'d, AnyPin>>,
    d1: Option<Peri<'d, AnyPin>>,
    d2: Option<Peri<'d, AnyPin>>,
    d3: Option<Peri<'d, AnyPin>>,
    nss: Option<Peri<'d, AnyPin>>,
    dma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
    config: Config,
}

impl<'d, T: Instance, M: PeriMode> Qspi<'d, T, M> {
    fn new_inner(
        peri: Peri<'d, T>,
        d0: Option<Peri<'d, AnyPin>>,
        d1: Option<Peri<'d, AnyPin>>,
        d2: Option<Peri<'d, AnyPin>>,
        d3: Option<Peri<'d, AnyPin>>,
        sck: Option<Peri<'d, AnyPin>>,
        nss: Option<Peri<'d, AnyPin>>,
        dma: Option<ChannelAndRequest<'d>>,
        config: Config,
        fsel: FlashSelection,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<T>();

        while T::REGS.sr().read().busy() {}

        #[cfg(stm32h7)]
        {
            use stm32_metapac::quadspi::regs::Cr;
            // Apply precautionary steps according to the errata...
            T::REGS.cr().write_value(Cr(0));
            while T::REGS.sr().read().busy() {}
            T::REGS.cr().write_value(Cr(0xFF000001));
            T::REGS.ccr().write(|w| w.set_frcm(true));
            T::REGS.ccr().write(|w| w.set_frcm(true));
            T::REGS.cr().write_value(Cr(0));
            while T::REGS.sr().read().busy() {}
        }

        T::REGS.cr().modify(|w| {
            w.set_en(true);
            //w.set_tcen(false);
            w.set_sshift(config.sample_shifting.into());
            w.set_fthres(config.fifo_threshold.into());
            w.set_prescaler(config.prescaler);
            w.set_fsel(fsel.into());
        });
        T::REGS.dcr().modify(|w| {
            w.set_fsize(config.memory_size.into());
            w.set_csht(config.cs_high_time.into());
            w.set_ckmode(true);
        });

        Self {
            _peri: peri,
            sck,
            d0,
            d1,
            d2,
            d3,
            nss,
            dma,
            _phantom: PhantomData,
            config,
        }
    }

    /// Do a QSPI command.
    pub fn blocking_command(&mut self, transaction: TransferConfig) {
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));
        self.setup_transaction(QspiMode::IndirectWrite, &transaction, None);

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Blocking read data.
    pub fn blocking_read(&mut self, buf: &mut [u8], transaction: TransferConfig) {
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));
        self.setup_transaction(QspiMode::IndirectWrite, &transaction, Some(buf.len()));

        let current_ar = T::REGS.ar().read().address();
        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectRead.into());
        });
        T::REGS.ar().write(|v| {
            v.set_address(current_ar);
        });

        for b in buf {
            while !T::REGS.sr().read().tcf() && (T::REGS.sr().read().flevel() == 0) {}
            *b = unsafe { (T::REGS.dr().as_ptr() as *mut u8).read_volatile() };
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Blocking write data.
    pub fn blocking_write(&mut self, buf: &[u8], transaction: TransferConfig) {
        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));

        self.setup_transaction(QspiMode::IndirectWrite, &transaction, Some(buf.len()));

        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectWrite.into());
        });

        for &b in buf {
            while !T::REGS.sr().read().ftf() {}
            unsafe { (T::REGS.dr().as_ptr() as *mut u8).write_volatile(b) };
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Enable memory map mode
    pub fn enable_memory_map(&mut self, transaction: &TransferConfig) {
        T::REGS.fcr().modify(|v| {
            v.set_csmf(true);
            v.set_ctcf(true);
            v.set_ctef(true);
            v.set_ctof(true);
        });
        T::REGS.ccr().write(|v| {
            v.set_fmode(QspiMode::MemoryMapped.into());
            v.set_imode(transaction.iwidth.into());
            v.set_instruction(transaction.instruction);
            v.set_admode(transaction.awidth.into());
            v.set_adsize(self.config.address_size.into());
            v.set_dmode(transaction.dwidth.into());
            v.set_abmode(QspiWidth::NONE.into());
            v.set_dcyc(transaction.dummy.into());
        });
    }

    /// Automaticly poll until a desired status is received.
    pub fn blocking_auto_poll(
        &mut self,
        // The transaction to send
        transaction: TransferConfig,
        // Polling frequency, in clock cycles
        interval: u16,
        // Data mask, 0 = ignore bit, 1 = match bit
        mask: u32,
        // Value to match
        match_value: u32,
        // Number of bytes to receive, 1..=4
        data_len: usize,
        // Matching mode
        match_mode: MatchMode,
        // Timeout
        #[cfg(feature = "time")] timeout: embassy_time::Duration,
    ) -> Result<(), Error> {
        self.setup_auto_poll(transaction, interval, mask, match_value, data_len, match_mode);

        #[cfg(feature = "time")]
        let deadline = embassy_time::Instant::now() + timeout;

        while !T::REGS.sr().read().smf() {
            #[cfg(feature = "time")]
            if embassy_time::Instant::now() > deadline {
                return Err(Error::AutoPollTimeout);
            }
        }

        Ok(())
    }

    fn setup_auto_poll(
        &mut self,
        transaction: TransferConfig,
        interval: u16,
        mask: u32,
        match_value: u32,
        data_len: usize,
        match_mode: MatchMode,
    ) {
        assert!(data_len >= 1);
        assert!(data_len <= 4);

        while T::REGS.sr().read().busy() {}

        T::REGS.fcr().modify(|v| {
            v.set_csmf(true);
            v.set_ctcf(true);
            v.set_ctef(true);
            v.set_ctof(true);
        });

        T::REGS.cr().modify(|m| {
            // Set Match Mode
            m.set_pmm(match_mode.into());
            // Stop on match
            m.set_apms(true);
        });

        T::REGS.psmkr().write(|w| w.set_mask(mask));
        T::REGS.psmar().write(|w| w.set_match_(match_value));
        T::REGS.pir().write(|w| w.set_interval(interval));

        self.setup_transaction(QspiMode::AutoPolling, &transaction, Some(data_len));
    }

    fn setup_transaction(&mut self, fmode: QspiMode, transaction: &TransferConfig, data_len: Option<usize>) {
        match (transaction.address, transaction.awidth) {
            (Some(_), QspiWidth::NONE) => panic!("QSPI address can't be sent with an address width of NONE"),
            (Some(_), _) => {}
            (None, QspiWidth::NONE) => {}
            (None, _) => panic!("QSPI address is not set, so the address width should be NONE"),
        }

        match (data_len, transaction.dwidth) {
            (Some(0), _) => panic!("QSPI data must be at least one byte"),
            (Some(_), QspiWidth::NONE) => panic!("QSPI data can't be sent with a data width of NONE"),
            (Some(_), _) => {}
            (None, QspiWidth::NONE) => {}
            (None, _) => panic!("QSPI data is empty, so the data width should be NONE"),
        }

        T::REGS.fcr().modify(|v| {
            v.set_csmf(true);
            v.set_ctcf(true);
            v.set_ctef(true);
            v.set_ctof(true);
        });

        while T::REGS.sr().read().busy() {}

        if let Some(len) = data_len {
            T::REGS.dlr().write(|v| v.set_dl(len as u32 - 1));
        }

        T::REGS.ccr().write(|v| {
            v.set_fmode(fmode.into());
            v.set_imode(transaction.iwidth.into());
            v.set_instruction(transaction.instruction);
            v.set_admode(transaction.awidth.into());
            v.set_adsize(self.config.address_size.into());
            v.set_dmode(transaction.dwidth.into());
            v.set_abmode(QspiWidth::NONE.into());
            v.set_dcyc(transaction.dummy.into());
        });

        if let Some(addr) = transaction.address {
            T::REGS.ar().write(|v| {
                v.set_address(addr);
            });
        }
    }
}

impl<'d, T: Instance> Qspi<'d, T, Blocking> {
    /// Create a new QSPI driver for bank 1, in blocking mode.
    pub fn new_blocking_bank1(
        peri: Peri<'d, T>,
        d0: Peri<'d, impl BK1D0Pin<T>>,
        d1: Peri<'d, impl BK1D1Pin<T>>,
        d2: Peri<'d, impl BK1D2Pin<T>>,
        d3: Peri<'d, impl BK1D3Pin<T>>,
        sck: Peri<'d, impl SckPin<T>>,
        nss: Peri<'d, impl BK1NSSPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d1, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d2, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d3, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(sck, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(
                nss,
                AfType::output_pull(OutputType::PushPull, config.gpio_speed, Pull::Up)
            ),
            None,
            config,
            FlashSelection::Flash1,
        )
    }

    /// Create a new QSPI driver for bank 2, in blocking mode.
    pub fn new_blocking_bank2(
        peri: Peri<'d, T>,
        d0: Peri<'d, impl BK2D0Pin<T>>,
        d1: Peri<'d, impl BK2D1Pin<T>>,
        d2: Peri<'d, impl BK2D2Pin<T>>,
        d3: Peri<'d, impl BK2D3Pin<T>>,
        sck: Peri<'d, impl SckPin<T>>,
        nss: Peri<'d, impl BK2NSSPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d1, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d2, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d3, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(sck, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(
                nss,
                AfType::output_pull(OutputType::PushPull, config.gpio_speed, Pull::Up)
            ),
            None,
            config,
            FlashSelection::Flash2,
        )
    }
}

impl<'d, T: Instance> Qspi<'d, T, Async> {
    /// Create a new QSPI driver for bank 1.
    pub fn new_bank1<D, I>(
        peri: Peri<'d, T>,
        d0: Peri<'d, impl BK1D0Pin<T>>,
        d1: Peri<'d, impl BK1D1Pin<T>>,
        d2: Peri<'d, impl BK1D2Pin<T>>,
        d3: Peri<'d, impl BK1D3Pin<T>>,
        sck: Peri<'d, impl SckPin<T>>,
        nss: Peri<'d, impl BK1NSSPin<T>>,
        dma: Peri<'d, D>,
        _irq: I,
        config: Config,
    ) -> Self
    where
        D: QuadDma<T>,
        I: Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d1, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d2, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d3, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(sck, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(
                nss,
                AfType::output_pull(OutputType::PushPull, config.gpio_speed, Pull::Up)
            ),
            new_dma!(dma, _irq),
            config,
            FlashSelection::Flash1,
        )
    }

    /// Create a new QSPI driver for bank 2.
    pub fn new_bank2<D, I>(
        peri: Peri<'d, T>,
        d0: Peri<'d, impl BK2D0Pin<T>>,
        d1: Peri<'d, impl BK2D1Pin<T>>,
        d2: Peri<'d, impl BK2D2Pin<T>>,
        d3: Peri<'d, impl BK2D3Pin<T>>,
        sck: Peri<'d, impl SckPin<T>>,
        nss: Peri<'d, impl BK2NSSPin<T>>,
        dma: Peri<'d, D>,
        _irq: I,
        config: Config,
    ) -> Self
    where
        D: QuadDma<T>,
        I: Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d1, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d2, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(d3, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(sck, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(
                nss,
                AfType::output_pull(OutputType::PushPull, config.gpio_speed, Pull::Up)
            ),
            new_dma!(dma, _irq),
            config,
            FlashSelection::Flash2,
        )
    }

    /// Blocking read data, using DMA.
    pub fn blocking_read_dma(&mut self, buf: &mut [u8], transaction: TransferConfig) {
        let transfer = self.start_read_transfer(transaction, buf);
        transfer.blocking_wait();
    }

    /// Async read data, using DMA.
    pub async fn read_dma(&mut self, buf: &mut [u8], transaction: TransferConfig) {
        let _scoped_wake_guard = T::RCC_INFO.wake_guard();
        let transfer = self.start_read_transfer(transaction, buf);
        transfer.await;
    }

    fn start_read_transfer<'a>(
        &'a mut self,
        transaction: TransferConfig,
        buf: &'a mut [u8],
    ) -> crate::dma::Transfer<'a> {
        self.setup_transaction(QspiMode::IndirectWrite, &transaction, Some(buf.len()));

        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectRead.into());
        });
        let current_ar = T::REGS.ar().read().address();
        T::REGS.ar().write(|v| {
            v.set_address(current_ar);
        });

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .read(T::REGS.dr().as_ptr() as *mut u8, buf, Default::default())
        };

        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(true));
        transfer
    }

    /// Blocking write data, using DMA.
    pub fn blocking_write_dma(&mut self, buf: &[u8], transaction: TransferConfig) {
        let transfer = self.start_write_transfer(transaction, buf);
        transfer.blocking_wait();
    }

    /// Async write data, using DMA.
    pub async fn write_dma(&mut self, buf: &[u8], transaction: TransferConfig) {
        let _scoped_wake_guard = T::RCC_INFO.wake_guard();
        let transfer = self.start_write_transfer(transaction, buf);
        transfer.await;
    }

    fn start_write_transfer<'a>(&'a mut self, transaction: TransferConfig, buf: &'a [u8]) -> crate::dma::Transfer<'a> {
        self.setup_transaction(QspiMode::IndirectWrite, &transaction, Some(buf.len()));

        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectWrite.into());
        });

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .write(buf, T::REGS.dr().as_ptr() as *mut u8, Default::default())
        };

        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(true));
        transfer
    }

    /// Automaticly poll until a desired status is received.
    /// In case the desired status is never received, it is advised to always use `WithTimeout::with_timeout()`.
    pub async fn auto_poll(
        &mut self,
        // The transaction to send
        transaction: TransferConfig,
        // Polling frequency, in clock cycles
        interval: u16,
        // Data mask, 0 = ignore bit, 1 = match bit
        mask: u32,
        // Value to match
        match_value: u32,
        // Number of bytes to receive, 1..=4
        data_len: usize,
        // Matching mode
        match_mode: MatchMode,
    ) {
        T::REGS.cr().modify(|m| {
            // Set Status Match Interrupt Enable
            m.set_smie(true);
        });

        self.setup_auto_poll(transaction, interval, mask, match_value, data_len, match_mode);

        AutoPollFuture {
            _peri: self._peri.reborrow(),
        }
        .await
    }
}

/// QSPI error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Timed Out waiting for Status MAtch
    AutoPollTimeout,
}

trait SealedInstance {
    const REGS: Regs;
}

/// QSPI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(SckPin, Instance);
pin_trait!(BK1D0Pin, Instance);
pin_trait!(BK1D1Pin, Instance);
pin_trait!(BK1D2Pin, Instance);
pin_trait!(BK1D3Pin, Instance);
pin_trait!(BK1NSSPin, Instance);

pin_trait!(BK2D0Pin, Instance);
pin_trait!(BK2D1Pin, Instance);
pin_trait!(BK2D2Pin, Instance);
pin_trait!(BK2D3Pin, Instance);
pin_trait!(BK2NSSPin, Instance);

dma_trait!(QuadDma, Instance);

macro_rules! impl_peripheral {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

foreach_interrupt! {
    ($inst:ident, quadspi, $block:ident, GLOBAL, $irq:ident) => {
        impl_peripheral!($inst, $irq);
    };
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct AutoPollFuture<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Unpin for AutoPollFuture<'d, T> {}
impl<'d, T: Instance> Drop for AutoPollFuture<'d, T> {
    fn drop(&mut self) {
        T::REGS.cr().modify(|m| {
            // Unset Status Match Interrupt Enable
            m.set_smie(false);
        });

        if T::REGS.ccr().read().fmode() == QspiMode::AutoPolling.into() && T::REGS.sr().read().busy() {
            // Abort autopolling if dropped while still running
            T::REGS.cr().modify(|m| m.set_abort(true));
            while T::REGS.sr().read().busy() {}
        }
    }
}

impl<'d, T: Instance> Future for AutoPollFuture<'d, T> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        AUTOPOLL_WAKER.register(cx.waker());

        if T::REGS.sr().read().busy() {
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(())
        }
    }
}

static AUTOPOLL_WAKER: AtomicWaker = AtomicWaker::new();

/// AutoPolling Match Mode
pub enum MatchMode {
    /// Match any masked bit
    OR,
    /// Match all masked bits
    AND,
}

impl From<MatchMode> for bool {
    fn from(mode: MatchMode) -> Self {
        match mode {
            MatchMode::OR => true,
            MatchMode::AND => false,
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::REGS.sr().read().smf() {
            // clear status match flag
            T::REGS.fcr().modify(|m| m.set_csmf(true));
            AUTOPOLL_WAKER.wake();
        }
    }
}
