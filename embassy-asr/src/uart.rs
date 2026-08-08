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

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::interrupt::Priority;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Channel as DmaChannel, Request, TransferOptions};
use crate::gpio::{AlternateFunction, AnyPin, Flex, Pin};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::uart0::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::{interrupt, pac, peripherals};

const FLAG_TXFE: u32 = 1 << 7;
const FLAG_TXFF: u32 = 1 << 5;
const FLAG_RXFE: u32 = 1 << 4;
const FLAG_BUSY: u32 = 1 << 3;

const INT_RX: u32 = 1 << 4;
const INT_TX: u32 = 1 << 5;
const INT_RT: u32 = 1 << 6;
const INT_FE: u32 = 1 << 7;
const INT_PE: u32 = 1 << 8;
const INT_BE: u32 = 1 << 9;
const INT_OE: u32 = 1 << 10;
const INT_RX_ALL: u32 = INT_RX | INT_RT | INT_FE | INT_PE | INT_BE | INT_OE;

const DR_FE: u32 = 1 << 8;
const DR_PE: u32 = 1 << 9;
const DR_BE: u32 = 1 << 10;
const DR_OE: u32 = 1 << 11;
const DR_ERROR_MASK: u32 = DR_FE | DR_PE | DR_BE | DR_OE;

const RCO4M_HZ: u32 = 3_600_000;
const XO32K_HZ: u32 = 32_768;
const XO24M_HZ: u32 = 24_000_000;

static STATE: [State; 4] = [const { State::new() }; 4];

struct State {
    tx_waker: AtomicWaker,
    rx_waker: AtomicWaker,
    /// Number of live [`UartTx`]/ [`UartRx`] halves.
    refcount: AtomicU8,
}

impl State {
    const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            rx_waker: AtomicWaker::new(),
            refcount: AtomicU8::new(0),
        }
    }
}

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
        let mis = regs.mis().read().bits();

        if mis == 0 {
            return;
        }

        if mis & INT_TX != 0 {
            // Disable TX interrupt; the TX task re-enables when needed.
            let imsc = regs.imsc().read().bits() & !INT_TX;
            unsafe {
                regs.imsc().write_with_zero(|w| w.bits(imsc));
            }
            info.state.tx_waker.wake();
        }

        if mis & INT_RX_ALL != 0 {
            let imsc = regs.imsc().read().bits() & !INT_RX_ALL;
            unsafe {
                regs.imsc().write_with_zero(|w| w.bits(imsc));
            }
            info.state.rx_waker.wake();
        }

        // ICR is write-1-to-clear; the PAC models it without field accessors.
        unsafe {
            regs.icr().write_with_zero(|w| w.bits(mis));
        }
    }
}

/// Number of data bits.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 5 data bits.
    DataBits5,
    /// 6 data bits.
    DataBits6,
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
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

/// Number of stop bits.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// One stop bit.
    STOP1,
    /// Two stop bits.
    STOP2,
}

/// Hardware flow control.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl {
    /// No RTS/CTS.
    None,
    /// RTS only.
    Rts,
    /// CTS only.
    Cts,
    /// RTS and CTS.
    RtsCts,
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

/// UART transfer / framing error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Self::Overrun => embedded_io::ErrorKind::OutOfMemory,
            Self::Framing | Self::Parity | Self::Break => embedded_io::ErrorKind::InvalidData,
            Self::Dma(_) => embedded_io::ErrorKind::Other,
        }
    }
}

fn decode_dr_error(value: u32) -> Result<u8, Error> {
    if value & DR_OE != 0 {
        return Err(Error::Overrun);
    }
    if value & DR_BE != 0 {
        return Err(Error::Break);
    }
    if value & DR_PE != 0 {
        return Err(Error::Parity);
    }
    if value & DR_FE != 0 {
        return Err(Error::Framing);
    }
    Ok((value & 0xff) as u8)
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
    let shift = match info.index {
        0 => 15,
        1 => 13,
        2 => 11,
        3 => 9,
        _ => unreachable!(),
    };
    let sel = (rcc_regs.cr2().read().bits() >> shift) & 0x3;
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

fn configure_pin(pin: Peri<'_, impl Pin>, af: AlternateFunction, output: bool) -> Peri<'_, AnyPin> {
    let mut flex = Flex::new(pin);
    if output {
        flex.set_as_output();
    } else {
        flex.set_as_input();
    }
    flex.set_alternate_function(af);
    flex.into_inner()
}

fn apply_config(info: &'static Info, config: Config, has_rx: bool, has_tx: bool) -> Result<(), ConfigError> {
    let uart_clk = uart_clock_hz(info)?;
    let (ibrd, fbrd) = calc_baud_div(uart_clk, config.baudrate).ok_or(ConfigError::Baudrate)?;

    let regs = info.regs();

    // Disable UART and flush FIFOs by clearing FEN, matching `uart_init`.
    regs.cr().modify(|_, w| w.uart_en().clear_bit());
    regs.lcr_h().modify(|_, w| w.fen().clear_bit());
    unsafe {
        regs.imsc().write_with_zero(|w| w.bits(0));
        regs.ibrd().write_with_zero(|w| w.bits(ibrd));
        regs.fbrd().write_with_zero(|w| w.bits(fbrd));
    }

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

    let mode = match (has_rx, has_tx) {
        (true, true) => pac::uart0::cr::UartMode::Txrx,
        (true, false) => pac::uart0::cr::UartMode::Rx,
        (false, true) => pac::uart0::cr::UartMode::Tx,
        (false, false) => return Err(ConfigError::NoRxOrTx),
    };
    let flow = match config.flow_control {
        FlowControl::None => pac::uart0::cr::FlowCtrl::None,
        FlowControl::Rts => pac::uart0::cr::FlowCtrl::Rts,
        FlowControl::Cts => pac::uart0::cr::FlowCtrl::Cts,
        FlowControl::RtsCts => pac::uart0::cr::FlowCtrl::CtsRts,
    };

    regs.cr().modify(|_, w| {
        w.uart_mode().variant(mode);
        w.flow_ctrl().variant(flow);
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
    unsafe {
        regs.imsc().write_with_zero(|w| w.bits(0));
        regs.dmacr().write_with_zero(|w| w.bits(0));
    }
    let _ = rcc::disable_peripheral(info.rcc);
}

fn flag(regs: &RegisterBlock, mask: u32) -> bool {
    regs.fr().read().bits() & mask != 0
}

fn read_dr(regs: &RegisterBlock) -> Result<u8, Error> {
    let value = regs.dr().read().bits();
    // Writing RSC_ECR clears sticky receive-status bits (PL011 / SDK).
    if value & DR_ERROR_MASK != 0 {
        unsafe {
            regs.rsc_ecr().write_with_zero(|w| w.bits(0));
        }
    }
    decode_dr_error(value)
}

fn write_dr(regs: &RegisterBlock, byte: u8) {
    unsafe {
        regs.dr().write_with_zero(|w| w.bits(u32::from(byte)));
    }
}

fn set_imsc_bits(regs: &RegisterBlock, mask: u32, enable: bool) {
    let value = regs.imsc().read().bits();
    let value = if enable { value | mask } else { value & !mask };
    unsafe {
        regs.imsc().write_with_zero(|w| w.bits(value));
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
    _pin: Option<Peri<'d, AnyPin>>,
    _rts: Option<Peri<'d, AnyPin>>,
    _mode: PhantomData<&'d mut M>,
}

/// Receive half of a UART.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    _pin: Option<Peri<'d, AnyPin>>,
    _cts: Option<Peri<'d, AnyPin>>,
    _mode: PhantomData<&'d mut M>,
}

impl<'d> Uart<'d, Blocking> {
    /// Create a blocking UART.
    ///
    /// `rx` / `tx` are `(pin, alternate_function)` pairs. Supply `None` for
    /// unused directions. Flow-control pins are not configured here; use
    /// [`Self::new_blocking_with_flow_control`] when RTS/CTS are required.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(peri, erase_pin(rx), erase_pin(tx), None, None, config, false)
    }

    /// Create a blocking UART with explicit RTS/CTS pins and AF selectors.
    pub fn new_blocking_with_flow_control<T: Instance>(
        peri: Peri<'d, T>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(
            peri,
            erase_pin(rx),
            erase_pin(tx),
            erase_pin(rts),
            erase_pin(cts),
            config,
            false,
        )
    }
}

impl<'d> Uart<'d, Async> {
    /// Create an interrupt-driven asynchronous UART.
    ///
    /// `irq` proves the UART vector is bound to [`InterruptHandler<T>`].
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(peri, erase_pin(rx), erase_pin(tx), None, None, config, true)
    }

    /// Create an asynchronous UART with explicit RTS/CTS pins and AF selectors.
    pub fn new_with_flow_control<T: Instance>(
        peri: Peri<'d, T>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner::<T>(
            peri,
            erase_pin(rx),
            erase_pin(tx),
            erase_pin(rts),
            erase_pin(cts),
            config,
            true,
        )
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
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        tx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        cts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        config: Config,
        enable_irq: bool,
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

        apply_config(info, config, has_rx, has_tx)?;

        info.state.refcount.store(2, Ordering::Release);

        if enable_irq {
            T::Interrupt::unpend();
            T::Interrupt::set_priority(Priority::P3);
            unsafe {
                T::Interrupt::enable();
            }
        }

        Ok(Self {
            tx: UartTx {
                info,
                _pin: tx_pin,
                _rts: rts_pin,
                _mode: PhantomData,
            },
            rx: UartRx {
                info,
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

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
        for &byte in buffer {
            while flag(regs, FLAG_TXFF) {}
            write_dr(regs, byte);
        }
        Ok(())
    }

    /// Blocking flush: wait until TX FIFO empty and UART not busy.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let regs = self.regs();
        while !flag(regs, FLAG_TXFE) || flag(regs, FLAG_BUSY) {}
        Ok(())
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
        for &byte in buffer {
            poll_fn(|cx| {
                if !flag(regs, FLAG_TXFF) {
                    return Poll::Ready(());
                }
                info.state.tx_waker.register(cx.waker());
                set_imsc_bits(regs, INT_TX, true);
                if !flag(regs, FLAG_TXFF) {
                    set_imsc_bits(regs, INT_TX, false);
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
            write_dr(regs, byte);
        }
        set_imsc_bits(regs, INT_TX, false);
        Ok(())
    }

    /// Wait until TX FIFO empty and UART not busy.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        poll_fn(|cx| {
            if flag(regs, FLAG_TXFE) && !flag(regs, FLAG_BUSY) {
                return Poll::Ready(());
            }
            // TX interrupt fires when FIFO drops to the threshold; also useful while draining.
            info.state.tx_waker.register(cx.waker());
            set_imsc_bits(regs, INT_TX, true);
            if flag(regs, FLAG_TXFE) && !flag(regs, FLAG_BUSY) {
                set_imsc_bits(regs, INT_TX, false);
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
        set_imsc_bits(regs, INT_TX, false);
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
        let result = unsafe {
            channel
                .write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Blocking read that fills the entire buffer.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let regs = self.regs();
        for slot in buffer.iter_mut() {
            while flag(regs, FLAG_RXFE) {}
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
        if flag(regs, FLAG_RXFE) {
            Ok(None)
        } else {
            Ok(Some(read_dr(regs)?))
        }
    }
}

impl<'d> UartRx<'d, Async> {
    async fn wait_rx_ready(&mut self) {
        let info = self.info;
        let regs = info.regs();
        poll_fn(|cx| {
            if !flag(regs, FLAG_RXFE) {
                return Poll::Ready(());
            }
            info.state.rx_waker.register(cx.waker());
            set_imsc_bits(regs, INT_RX_ALL, true);
            if !flag(regs, FLAG_RXFE) {
                set_imsc_bits(regs, INT_RX_ALL, false);
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
        set_imsc_bits(regs, INT_RX_ALL, false);
    }

    /// Interrupt-driven read that fills the entire buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            self.wait_rx_ready().await;
            *slot = read_dr(self.regs())?;
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
        let result = unsafe {
            channel
                .read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        set_dma_req(regs, false, false);
        result
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

impl<'d, M: Mode> embedded_io::Write for UartTx<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_io::Write for Uart<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<'d, M: Mode> embedded_io::Read for UartRx<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        // Block for the first byte, then drain the FIFO without further waits.
        while flag(self.regs(), FLAG_RXFE) {}
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

impl<'d, M: Mode> embedded_io::Read for Uart<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
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
        self.wait_rx_ready().await;
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

impl<'d> embedded_io_async::Read for Uart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(&mut self.rx, buf).await
    }
}

// --- Instance --------------------------------------------------------------

mod sealed {
    use super::Info;

    pub trait Instance {
        fn info() -> &'static Info;
    }
}

/// UART instance (UART0–UART3).
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for this UART.
    type Interrupt: Interrupt;
}

macro_rules! impl_uart {
    ($peri:ident, $interrupt:ident, $index:expr, $base:expr, $rcc:ident, $pclk0:expr, $dma_tx:ident, $dma_rx:ident) => {
        impl sealed::Instance for peripherals::$peri {
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
