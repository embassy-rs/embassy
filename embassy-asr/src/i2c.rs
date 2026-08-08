//! I2C0–I2C2 driver for the ASR6601 (Marvell-style TWSI).
//!
//! Register programming follows the vendor `tremo_i2c` driver and the I2C
//! examples under `projects/*/examples/i2c`. Transfers use the non-FIFO
//! byte path (`DBR` + `CR.TRANS_BYTE`), which is the SDK default.
//!
//! Alternate-function numbers come from datasheet Table 4-3 (Fun=3 for every
//! documented I2C remap). Constructors either take typed pins that encode that
//! AF, or accept an explicit [`AlternateFunction`] per pin.
//!
//! # Unsafe contracts
//!
//! - [`InterruptHandler::on_interrupt`] may be called only from the vector bound
//!   through [`crate::bind_interrupts!`] for the matching I2C instance.
//! - Constructing a driver steals the PAC register block for that I2C. The
//!   owned [`Peri`] token is the exclusivity proof; do not also use
//!   `pac::I2c*::steal()` from application code while the driver is alive.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::interrupt::Priority;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal::i2c::Operation as EhOperation;

use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::i2c0::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::{Peri, PeripheralType, interrupt, peripherals};

const SR_ARB_LOSS: u32 = 1 << 18;
const SR_IDBR_EMPTY: u32 = 1 << 19;
const SR_DBR_FULL: u32 = 1 << 20;
const SR_BUS_ERROR: u32 = 1 << 22;
const SR_SLAVE_ADDR_DET: u32 = 1 << 23;
const SR_SLAVE_STOP_DET: u32 = 1 << 24;
const SR_ERROR_MASK: u32 = SR_ARB_LOSS | SR_BUS_ERROR;

const CR_INTR_MASK: u32 = (1 << 18) // arb loss
    | (1 << 19) // idbr empty
    | (1 << 20) // dbr full
    | (1 << 22) // bus error
    | (1 << 23) // slave addr
    | (1 << 24) // slave stop
    | (1 << 25) // master stop intr en
    | (1 << 27); // trans done

/// I2C transfer errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// ACK was not received for an address or data byte.
    Nack,
    /// Arbitration was lost.
    Arbitration,
    /// Protocol / bus error flag.
    Bus,
    /// Timed out waiting for the unit to leave reset / busy.
    Timeout,
    /// 10-bit addressing is not supported by the vendor programming model.
    AddressMode,
    /// RCC clocks have not been published by [`crate::init`].
    ClocksNotInitialized,
    /// Enabling or resetting the I2C clock failed.
    Rcc(rcc::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nack => f.write_str("I2C NACK"),
            Self::Arbitration => f.write_str("I2C arbitration lost"),
            Self::Bus => f.write_str("I2C bus error"),
            Self::Timeout => f.write_str("I2C timeout"),
            Self::AddressMode => f.write_str("I2C 10-bit addressing unsupported"),
            Self::ClocksNotInitialized => f.write_str("RCC clocks are not initialized"),
            Self::Rcc(_) => f.write_str("I2C RCC error"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Self::Nack => embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Unknown),
            Self::Arbitration => embedded_hal::i2c::ErrorKind::ArbitrationLoss,
            Self::Bus => embedded_hal::i2c::ErrorKind::Bus,
            _ => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

/// Bus speed programmed through `CR.BUS_MODE` and the LCR/WCR dividers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frequency {
    /// Standard mode, 100 kHz.
    Standard,
    /// Fast mode, 400 kHz.
    Fast,
}

impl Frequency {
    const fn hertz(self) -> u32 {
        match self {
            Self::Standard => 100_000,
            Self::Fast => 400_000,
        }
    }
}

/// I2C master configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target bus frequency.
    pub frequency: Frequency,
    /// Optional internal pull on SCL.
    ///
    /// External pull-ups are recommended; leave this as [`Pull::None`] when
    /// board pull-ups are present.
    pub scl_pull: Pull,
    /// Optional internal pull on SDA.
    pub sda_pull: Pull,
}

impl Config {
    /// Standard-mode master with no internal pulls.
    pub const fn new() -> Self {
        Self {
            frequency: Frequency::Standard,
            scl_pull: Pull::None,
            sda_pull: Pull::None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction observed after a slave address match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlaveOp {
    /// Master is writing; the slave should receive.
    Read,
    /// Master is reading; the slave should transmit.
    Write,
}

#[derive(Clone, Copy)]
enum PclkSel {
    Pclk0,
    Pclk1,
}

struct State {
    waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

static STATE_I2C0: State = State::new();
static STATE_I2C1: State = State::new();
static STATE_I2C2: State = State::new();

struct Info {
    regs: *const RegisterBlock,
    peripheral: Peripheral,
    pclk: PclkSel,
    state: &'static State,
}

// Register blocks are Sync; the pointer is unique to the owned I2C instance.
unsafe impl Sync for Info {}

impl Info {
    fn regs(&self) -> &'static RegisterBlock {
        unsafe { &*self.regs }
    }
}

/// Interrupt handler for one I2C instance.
///
/// Masks the sticky event enables that woke the vector and wakes any async
/// waiter. Status bits are left for the waiter to observe and clear.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();
        let sr = regs.sr().read().bits();

        // Mask level/event enables; the waiter re-enables as needed.
        let cr = regs.cr().read().bits() & !CR_INTR_MASK;
        regs.cr().write_with_zero(|w| unsafe { w.bits(cr) });

        // Clear only the error flags in the ISR so a cancelled waiter cannot
        // leave sticky bus errors latched forever. Completion flags stay for
        // the waiter.
        let errors = sr & SR_ERROR_MASK;
        if errors != 0 {
            regs.sr().write_with_zero(|w| unsafe { w.bits(errors) });
        }

        info.state.waker.wake();
    }
}

/// I2C master driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    _scl: Flex<'d>,
    _sda: Flex<'d>,
    _mode: PhantomData<&'d mut M>,
}

/// I2C slave driver.
///
/// Matches the vendor slave example: wait for address detect, then exchange
/// bytes with ACK/NAK control until stop.
pub struct I2cSlave<'d, M: Mode> {
    info: &'static Info,
    _scl: Flex<'d>,
    _sda: Flex<'d>,
    _mode: PhantomData<&'d mut M>,
}

fn configure_i2c_pin<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction, pull: Pull) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    // The TWSI pad is open-drain when muxed; direction follows the vendor
    // examples which only program the IOMUX.
    flex.set_as_input();
    flex.set_pull(pull);
    flex.set_alternate_function(af);
    flex
}

fn pclk_hz(sel: PclkSel) -> Result<u32, Error> {
    let clocks = rcc::clocks().ok_or(Error::ClocksNotInitialized)?;
    Ok(match sel {
        PclkSel::Pclk0 => clocks.pclk0_hz,
        PclkSel::Pclk1 => clocks.pclk1_hz,
    })
}

fn unit_reset(regs: &RegisterBlock) -> Result<(), Error> {
    let mut timeout = 1_000u32;
    while regs.sr().read().unit_busy().bit_is_set() {
        timeout = timeout.saturating_sub(1);
        if timeout == 0 {
            return Err(Error::Timeout);
        }
    }

    // Vendor: CR = UNIT_RESET only, pulse reset, clear SR, clear reset bit.
    unsafe {
        regs.cr().write_with_zero(|w| w.unit_reset().set_bit());
        let sr = regs.sr().read().bits();
        regs.sr().write_with_zero(|w| w.bits(sr));
        regs.cr().write_with_zero(|w| w.bits(0));
    }
    Ok(())
}

fn program_timing(regs: &RegisterBlock, pclk: u32, frequency: Frequency) {
    let slv = (pclk / Frequency::Standard.hertz()).saturating_sub(8) / 2;
    let flv = (pclk / Frequency::Fast.hertz()).saturating_sub(8) / 2;
    let flv = flv.saturating_sub(1);
    let lcr = (slv & 0x1ff) | ((flv & 0x1ff) << 9);
    let wcr = flv / 3;

    unsafe {
        regs.lcr().write_with_zero(|w| w.bits(lcr));
        regs.wcr().write_with_zero(|w| w.bits(wcr));
    }

    regs.cr().modify(|_, w| match frequency {
        Frequency::Standard => w.bus_mode().standard(),
        Frequency::Fast => w.bus_mode().fast(),
    });
}

fn enable_unit(regs: &RegisterBlock, enable: bool) {
    regs.cr().modify(|_, w| {
        w.twsi_unit_en().bit(enable);
        w.scl_en().bit(enable)
    });
}

fn clear_sr(regs: &RegisterBlock, mask: u32) {
    unsafe {
        regs.sr().write_with_zero(|w| w.bits(mask));
    }
}

fn check_errors(regs: &RegisterBlock) -> Result<(), Error> {
    let sr = regs.sr().read();
    if sr.arb_loss_det().bit_is_set() {
        clear_sr(regs, SR_ARB_LOSS);
        return Err(Error::Arbitration);
    }
    if sr.bus_error().bit_is_set() {
        clear_sr(regs, SR_BUS_ERROR);
        return Err(Error::Bus);
    }
    Ok(())
}

fn nack_seen(regs: &RegisterBlock) -> bool {
    // ACK_STATUS = 1 means NAK was received for the previous byte.
    regs.sr().read().ack_status().bit_is_set()
}

fn master_abort(regs: &RegisterBlock) {
    regs.cr().modify(|_, w| {
        w.start().clear_bit();
        w.master_abort().set_bit()
    });
}

fn master_send_start(regs: &RegisterBlock, addr7: u8, read: bool) {
    let data = (addr7 << 1) | u8::from(read);
    regs.cr().modify(|_, w| w.master_abort().clear_bit());
    unsafe {
        regs.dbr().write_with_zero(|w| w.bits(u32::from(data)));
    }
    regs.cr().modify(|_, w| {
        w.stop().clear_bit();
        w.start().set_bit();
        w.trans_byte().set_bit()
    });
}

fn master_send_byte(regs: &RegisterBlock, data: u8) {
    unsafe {
        regs.dbr().write_with_zero(|w| w.bits(u32::from(data)));
    }
    regs.cr().modify(|_, w| {
        w.start().clear_bit();
        w.trans_byte().set_bit()
    });
}

fn set_receive_mode(regs: &RegisterBlock, ack: bool) {
    // CR.ACKNAK = 1 requests NAK.
    regs.cr().modify(|_, w| {
        w.acknak().bit(!ack);
        w.start().clear_bit();
        w.trans_byte().set_bit()
    });
}

fn read_dbr(regs: &RegisterBlock) -> u8 {
    regs.dbr().read().bits() as u8
}

fn enable_clock(info: &'static Info) -> Result<(), Error> {
    rcc::enable_peripheral(info.peripheral).map_err(Error::Rcc)?;
    rcc::reset_peripheral(info.peripheral).map_err(Error::Rcc)?;
    Ok(())
}

fn init_master(info: &'static Info, config: Config) -> Result<(), Error> {
    let regs = info.regs();
    let pclk = pclk_hz(info.pclk)?;
    unit_reset(regs)?;
    // FIFO stays disabled (SDK default).
    regs.cr().modify(|_, w| w.fifo_en().clear_bit());
    program_timing(regs, pclk, config.frequency);
    enable_unit(regs, true);
    Ok(())
}

fn init_slave(info: &'static Info, address: u8) -> Result<(), Error> {
    if address > 0x7f {
        return Err(Error::AddressMode);
    }
    let regs = info.regs();
    let pclk = pclk_hz(info.pclk)?;
    unit_reset(regs)?;
    regs.cr().modify(|_, w| w.fifo_en().clear_bit());
    // Dividers are still required by the hardware; use standard timing.
    program_timing(regs, pclk, Frequency::Standard);
    unsafe {
        regs.sar().write_with_zero(|w| w.bits(u32::from(address)));
    }
    enable_unit(regs, true);
    Ok(())
}

fn shutdown(info: &'static Info) {
    let regs = info.regs();
    enable_unit(regs, false);
    let _ = rcc::disable_peripheral(info.peripheral);
}

fn set_irq_enables(regs: &RegisterBlock, mask: u32, enable: bool) {
    regs.cr().modify(|r, w| {
        let bits = if enable { r.bits() | mask } else { r.bits() & !mask };
        unsafe { w.bits(bits) }
    });
}

impl<'d> I2c<'d, Blocking> {
    /// Create a blocking master using typed pins (datasheet Fun=3 remaps).
    pub fn new_blocking<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_blocking_af(peri, scl, Scl::AF, sda, Sda::AF, config)
    }

    /// Create a blocking master with explicit alternate-function numbers.
    pub fn new_blocking_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, config, false)
    }
}

impl<'d> I2c<'d, Async> {
    /// Create an async master using typed pins.
    ///
    /// `irq` proves the I2C vector is bound to [`InterruptHandler<T>`].
    pub fn new<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_af(peri, scl, Scl::AF, sda, Sda::AF, irq, config)
    }

    /// Create an async master with explicit alternate-function numbers.
    pub fn new_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, config, true)
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        config: Config,
        enable_irq: bool,
    ) -> Result<Self, Error> {
        let info = T::info();
        enable_clock(info)?;

        let scl = configure_i2c_pin(scl, scl_af, config.scl_pull);
        let sda = configure_i2c_pin(sda, sda_af, config.sda_pull);

        init_master(info, config)?;

        if enable_irq {
            T::Interrupt::unpend();
            T::Interrupt::set_priority(Priority::P3);
            unsafe {
                T::Interrupt::enable();
            }
        }

        Ok(Self {
            info,
            _scl: scl,
            _sda: sda,
            _mode: PhantomData,
        })
    }

    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    fn finish_error(&self, err: Error) -> Error {
        master_abort(self.regs());
        err
    }

    fn blocking_wait_tx_empty(&self) -> Result<(), Error> {
        let regs = self.regs();
        loop {
            check_errors(regs)?;
            if regs.sr().read().idbr_empty().bit_is_set() {
                clear_sr(regs, SR_IDBR_EMPTY);
                if nack_seen(regs) {
                    return Err(self.finish_error(Error::Nack));
                }
                return Ok(());
            }
        }
    }

    fn blocking_wait_rx_full(&self) -> Result<(), Error> {
        let regs = self.regs();
        loop {
            check_errors(regs)?;
            if regs.sr().read().dbr_full().bit_is_set() {
                clear_sr(regs, SR_DBR_FULL);
                return Ok(());
            }
        }
    }

    fn blocking_write_ops(&mut self, addr: u8, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        let regs = self.regs();
        master_send_start(regs, addr, false);
        clear_sr(regs, SR_IDBR_EMPTY);
        self.blocking_wait_tx_empty()?;

        for &byte in bytes {
            master_send_byte(regs, byte);
            clear_sr(regs, SR_IDBR_EMPTY);
            self.blocking_wait_tx_empty()?;
        }

        if send_stop {
            master_abort(regs);
        }
        Ok(())
    }

    fn blocking_read_ops(&mut self, addr: u8, buffer: &mut [u8], restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            if send_stop {
                master_abort(self.regs());
            }
            return Ok(());
        }

        let regs = self.regs();
        if !restart {
            // Fresh transaction: address phase.
        }
        master_send_start(regs, addr, true);
        clear_sr(regs, SR_IDBR_EMPTY);
        self.blocking_wait_tx_empty()?;

        let last = buffer.len() - 1;
        for (i, slot) in buffer.iter_mut().enumerate() {
            let ack = i != last;
            set_receive_mode(regs, ack);
            self.blocking_wait_rx_full()?;
            *slot = read_dbr(regs);
        }

        if send_stop {
            master_abort(regs);
        }
        Ok(())
    }

    /// Blocking write of `bytes` to `address` (7-bit).
    pub fn blocking_write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_write_ops(address, bytes, true)
    }

    /// Blocking read of `buffer.len()` bytes from `address` (7-bit).
    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_read_ops(address, buffer, false, true)
    }

    /// Blocking write then repeated-start read.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_write_ops(address, write, false)?;
        self.blocking_read_ops(address, read, true, true)
    }

    /// Blocking embedded-hal transaction.
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let last = operations.len() - 1;
        for (i, op) in operations.iter_mut().enumerate() {
            let stop = i == last;
            match op {
                EhOperation::Write(bytes) => self.blocking_write_ops(address, bytes, stop)?,
                EhOperation::Read(buffer) => {
                    let restart = i != 0;
                    self.blocking_read_ops(address, buffer, restart, stop)?;
                }
            }
        }
        Ok(())
    }
}

impl<'d> I2c<'d, Async> {
    async fn wait_on<F>(&mut self, mut ready: F, irq_mask: u32) -> Result<(), Error>
    where
        F: FnMut(&RegisterBlock) -> Poll<Result<(), Error>>,
    {
        let regs = self.regs();
        let state = self.info.state;

        poll_fn(|cx| {
            state.waker.register(cx.waker());
            match ready(regs) {
                Poll::Ready(result) => {
                    set_irq_enables(regs, irq_mask, false);
                    Poll::Ready(result)
                }
                Poll::Pending => {
                    set_irq_enables(regs, irq_mask | SR_ERROR_MASK, true);
                    // Re-check after arming to close the race.
                    match ready(regs) {
                        Poll::Ready(result) => {
                            set_irq_enables(regs, irq_mask | SR_ERROR_MASK, false);
                            Poll::Ready(result)
                        }
                        Poll::Pending => Poll::Pending,
                    }
                }
            }
        })
        .await
    }

    async fn wait_tx_empty(&mut self) -> Result<(), Error> {
        self.wait_on(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().idbr_empty().bit_is_set() {
                    clear_sr(regs, SR_IDBR_EMPTY);
                    if nack_seen(regs) {
                        Poll::Ready(Err(Error::Nack))
                    } else {
                        Poll::Ready(Ok(()))
                    }
                } else {
                    Poll::Pending
                }
            },
            SR_IDBR_EMPTY,
        )
        .await
        .map_err(|e| self.finish_error(e))
    }

    async fn wait_rx_full(&mut self) -> Result<(), Error> {
        self.wait_on(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().dbr_full().bit_is_set() {
                    clear_sr(regs, SR_DBR_FULL);
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            SR_DBR_FULL,
        )
        .await
        .map_err(|e| self.finish_error(e))
    }

    async fn write_ops(&mut self, addr: u8, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        let regs = self.regs();
        master_send_start(regs, addr, false);
        clear_sr(regs, SR_IDBR_EMPTY);
        self.wait_tx_empty().await?;

        for &byte in bytes {
            master_send_byte(self.regs(), byte);
            clear_sr(self.regs(), SR_IDBR_EMPTY);
            self.wait_tx_empty().await?;
        }

        if send_stop {
            master_abort(self.regs());
        }
        Ok(())
    }

    async fn read_ops(&mut self, addr: u8, buffer: &mut [u8], _restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            if send_stop {
                master_abort(self.regs());
            }
            return Ok(());
        }

        let regs = self.regs();
        master_send_start(regs, addr, true);
        clear_sr(regs, SR_IDBR_EMPTY);
        self.wait_tx_empty().await?;

        let last = buffer.len() - 1;
        for (i, slot) in buffer.iter_mut().enumerate() {
            let ack = i != last;
            set_receive_mode(self.regs(), ack);
            self.wait_rx_full().await?;
            *slot = read_dbr(self.regs());
        }

        if send_stop {
            master_abort(self.regs());
        }
        Ok(())
    }

    /// Async write of `bytes` to `address` (7-bit).
    pub async fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.write_ops(address, bytes, true).await
    }

    /// Async read of `buffer.len()` bytes from `address` (7-bit).
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.read_ops(address, buffer, false, true).await
    }

    /// Async write then repeated-start read.
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.write_ops(address, write, false).await?;
        self.read_ops(address, read, true, true).await
    }

    /// Async embedded-hal transaction.
    pub async fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let last = operations.len() - 1;
        for (i, op) in operations.iter_mut().enumerate() {
            let stop = i == last;
            match op {
                EhOperation::Write(bytes) => self.write_ops(address, bytes, stop).await?,
                EhOperation::Read(buffer) => {
                    let restart = i != 0;
                    self.read_ops(address, buffer, restart, stop).await?;
                }
            }
        }
        Ok(())
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        shutdown(self.info);
    }
}

impl<'d> I2cSlave<'d, Blocking> {
    /// Create a blocking slave using typed pins.
    pub fn new_blocking<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
    ) -> Result<Self, Error> {
        Self::new_blocking_af(peri, scl, Scl::AF, sda, Sda::AF, address, scl_pull, sda_pull)
    }

    /// Create a blocking slave with explicit alternate-function numbers.
    pub fn new_blocking_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, address, scl_pull, sda_pull, false)
    }
}

impl<'d> I2cSlave<'d, Async> {
    /// Create an async slave using typed pins.
    pub fn new<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_af(peri, scl, Scl::AF, sda, Sda::AF, address, scl_pull, sda_pull, irq)
    }

    /// Create an async slave with explicit alternate-function numbers.
    pub fn new_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, address, scl_pull, sda_pull, true)
    }
}

impl<'d, M: Mode> I2cSlave<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
        enable_irq: bool,
    ) -> Result<Self, Error> {
        let info = T::info();
        enable_clock(info)?;

        let scl = configure_i2c_pin(scl, scl_af, scl_pull);
        let sda = configure_i2c_pin(sda, sda_af, sda_pull);

        init_slave(info, address)?;

        if enable_irq {
            T::Interrupt::unpend();
            T::Interrupt::set_priority(Priority::P3);
            unsafe {
                T::Interrupt::enable();
            }
        }

        Ok(Self {
            info,
            _scl: scl,
            _sda: sda,
            _mode: PhantomData,
        })
    }

    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Block until the slave address is detected; return the transfer direction.
    pub fn blocking_listen(&mut self) -> Result<SlaveOp, Error> {
        let regs = self.regs();
        loop {
            check_errors(regs)?;
            if regs.sr().read().slave_addr_det().bit_is_set() {
                clear_sr(regs, SR_SLAVE_ADDR_DET);
                // RW_MODE mirrors the R/W bit: 0 = master write / slave read.
                let op = if regs.sr().read().rw_mode().bit_is_set() {
                    SlaveOp::Write
                } else {
                    SlaveOp::Read
                };
                return Ok(op);
            }
        }
    }

    /// Receive `buffer.len()` bytes as slave (master write).
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let regs = self.regs();
        for slot in buffer.iter_mut() {
            set_receive_mode(regs, true);
            loop {
                check_errors(regs)?;
                if regs.sr().read().dbr_full().bit_is_set() {
                    clear_sr(regs, SR_DBR_FULL);
                    break;
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_sr(regs, SR_SLAVE_STOP_DET);
                    return Ok(());
                }
            }
            *slot = read_dbr(regs);
        }
        Ok(())
    }

    /// Transmit `bytes` as slave (master read).
    pub fn blocking_write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
        for &byte in bytes {
            master_send_byte(regs, byte);
            clear_sr(regs, SR_IDBR_EMPTY);
            loop {
                check_errors(regs)?;
                if regs.sr().read().idbr_empty().bit_is_set() {
                    clear_sr(regs, SR_IDBR_EMPTY);
                    break;
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_sr(regs, SR_SLAVE_STOP_DET);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Wait for a STOP condition after a slave transfer.
    pub fn blocking_wait_stop(&mut self) -> Result<(), Error> {
        let regs = self.regs();
        set_receive_mode(regs, true);
        loop {
            check_errors(regs)?;
            if regs.sr().read().slave_stop_det().bit_is_set() {
                clear_sr(regs, SR_SLAVE_STOP_DET);
                return Ok(());
            }
        }
    }
}

impl<'d> I2cSlave<'d, Async> {
    async fn wait_mask<F>(&mut self, mut ready: F, irq_mask: u32) -> Result<(), Error>
    where
        F: FnMut(&RegisterBlock) -> Poll<Result<(), Error>>,
    {
        let regs = self.regs();
        let state = self.info.state;

        poll_fn(|cx| {
            state.waker.register(cx.waker());
            match ready(regs) {
                Poll::Ready(result) => {
                    set_irq_enables(regs, irq_mask, false);
                    Poll::Ready(result)
                }
                Poll::Pending => {
                    set_irq_enables(regs, irq_mask | SR_ERROR_MASK, true);
                    match ready(regs) {
                        Poll::Ready(result) => {
                            set_irq_enables(regs, irq_mask | SR_ERROR_MASK, false);
                            Poll::Ready(result)
                        }
                        Poll::Pending => Poll::Pending,
                    }
                }
            }
        })
        .await
    }

    /// Wait until the slave address is detected; return the transfer direction.
    pub async fn listen(&mut self) -> Result<SlaveOp, Error> {
        let mut op = SlaveOp::Read;
        self.wait_mask(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().slave_addr_det().bit_is_set() {
                    clear_sr(regs, SR_SLAVE_ADDR_DET);
                    op = if regs.sr().read().rw_mode().bit_is_set() {
                        SlaveOp::Write
                    } else {
                        SlaveOp::Read
                    };
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            SR_SLAVE_ADDR_DET,
        )
        .await?;
        Ok(op)
    }

    /// Receive `buffer.len()` bytes as slave.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            set_receive_mode(self.regs(), true);
            let mut done = false;
            self.wait_mask(
                |regs| {
                    if let Err(e) = check_errors(regs) {
                        return Poll::Ready(Err(e));
                    }
                    if regs.sr().read().dbr_full().bit_is_set() {
                        clear_sr(regs, SR_DBR_FULL);
                        done = true;
                        Poll::Ready(Ok(()))
                    } else if regs.sr().read().slave_stop_det().bit_is_set() {
                        clear_sr(regs, SR_SLAVE_STOP_DET);
                        done = false;
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                },
                SR_DBR_FULL | SR_SLAVE_STOP_DET,
            )
            .await?;
            if !done {
                return Ok(());
            }
            *slot = read_dbr(self.regs());
        }
        Ok(())
    }

    /// Transmit `bytes` as slave.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for &byte in bytes {
            master_send_byte(self.regs(), byte);
            clear_sr(self.regs(), SR_IDBR_EMPTY);
            let mut early_stop = false;
            self.wait_mask(
                |regs| {
                    if let Err(e) = check_errors(regs) {
                        return Poll::Ready(Err(e));
                    }
                    if regs.sr().read().idbr_empty().bit_is_set() {
                        clear_sr(regs, SR_IDBR_EMPTY);
                        Poll::Ready(Ok(()))
                    } else if regs.sr().read().slave_stop_det().bit_is_set() {
                        clear_sr(regs, SR_SLAVE_STOP_DET);
                        early_stop = true;
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                },
                SR_IDBR_EMPTY | SR_SLAVE_STOP_DET,
            )
            .await?;
            if early_stop {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Wait for a STOP condition after a slave transfer.
    pub async fn wait_stop(&mut self) -> Result<(), Error> {
        set_receive_mode(self.regs(), true);
        self.wait_mask(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_sr(regs, SR_SLAVE_STOP_DET);
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            SR_SLAVE_STOP_DET,
        )
        .await
    }
}

impl<'d, M: Mode> Drop for I2cSlave<'d, M> {
    fn drop(&mut self) {
        shutdown(self.info);
    }
}

// --- embedded-hal ----------------------------------------------------------

impl<'d, M: Mode> embedded_hal::i2c::ErrorType for I2c<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for I2c<'d, M> {
    fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Self::Error> {
        self.blocking_transaction(address, operations)
    }
}

impl<'d> embedded_hal_async::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for I2c<'d, Async> {
    async fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Self::Error> {
        I2c::transaction(self, address, operations).await
    }
}

// --- Instance / pins -------------------------------------------------------

mod sealed {
    use super::Info;

    pub trait Instance {
        fn info() -> &'static Info;
    }
}

/// I2C instance (I2C0–I2C2).
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for this I2C.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_i2c {
    ($peri:ident, $interrupt:ident, $base:expr, $rcc:ident, $pclk:ident, $state:ident) => {
        impl sealed::Instance for peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: $base as *const RegisterBlock,
                    peripheral: Peripheral::$rcc,
                    pclk: PclkSel::$pclk,
                    state: &$state,
                };
                &INFO
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_i2c!(I2C0, I2C0, 0x4000_7000, I2c0, Pclk0, STATE_I2C0);
impl_i2c!(I2C1, I2C1, 0x4001_4000, I2c1, Pclk1, STATE_I2C1);
impl_i2c!(I2C2, I2C2, 0x4001_5000, I2c2, Pclk1, STATE_I2C2);

/// SCL pin for an I2C instance.
pub trait SclPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

/// SDA pin for an I2C instance.
pub trait SdaPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $trait:ident, $af:ident) => {
        impl $trait<peripherals::$instance> for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
    };
}

// Datasheet Table 4-3: every documented I2C remap uses Fun=3.
impl_pin!(PA2, I2C0, SclPin, Function3);
impl_pin!(PA3, I2C0, SdaPin, Function3);
impl_pin!(PA14, I2C0, SclPin, Function3);
impl_pin!(PA15, I2C0, SdaPin, Function3);
impl_pin!(PB10, I2C0, SclPin, Function3);
impl_pin!(PB11, I2C0, SdaPin, Function3);

impl_pin!(PA6, I2C1, SclPin, Function3);
impl_pin!(PA7, I2C1, SdaPin, Function3);
impl_pin!(PB14, I2C1, SclPin, Function3);
impl_pin!(PB15, I2C1, SdaPin, Function3);
impl_pin!(PC10, I2C1, SclPin, Function3);

impl_pin!(PA10, I2C2, SclPin, Function3);
impl_pin!(PA11, I2C2, SdaPin, Function3);
impl_pin!(PB7, I2C2, SdaPin, Function3);
impl_pin!(PC2, I2C2, SclPin, Function3);
impl_pin!(PC3, I2C2, SdaPin, Function3);
impl_pin!(PC15, I2C2, SdaPin, Function3);
