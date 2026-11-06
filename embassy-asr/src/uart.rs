//! UART0–UART3 driver for the ASR6601.
//!
//! Register programming follows the vendor `tremo_uart` driver (PL011-style
//! UART). Pin mux numbers are supplied explicitly by the caller; this crate
//! does not invent a pinmux matrix.
//!
//! # Unsafe contracts
//!
//! - [`InterruptHandler::on_interrupt`] may be called only from the vector bound
//!   through [`crate::bind_interrupts!`] for the matching UART instance.
//! - DMA helpers require the supplied channel to remain exclusively owned for
//!   the transfer, and the buffer to stay valid and DMA-accessible until the
//!   future (or blocking call) returns.
//! - Constructing a driver steals the PAC register block for that UART. The
//!   owned [`Peri`] token is the exclusivity proof; do not also use
//!   `pac::Uart*::steal()` from application code while the driver is alive.

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::interrupt::Priority;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Channel as DmaChannel, Request, TransferOptions};
use crate::gpio::{AlternateFunction, AnyPin, Flex, Pin};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::uart0::RegisterBlock;
use crate::rcc::{self, Peripheral, RCO4M_HZ, XO24M_HZ, XO32K_HZ};
use crate::{interrupt, pac, peripherals};

/// Capacity of the per-UART software RX ring buffer.
///
/// The hardware RX FIFO is only 16 bytes deep, so in interrupt-driven (async)
/// mode the ISR drains the FIFO into this much larger ring buffer. This
/// decouples capture (which happens at line rate in the ISR) from the reader
/// (which may be briefly busy), preventing FIFO overruns and byte loss.
const RX_BUF_CAP: usize = 512;

/// Diagnostic counters for RX byte loss. `RX_HW_OVERRUN` counts hardware FIFO
/// overrun errors (DR.OE), i.e. bytes dropped because the ISR did not drain the
/// 16-byte FIFO in time. `RX_RING_DROP` counts the software ring buffer dropping
/// its newest byte because the reader was more than [`RX_BUF_CAP`] bytes behind.
static RX_HW_OVERRUN: AtomicU32 = AtomicU32::new(0);
static RX_RING_DROP: AtomicU32 = AtomicU32::new(0);

// Software encoding of latched RX errors in `State.rx_error`, not register bits.
pub use crate::serial::{DataBits, FlowControl, StopBits};

const RX_ERR_OVERRUN: u8 = 1 << 0;
const RX_ERR_BREAK: u8 = 1 << 1;
const RX_ERR_PARITY: u8 = 1 << 2;
const RX_ERR_FRAMING: u8 = 1 << 3;

/// Number of hardware FIFO overrun errors seen since boot (all UARTs).
pub fn rx_hw_overrun_count() -> u32 {
    RX_HW_OVERRUN.load(Ordering::Relaxed)
}

/// Number of software ring-buffer drop-newest events since boot (all UARTs).
pub fn rx_ring_drop_count() -> u32 {
    RX_RING_DROP.load(Ordering::Relaxed)
}

static STATE: [State; 4] = [const { State::new() }; 4];

struct State {
    tx_waker: AtomicWaker,
    rx_waker: AtomicWaker,
    /// Number of live [`UartTx`]/ [`UartRx`] halves.
    refcount: AtomicU8,
    /// Baud rate used for break timing.
    baudrate: AtomicU32,
    /// RX error flags latched by the ISR and consumed by the reader.
    rx_error: AtomicU8,
    /// RX ring buffer write index (advanced by the ISR).
    rx_head: AtomicU16,
    /// RX ring buffer read index (advanced only by the reader).
    rx_tail: AtomicU16,
    /// RX ring buffer storage.
    rx_buf: UnsafeCell<[u8; RX_BUF_CAP]>,
}

impl State {
    const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            rx_waker: AtomicWaker::new(),
            refcount: AtomicU8::new(0),
            baudrate: AtomicU32::new(0),
            rx_error: AtomicU8::new(0),
            rx_head: AtomicU16::new(0),
            rx_tail: AtomicU16::new(0),
            rx_buf: UnsafeCell::new([0; RX_BUF_CAP]),
        }
    }

    /// Reset the RX ring buffer and error latch to empty.
    fn reset_rx(&self) {
        self.rx_head.store(0, Ordering::Release);
        self.rx_tail.store(0, Ordering::Release);
        self.rx_error.store(0, Ordering::Release);
    }
}

// Safety: `STATE` is a shared static read from both thread and interrupt
// context. The atomics (`tx_waker`, `rx_waker`, `refcount`, `baudrate`,
// `rx_error`, `rx_head`, `rx_tail`) are `Sync`. The `rx_buf` `UnsafeCell` is
// only ever accessed through the atomic indices: the ISR writes only at
// `rx_head` and advances it, the reader reads at `rx_tail` and advances it.
// The producer never moves `rx_tail` (the ring drops the newest byte when
// full), so producer and reader never touch the same slot concurrently.
unsafe impl Sync for State {}

struct Info {
    index: usize,
    base: usize,
    rcc: Peripheral,
    /// `true` when the default UART clock is PCLK0; otherwise PCLK1.
    pclk0: bool,
    dma_tx: Request,
    dma_rx: Request,
    state: &'static State,
}

impl Info {
    fn regs(&self) -> &'static RegisterBlock {
        unsafe { &*(self.base as *const RegisterBlock) }
    }
}

/// UART interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();
        let mis = regs.mis().read();

        if mis.bits() == 0 {
            return;
        }

        if mis.txmis().bit_is_set() {
            // Disable TX interrupt; the TX task re-enables when needed.
            regs.imsc().modify(|_, w| w.txim().clear_bit());
            info.state.tx_waker.wake();
        }

        if mis.rxmis().bit_is_set()
            || mis.rtmis().bit_is_set()
            || mis.femis().bit_is_set()
            || mis.pemis().bit_is_set()
            || mis.bemis().bit_is_set()
            || mis.oemis().bit_is_set()
        {
            // Drain the hardware RX FIFO into the software ring buffer.
            //
            // The RX interrupt is deliberately *not* masked: it stays armed so
            // capture continues at line rate. Once the FIFO is drained below the
            // trigger level the interrupt stops, so there is no storm. This
            // prevents the 16-byte hardware FIFO from overflowing (and dropping
            // bytes) while the reader is momentarily busy.
            loop {
                if regs.fr().read().rxfe().bit_is_set() {
                    break;
                }
                let dr = regs.dr().read();
                let mut error = 0;
                if dr.oe().bit_is_set() {
                    RX_HW_OVERRUN.fetch_add(1, Ordering::Relaxed);
                    error |= RX_ERR_OVERRUN;
                }
                if dr.be().bit_is_set() {
                    error |= RX_ERR_BREAK;
                }
                if dr.pe().bit_is_set() {
                    error |= RX_ERR_PARITY;
                }
                if dr.fe().bit_is_set() {
                    error |= RX_ERR_FRAMING;
                }
                if error != 0 {
                    // Writing RSC_ECR clears sticky receive-status bits.
                    unsafe {
                        regs.rsr_ecr().write_with_zero(|w| w.bits(0));
                    }
                    info.state.rx_error.fetch_or(error, Ordering::Release);
                    info.state.rx_waker.wake();
                    continue;
                }
                let b = dr.data().bits();

                let head = info.state.rx_head.load(Ordering::Acquire) as usize;
                let next = (head + 1) % RX_BUF_CAP;
                let tail = info.state.rx_tail.load(Ordering::Acquire) as usize;
                if next == tail {
                    // Ring buffer full (reader is far behind). Keep the bytes
                    // already queued and drop this newest one. The producer
                    // must not advance `tail`: only the reader owns it.
                    RX_RING_DROP.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                unsafe {
                    let buf = info.state.rx_buf.get();
                    (*buf)[head] = b;
                }
                info.state.rx_head.store(next as u16, Ordering::Release);
            }
            info.state.rx_waker.wake();
        }

        // ICR is write-1-to-clear: clear exactly the sources latched at
        // entry (rxic/txic/rtic/feic/peic/beic/oeic), so an event that fires
        // mid-handler is not dropped. The raw snapshot write preserves that.
        unsafe {
            regs.icr().write_with_zero(|w| w.bits(mis.bits()));
        }
    }
}

/// Parity selection.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// No parity bit.
    None,
    /// Odd parity.
    Odd,
    /// Even parity.
    Even,
}

/// UART configuration.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate in bits per second.
    pub baudrate: u32,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Parity.
    pub parity: Parity,
    /// Number of stop bits.
    pub stop_bits: StopBits,
    /// Hardware flow control.
    pub flow_control: FlowControl,
    /// Enable the TX/RX FIFOs (`LCR_H.FEN`).
    ///
    /// The vendor SDK defaults this to disabled; enabling it is recommended for
    /// interrupt-driven and DMA transfers.
    pub fifo: bool,
}

impl Config {
    /// Create a 115200 8N1 configuration with FIFOs enabled.
    pub const fn new() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: DataBits::DataBits8,
            parity: Parity::None,
            stop_bits: StopBits::STOP1,
            flow_control: FlowControl::None,
            fifo: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Baud rate cannot be represented for the active UART clock.
    Baudrate,
    /// RCC clocks have not been published by [`crate::init`].
    ClocksNotInitialized,
    /// Neither an RX nor a TX pin was supplied.
    NoRxOrTx,
    /// Flow-control pins required by [`Config::flow_control`] were missing.
    MissingFlowControlPin,
    /// Enabling or resetting the UART clock failed.
    Rcc(rcc::Error),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Baudrate => write!(f, "UART invalid baud rate"),
            Self::ClocksNotInitialized => write!(f, "UART clocks not initialized"),
            Self::NoRxOrTx => write!(f, "UART no RX or TX pin"),
            Self::MissingFlowControlPin => write!(f, "UART missing flow control pin"),
            Self::Rcc(err) => write!(f, "UART RCC error: {err}"),
        }
    }
}

impl core::error::Error for ConfigError {}

/// UART transfer / framing error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// RX FIFO overrun.
    Overrun,
    /// Framing error.
    Framing,
    /// Parity error.
    Parity,
    /// Break condition.
    Break,
    /// DMA helper failed.
    Dma(dma::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => write!(f, "UART overrun"),
            Self::Framing => write!(f, "UART framing error"),
            Self::Parity => write!(f, "UART parity error"),
            Self::Break => write!(f, "UART break"),
            Self::Dma(_) => write!(f, "UART DMA error"),
        }
    }
}

impl core::error::Error for Error {}

fn take_rx_error(state: &State) -> Option<Error> {
    let error = state.rx_error.swap(0, Ordering::AcqRel);
    if error & RX_ERR_OVERRUN != 0 {
        Some(Error::Overrun)
    } else if error & RX_ERR_BREAK != 0 {
        Some(Error::Break)
    } else if error & RX_ERR_PARITY != 0 {
        Some(Error::Parity)
    } else if error & RX_ERR_FRAMING != 0 {
        Some(Error::Framing)
    } else {
        None
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Self::Overrun => embedded_io::ErrorKind::OutOfMemory,
            Self::Framing | Self::Parity | Self::Break => embedded_io::ErrorKind::InvalidData,
            Self::Dma(_) => embedded_io::ErrorKind::Other,
        }
    }
}

fn decode_dr_error(dr: &pac::uart0::dr::R) -> Result<u8, Error> {
    if dr.oe().bit_is_set() {
        return Err(Error::Overrun);
    }
    if dr.be().bit_is_set() {
        return Err(Error::Break);
    }
    if dr.pe().bit_is_set() {
        return Err(Error::Parity);
    }
    if dr.fe().bit_is_set() {
        return Err(Error::Framing);
    }
    Ok(dr.data().bits())
}

fn calc_baud_div(uart_clk: u32, baud: u32) -> Option<(u32, u32)> {
    let temp = 16u32.checked_mul(baud)?;
    if baud == 0 || uart_clk < temp {
        return None;
    }

    let int_div = uart_clk / temp;
    let remainder = uart_clk % temp;
    let temp = 8 * remainder / baud;
    let fac_div = (temp >> 1) + (temp & 1);
    Some((int_div, fac_div & 0x3f))
}

fn uart_clock_hz(info: &'static Info) -> Result<u32, ConfigError> {
    let clocks = rcc::clocks().ok_or(ConfigError::ClocksNotInitialized)?;
    let rcc_regs = unsafe { pac::Rcc::steal() };
    // The PAC owns the per-UART selector positions; only the 2-bit value is
    // interpreted here. Values match the vendor clock tree.
    let sel = match info.index {
        0 => rcc_regs.cr2().read().uart0_clk_sel().bits(),
        1 => rcc_regs.cr2().read().uart1_clk_sel().bits(),
        2 => rcc_regs.cr2().read().uart2_clk_sel().bits(),
        3 => rcc_regs.cr2().read().uart3_clk_sel().bits(),
        _ => unreachable!(),
    };
    Ok(match sel {
        1 => RCO4M_HZ,
        2 => XO32K_HZ,
        3 => XO24M_HZ,
        _ if info.pclk0 => clocks.pclk0_hz,
        _ => clocks.pclk1_hz,
    })
}

fn erase_pin<'d>(
    pin: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
) -> Option<(Peri<'d, AnyPin>, AlternateFunction)> {
    pin.map(|(p, af)| (p.into(), af))
}

fn configure_pin(pin: Peri<'_, impl Pin>, af: AlternateFunction, output: bool) -> Flex<'_> {
    let mut flex = Flex::new(pin);
    if output {
        flex.set_as_output();
    } else {
        flex.set_as_input();
    }
    flex.set_alternate_function(af);
    flex
}

fn apply_config(info: &'static Info, config: &Config, has_rx: bool, has_tx: bool) -> Result<(), ConfigError> {
    let uart_clk = uart_clock_hz(info)?;
    let (ibrd, fbrd) = calc_baud_div(uart_clk, config.baudrate).ok_or(ConfigError::Baudrate)?;

    let regs = info.regs();

    // Disable UART and flush FIFOs by clearing FEN, matching `uart_init`.
    regs.cr().modify(|_, w| w.uart_en().clear_bit());
    regs.lcr_h().modify(|_, w| w.fen().clear_bit());
    unsafe {
        regs.imsc().write_with_zero(|w| w);
        regs.ibrd().write_with_zero(|w| w.baud_divint().bits(ibrd as u16));
        regs.fbrd().write_with_zero(|w| w.baud_divfrac().bits(fbrd as u8));
    }

    // Reset the software RX ring buffer to empty. The RX ISR may be running,
    // so mask interrupts while resetting the shared indices and error latch.
    critical_section::with(|_| info.state.reset_rx());
    info.state.baudrate.store(config.baudrate, Ordering::Release);

    let wlen = match config.data_bits {
        DataBits::DataBits5 => pac::uart0::lcr_h::Wlen::Value5,
        DataBits::DataBits6 => pac::uart0::lcr_h::Wlen::Value6,
        DataBits::DataBits7 => pac::uart0::lcr_h::Wlen::Value7,
        DataBits::DataBits8 => pac::uart0::lcr_h::Wlen::Value8,
    };
    let stop = match config.stop_bits {
        StopBits::STOP1 => pac::uart0::lcr_h::Stop::Value1,
        StopBits::STOP2 => pac::uart0::lcr_h::Stop::Value2,
    };

    regs.lcr_h().modify(|_, w| {
        w.wlen().variant(wlen);
        w.stop().variant(stop);
        w.fen().bit(config.fifo);
        match config.parity {
            Parity::None => {
                w.pen().clear_bit();
            }
            Parity::Odd => {
                w.pen().set_bit();
                w.eps_even().clear_bit();
            }
            Parity::Even => {
                w.pen().set_bit();
                w.eps_even().set_bit();
            }
        }
        w
    });

    let (txe, rxe) = match (has_rx, has_tx) {
        (true, true) => (true, true),
        (true, false) => (false, true),
        (false, true) => (true, false),
        (false, false) => return Err(ConfigError::NoRxOrTx),
    };
    let (rtsen, ctsen) = match config.flow_control {
        FlowControl::None => (false, false),
        FlowControl::Rts => (true, false),
        FlowControl::Cts => (false, true),
        FlowControl::RtsCts => (true, true),
    };

    regs.cr().modify(|_, w| {
        w.txe().bit(txe);
        w.rxe().bit(rxe);
        w.rtsen().bit(rtsen);
        w.ctsen().bit(ctsen);
        w
    });

    // Prefer a low TX interrupt threshold so TX wakes soon when space appears.
    regs.ifls().modify(|_, w| {
        w.tx().variant(pac::uart0::ifls::Tx::Value1_8);
        w.rx().variant(pac::uart0::ifls::Rx::Value1_8);
        w
    });

    regs.cr().modify(|_, w| w.uart_en().set_bit());
    Ok(())
}

fn enable_clock(info: &'static Info) -> Result<(), ConfigError> {
    rcc::enable_peripheral(info.rcc).map_err(ConfigError::Rcc)?;
    rcc::reset_peripheral(info.rcc).map_err(ConfigError::Rcc)?;
    Ok(())
}

fn shutdown(info: &'static Info) {
    let regs = info.regs();
    regs.cr().modify(|_, w| w.uart_en().clear_bit());
    // Disable-all writes: every enable in these registers defaults off.
    unsafe {
        regs.imsc().write_with_zero(|w| w);
    }
    regs.dmacr().modify(|_, w| w.tx_en().clear_bit().rx_en().clear_bit());
    let _ = rcc::disable_peripheral(info.rcc);
}

/// Enable or disable the receive interrupt group (RX, receive-timeout,
/// and all receive-error sources).
fn set_rx_interrupts(regs: &RegisterBlock, enable: bool) {
    regs.imsc().modify(|_, w| {
        w.rxim().bit(enable);
        w.rtim().bit(enable);
        w.feim().bit(enable);
        w.peim().bit(enable);
        w.beim().bit(enable);
        w.oeim().bit(enable)
    });
}

/// Enable or disable the transmit interrupt.
fn set_tx_interrupt(regs: &RegisterBlock, enable: bool) {
    regs.imsc().modify(|_, w| w.txim().bit(enable));
}

fn read_dr(regs: &RegisterBlock) -> Result<u8, Error> {
    let dr = regs.dr().read();
    // Writing RSC_ECR clears sticky receive-status bits (PL011 / SDK).
    if dr.fe().bit_is_set() || dr.pe().bit_is_set() || dr.be().bit_is_set() || dr.oe().bit_is_set() {
        unsafe {
            regs.rsr_ecr().write_with_zero(|w| w.bits(0));
        }
    }
    decode_dr_error(&dr)
}

fn write_dr(regs: &RegisterBlock, byte: u8) {
    unsafe {
        regs.dr().write_with_zero(|w| w.data().bits(byte));
    }
}

fn set_dma_req(regs: &RegisterBlock, tx: bool, enable: bool) {
    regs.dmacr().modify(|_, w| {
        if tx {
            w.tx_en().bit(enable);
        } else {
            w.rx_en().bit(enable);
        }
        w
    });
}

/// Bidirectional UART driver.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

/// Transmit half of a UART.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    has_tx: bool,
    has_rx: bool,
    // `Flex` (not bare `Peri`) so dropping the half deconfigures the pins
    // back to analog via `Flex::drop`.
    _pin: Option<Flex<'d>>,
    _rts: Option<Flex<'d>>,
    _mode: PhantomData<&'d mut M>,
}

/// Receive half of a UART.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    has_tx: bool,
    has_rx: bool,
    _pin: Option<Flex<'d>>,
    _cts: Option<Flex<'d>>,
    _mode: PhantomData<&'d mut M>,
}

impl<'d> Uart<'d, Blocking> {
    /// Create a blocking UART.
    ///
    /// `tx` / `rx` are `(pin, alternate_function)` pairs. Supply `None` for
    /// unused directions. Flow-control pins are not configured here; use
    /// [`Self::new_blocking_with_rtscts`] when RTS/CTS are required.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(peri, erase_pin(tx), erase_pin(rx), None, None, config)
    }

    /// Create a blocking UART with explicit RTS/CTS pins and AF selectors.
    pub fn new_blocking_with_rtscts<T: Instance>(
        peri: Peri<'d, T>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(
            peri,
            erase_pin(tx),
            erase_pin(rx),
            erase_pin(rts),
            erase_pin(cts),
            config,
        )
    }
}

impl<'d> Uart<'d, Async> {
    /// Create an interrupt-driven asynchronous UART.
    ///
    /// `irq` proves the UART vector is bound to [`InterruptHandler<T>`].
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let has_rx = rx.is_some();
        let this = Self::new_inner::<T>(peri, erase_pin(tx), erase_pin(rx), None, None, config)?;
        this.enable_interrupts::<T>(has_rx);
        Ok(this)
    }

    /// Create an asynchronous UART with explicit RTS/CTS pins and AF selectors.
    pub fn new_with_rtscts<T: Instance>(
        peri: Peri<'d, T>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let has_rx = rx.is_some();
        let this = Self::new_inner::<T>(
            peri,
            erase_pin(tx),
            erase_pin(rx),
            erase_pin(rts),
            erase_pin(cts),
            config,
        )?;
        this.enable_interrupts::<T>(has_rx);
        Ok(this)
    }

    /// Write `buffer` using interrupt-driven TX.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Read into `buffer` until it is full using interrupt-driven RX.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Wait until the transmit FIFO and shift register are empty.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Write `buffer` with a DMA channel (memory-to-peripheral).
    ///
    /// Enables UART TX DMA for the duration of the transfer. Prefer
    /// [`Config::fifo`] enabled.
    pub async fn write_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write_dma(channel, buffer).await
    }

    /// Read `buffer.len()` bytes with a DMA channel (peripheral-to-memory).
    pub async fn read_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read_dma(channel, buffer).await
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    /// Arm the NVIC vector and, when receiving, the RX drain interrupt.
    ///
    /// Called by the async constructors only; blocking instances never
    /// enable the vector. The RX path keeps its interrupt armed; see
    /// [`InterruptHandler::on_interrupt`].
    fn enable_interrupts<T: Instance>(&self, has_rx: bool) {
        let info = self.tx.info;
        // Arm the RX interrupt so the ISR starts draining the hardware FIFO
        // into the ring buffer immediately.
        if has_rx {
            set_rx_interrupts(info.regs(), true);
        }
        T::Interrupt::unpend();
        // Run above the RTC time-driver (P2) and DMA (P2) interrupts. The
        // RTC fires at 32.768 kHz and its handler can spin in `wait_sync()`
        // across the async clock domain; at P3 the UART RX ISR is starved
        // during those windows and the 16-byte hardware RX FIFO overruns,
        // dropping bytes. At P1 capture is never delayed by the time driver.
        T::Interrupt::set_priority(Priority::P1);
        unsafe {
            T::Interrupt::enable();
        }
    }

    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        cts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        match config.flow_control {
            FlowControl::Rts if rts.is_none() => return Err(ConfigError::MissingFlowControlPin),
            FlowControl::Cts if cts.is_none() => return Err(ConfigError::MissingFlowControlPin),
            FlowControl::RtsCts if rts.is_none() || cts.is_none() => {
                return Err(ConfigError::MissingFlowControlPin);
            }
            _ => {}
        }

        let info = T::info();
        enable_clock(info)?;

        let has_rx = rx.is_some();
        let has_tx = tx.is_some();

        let rx_pin = rx.map(|(p, af)| configure_pin(p, af, false));
        let tx_pin = tx.map(|(p, af)| configure_pin(p, af, true));
        let rts_pin = rts.map(|(p, af)| configure_pin(p, af, true));
        let cts_pin = cts.map(|(p, af)| configure_pin(p, af, false));

        apply_config(info, &config, has_rx, has_tx)?;

        info.state.refcount.store(2, Ordering::Release);

        Ok(Self {
            tx: UartTx {
                info,
                has_tx,
                has_rx,
                _pin: tx_pin,
                _rts: rts_pin,
                _mode: PhantomData,
            },
            rx: UartRx {
                info,
                has_tx,
                has_rx,
                _pin: rx_pin,
                _cts: cts_pin,
                _mode: PhantomData,
            },
        })
    }

    /// Split into independent TX and RX drivers.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Reborrow as independent TX and RX drivers.
    ///
    /// Deliberately `&mut` borrows rather than the guideline's owned-halves
    /// form: the two halves share one peripheral closed over by a shutdown
    /// refcount (see the `Drop` impls below), so owned `split_ref` halves
    /// would run `Drop` on scope exit and shut the peripheral down while the
    /// driver is still alive.
    pub fn split_ref(&mut self) -> (&mut UartTx<'d, M>, &mut UartRx<'d, M>) {
        (&mut self.tx, &mut self.rx)
    }

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Blocking read that fills the entire buffer.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Blocking flush of the transmitter.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Whether the transmitter is currently busy sending a frame.
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    /// Change the baud rate.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)
    }

    /// Send a break condition on the TX line.
    pub fn send_break(&mut self) {
        self.tx.send_break();
    }

    /// Reconfigure the UART with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.tx.info, config, self.tx.has_rx, self.tx.has_tx)
    }

    /// Blocking DMA write.
    pub fn blocking_write_dma(&mut self, channel: &mut DmaChannel<'_, Blocking>, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write_dma(channel, buffer)
    }

    /// Blocking DMA read of `buffer.len()` bytes.
    pub fn blocking_read_dma(
        &mut self,
        channel: &mut DmaChannel<'_, Blocking>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.rx.blocking_read_dma(channel, buffer)
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Send a break condition (TX line held low for at least one frame time).
    ///
    /// This sets the BRK bit in LCR_H, holds it for at least one frame time,
    /// then clears it.
    pub fn send_break(&mut self) {
        let regs = self.regs();
        // Hold BRK for at least one 10-bit frame at the configured baud rate.
        let baud = match self.info.state.baudrate.load(Ordering::Relaxed) {
            0 => 115_200,
            baud => baud,
        };
        let hclk = crate::rcc::clocks().map(|c| c.hclk_hz).unwrap_or(24_000_000);
        let cycles = (u64::from(hclk) * 10)
            .div_ceil(u64::from(baud))
            .min(u64::from(u32::MAX)) as u32;

        regs.lcr_h().modify(|_, w| w.brk().set_bit());
        cortex_m::asm::delay(cycles);
        regs.lcr_h().modify(|_, w| w.brk().clear_bit());
    }

    /// Change the baud rate.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        let regs = self.regs();
        let info = self.info;
        let uart_clk = uart_clock_hz(info)?;
        let (ibrd, fbrd) = calc_baud_div(uart_clk, baudrate).ok_or(ConfigError::Baudrate)?;

        // ARM PL011 requires the UART to be disabled while changing divisors.
        regs.cr().modify(|_, w| w.uart_en().clear_bit());
        unsafe {
            regs.ibrd().write_with_zero(|w| w.baud_divint().bits(ibrd as u16));
            regs.fbrd().write_with_zero(|w| w.baud_divfrac().bits(fbrd as u8));
        }
        // LCR_H write is needed to latch the new baud rate divisors.
        regs.lcr_h().modify(|r, w| unsafe { w.bits(r.bits()) });
        regs.cr().modify(|_, w| w.uart_en().set_bit());
        info.state.baudrate.store(baudrate, Ordering::Release);
        Ok(())
    }

    /// Reconfigure with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config, self.has_rx, self.has_tx)
    }

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
        for &byte in buffer {
            while regs.fr().read().txff().bit_is_set() {}
            write_dr(regs, byte);
        }
        Ok(())
    }

    /// Blocking flush: wait until TX FIFO empty and UART not busy.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let regs = self.regs();
        while !regs.fr().read().txfe().bit_is_set() || regs.fr().read().busy().bit_is_set() {}
        Ok(())
    }

    /// Whether the transmitter is currently busy sending a frame.
    pub fn busy(&self) -> bool {
        self.regs().fr().read().busy().bit_is_set()
    }

    /// Blocking DMA write.
    pub fn blocking_write_dma(&mut self, channel: &mut DmaChannel<'_, Blocking>, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let dest = regs.dr().as_ptr();
        set_dma_req(regs, true, true);
        let result = unsafe {
            channel
                .start_write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .blocking_wait()
                .map_err(Error::Dma)
        };
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d> UartTx<'d, Async> {
    /// Interrupt-driven write of the entire buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        // If dropped mid-byte the TX interrupt would stay armed with no
        // waiter; disarm it. Steady-state RX stays armed by design.
        let irq_guard = OnDrop::new(|| set_tx_interrupt(regs, false));
        for &byte in buffer {
            poll_fn(|cx| {
                if !regs.fr().read().txff().bit_is_set() {
                    return Poll::Ready(());
                }
                info.state.tx_waker.register(cx.waker());
                set_tx_interrupt(regs, true);
                if !regs.fr().read().txff().bit_is_set() {
                    set_tx_interrupt(regs, false);
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
            write_dr(regs, byte);
        }
        irq_guard.defuse();
        set_tx_interrupt(regs, false);
        Ok(())
    }

    /// Wait until TX FIFO empty and UART not busy.
    ///
    /// The ASR6601 TX FIFO level interrupt is a one-shot per threshold
    /// crossing: it does not re-assert while the FIFO keeps draining (unlike
    /// the PL011), so the final `TXFE && !BUSY` state is never signaled by an
    /// interrupt. Besides the interrupt, re-check the flags on a timer.
    ///
    /// The re-check interval must stay in the millisecond range: LPTIM0
    /// compare values only a few ticks ahead of the running counter are
    /// missed for the current 2 s cycle on this silicon, delaying the alarm
    /// by up to one full period.
    ///
    /// Requires the `embassy-time` driver to be initialized (used for the re-check timer).
    pub async fn flush(&mut self) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        // Fast path: already drained, avoid arming the interrupt and timer.
        if regs.fr().read().txfe().bit_is_set() && !regs.fr().read().busy().bit_is_set() {
            return Ok(());
        }
        // If dropped mid-flush the TX interrupt would stay armed with no
        // waiter; disarm it.
        let irq_guard = OnDrop::new(|| set_tx_interrupt(regs, false));
        loop {
            let event = poll_fn(|cx| {
                if regs.fr().read().txfe().bit_is_set() && !regs.fr().read().busy().bit_is_set() {
                    return Poll::Ready(());
                }
                // TX interrupt fires when the FIFO drops to the threshold.
                info.state.tx_waker.register(cx.waker());
                set_tx_interrupt(regs, true);
                if regs.fr().read().txfe().bit_is_set() && !regs.fr().read().busy().bit_is_set() {
                    set_tx_interrupt(regs, false);
                    return Poll::Ready(());
                }
                Poll::Pending
            });
            let timer = embassy_time::Timer::after(embassy_time::Duration::from_millis(5));
            match embassy_futures::select::select(event, timer).await {
                embassy_futures::select::Either::First(()) => break,
                embassy_futures::select::Either::Second(()) => continue,
            }
        }
        irq_guard.defuse();
        set_tx_interrupt(regs, false);
        Ok(())
    }

    /// DMA write using an async DMA channel.
    pub async fn write_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let dest = regs.dr().as_ptr();
        set_dma_req(regs, true, true);
        compiler_fence(Ordering::SeqCst);
        // If dropped mid-transfer the DMA request would stay armed with no
        // waiter; disarm it. The DMA `Transfer` future aborts its own channel.
        let dma_guard = OnDrop::new(|| set_dma_req(regs, true, false));
        let result = unsafe {
            channel
                .write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        dma_guard.defuse();
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Reconfigure with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config, self.has_rx, self.has_tx)
    }

    /// Blocking read that fills the entire buffer.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let regs = self.regs();
        for slot in buffer.iter_mut() {
            while regs.fr().read().rxfe().bit_is_set() {}
            *slot = read_dr(regs)?;
        }
        Ok(())
    }

    /// Blocking DMA read of `buffer.len()` bytes.
    pub fn blocking_read_dma(
        &mut self,
        channel: &mut DmaChannel<'_, Blocking>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let src = regs.dr().as_ptr();
        set_dma_req(regs, false, true);
        let result = unsafe {
            channel
                .start_read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .blocking_wait()
                .map_err(Error::Dma)
        };
        set_dma_req(regs, false, false);
        result
    }

    fn nb_read_byte(&mut self) -> Result<Option<u8>, Error> {
        let regs = self.regs();
        if regs.fr().read().rxfe().bit_is_set() {
            Ok(None)
        } else {
            Ok(Some(read_dr(regs)?))
        }
    }
}

impl<'d> UartRx<'d, Async> {
    /// Wait until at least one byte is available in the software RX ring
    /// buffer (which the ISR keeps filled from the hardware FIFO).
    async fn wait_rx_ready(&mut self) -> Result<(), Error> {
        let info = self.info;
        poll_fn(|cx| {
            if let Some(error) = take_rx_error(info.state) {
                return Poll::Ready(Err(error));
            }
            if info.state.rx_head.load(Ordering::Acquire) != info.state.rx_tail.load(Ordering::Acquire) {
                return Poll::Ready(Ok(()));
            }
            info.state.rx_waker.register(cx.waker());
            // Make sure the RX interrupt is armed so the ISR keeps capturing.
            set_rx_interrupts(info.regs(), true);
            if let Some(error) = take_rx_error(info.state) {
                return Poll::Ready(Err(error));
            }
            if info.state.rx_head.load(Ordering::Acquire) != info.state.rx_tail.load(Ordering::Acquire) {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await
        // Leave the RX interrupt armed; the ISR keeps draining the FIFO into the
        // ring buffer so no bytes are lost between reads.
    }

    /// Read the next byte from the software RX ring buffer, if any.
    fn nb_read_byte_ring(&mut self) -> Result<Option<u8>, Error> {
        let info = self.info;
        let tail = info.state.rx_tail.load(Ordering::Acquire) as usize;
        if tail == info.state.rx_head.load(Ordering::Acquire) as usize {
            return Ok(None);
        }
        let b = unsafe { (*info.state.rx_buf.get())[tail] };
        info.state
            .rx_tail
            .store(((tail + 1) % RX_BUF_CAP) as u16, Ordering::Release);
        Ok(Some(b))
    }

    /// Interrupt-driven read that fills the entire buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            self.wait_rx_ready().await?;
            // `wait_rx_ready` guarantees a byte is present.
            *slot = self.nb_read_byte_ring()?.unwrap();
        }
        Ok(())
    }

    /// DMA read of `buffer.len()` bytes.
    pub async fn read_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let src = regs.dr().as_ptr();
        set_dma_req(regs, false, true);
        compiler_fence(Ordering::SeqCst);
        // If dropped mid-transfer the DMA request would stay armed with no
        // waiter; disarm it. The DMA `Transfer` future aborts its own channel.
        let dma_guard = OnDrop::new(|| set_dma_req(regs, false, false));
        let result = unsafe {
            channel
                .read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        dma_guard.defuse();
        set_dma_req(regs, false, false);
        result
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for Uart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d, M: Mode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        if self.info.state.refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
            shutdown(self.info);
        }
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        if self.info.state.refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
            shutdown(self.info);
        }
    }
}

// --- embedded-io -----------------------------------------------------------

impl<'d, M: Mode> embedded_io::ErrorType for Uart<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for UartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for UartRx<'d, M> {
    type Error = Error;
}

impl<'d> embedded_io::Write for UartTx<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::Write for Uart<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for UartTx<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.regs().fr().read().txff().bit_is_set())
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for Uart<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.tx.regs().fr().read().txff().bit_is_set())
    }
}

impl<'d> embedded_io::Read for UartRx<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        // Block for the first byte, then drain the FIFO without further waits.
        while self.regs().fr().read().rxfe().bit_is_set() {}
        let mut n = 0;
        while n < buf.len() {
            match self.nb_read_byte()? {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        Ok(n)
    }
}

impl<'d> embedded_io::Read for Uart<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for UartRx<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        let state = self.info.state;
        Ok(
            state.rx_head.load(Ordering::Acquire) != state.rx_tail.load(Ordering::Acquire)
                || !self.regs().fr().read().rxfe().bit_is_set(),
        )
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for Uart<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        self.rx.read_ready()
    }
}

impl<'d> embedded_io_async::Write for UartTx<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        UartTx::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        UartTx::flush(self).await
    }
}

impl<'d> embedded_io_async::Write for Uart<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Uart::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Uart::flush(self).await
    }
}

impl<'d> embedded_io_async::Read for UartRx<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.wait_rx_ready().await?;
        let mut n = 0;
        while n < buf.len() {
            match self.nb_read_byte_ring()? {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        Ok(n)
    }
}

impl<'d> embedded_io_async::Read for Uart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(&mut self.rx, buf).await
    }
}

// --- Instance --------------------------------------------------------------

trait SealedInstance {
    fn info() -> &'static Info;
}

/// UART instance (UART0–UART3).
#[allow(private_bounds, private_interfaces)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// NVIC vector for this UART.
    type Interrupt: Interrupt;
}

macro_rules! impl_uart {
    ($peri:ident, $interrupt:ident, $index:expr, $base:expr, $rcc:ident, $pclk0:expr, $dma_tx:ident, $dma_rx:ident) => {
        #[allow(private_interfaces)]
        impl SealedInstance for peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    index: $index,
                    base: $base,
                    rcc: Peripheral::$rcc,
                    pclk0: $pclk0,
                    dma_tx: Request::$dma_tx,
                    dma_rx: Request::$dma_rx,
                    state: &STATE[$index],
                };
                &INFO
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_uart!(UART0, UART0, 0, 0x4000_3000, Uart0, true, Uart0Tx, Uart0Rx);
impl_uart!(UART1, UART1, 1, 0x4000_4000, Uart1, true, Uart1Tx, Uart1Rx);
impl_uart!(UART2, UART2, 2, 0x4001_0000, Uart2, false, Uart2Tx, Uart2Rx);
impl_uart!(UART3, UART3, 3, 0x4001_1000, Uart3, false, Uart3Tx, Uart3Rx);

// --- embedded-hal 0.2 blocking traits ----------------------------------------

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}
