use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use paste::paste;

use crate::clocks::periph_helpers::{Div4, LpuartClockSel, LpuartConfig};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::pac::lpuart::vals::{
    Idlecfg as IdleConfig, Ilt as IdleType, M as DataBits, Msbf as MsbFirst, Pt as Parity, Rst, Rxflush,
    Sbns as StopBits, Swap, Tc, Tdre, Txctsc as TxCtsConfig, Txctssrc as TxCtsSource, Txflush,
};
use crate::pac::{self};

pub mod buffered;

// ============================================================================
// DMA INTEGRATION
// ============================================================================

use crate::dma::{
    Channel as DmaChannelTrait, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt, RingBuffer,
};

// ============================================================================
// MISC
// ============================================================================

mod sealed {
    /// Simply seal a trait to prevent external implementations
    pub trait Sealed {}
}

// ============================================================================
// INSTANCE TRAIT
// ============================================================================

trait SealedInstance {
    fn info() -> &'static Info;
    fn buffered_state() -> &'static buffered::State;
}

struct Info {
    regs: crate::pac::lpuart::Lpuart,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> crate::pac::lpuart::Lpuart {
        self.regs
    }
}

/// Trait for LPUART peripheral instances
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpuartConfig> {
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpuartInstance;
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Type-safe DMA request source for TX
    type TxDmaRequest: DmaRequest;
    /// Type-safe DMA request source for RX
    type RxDmaRequest: DmaRequest;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

macro_rules! impl_instance {
    ($($n:expr);* $(;)?) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPUART $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info = Info {
                            regs: pac::[<LPUART $n>],
                        };
                        &INFO
                    }

                    fn buffered_state() -> &'static buffered::State {
                        static BUFFERED_STATE: buffered::State = buffered::State::new();
                        &BUFFERED_STATE
                    }
                }

                impl Instance for crate::peripherals::[<LPUART $n>] {
                    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpuartInstance
                        = crate::clocks::periph_helpers::LpuartInstance::[<Lpuart $n>];
                    type Interrupt = crate::interrupt::typelevel::[<LPUART $n>];
                    type TxDmaRequest = crate::dma::[<Lpuart $n TxRequest>];
                    type RxDmaRequest = crate::dma::[<Lpuart $n RxRequest>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_lpuart $n>];
                    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_lpuart $n _wake>];
                }
            }
        )*
    };
}

// DMA request sources are now type-safe via associated types.
// The request source numbers are defined in src/dma.rs:
// LPUART0: RX=21, TX=22 -> Lpuart0RxRequest, Lpuart0TxRequest
// LPUART1: RX=23, TX=24 -> Lpuart1RxRequest, Lpuart1TxRequest
// LPUART2: RX=25, TX=26 -> Lpuart2RxRequest, Lpuart2TxRequest
// LPUART3: RX=27, TX=28 -> Lpuart3RxRequest, Lpuart3TxRequest
// LPUART4: RX=29, TX=30 -> Lpuart4RxRequest, Lpuart4TxRequest
// LPUART5: RX=31, TX=32 -> Lpuart5RxRequest, Lpuart5TxRequest
impl_instance!(0; 1; 2; 3; 4; 5);

// ============================================================================
// INSTANCE HELPER FUNCTIONS
// ============================================================================

/// Perform software reset on the LPUART peripheral
fn perform_software_reset(info: &'static Info) {
    // Software reset - set and clear RST bit (Global register)
    info.regs().global().write(|w| w.set_rst(Rst::RESET));
    info.regs().global().write(|w| w.set_rst(Rst::NO_EFFECT));
}

/// Disable both transmitter and receiver
fn disable_transceiver(info: &'static Info) {
    info.regs().ctrl().modify(|w| {
        w.set_te(false);
        w.set_re(false);
    });
}

/// Calculate and configure baudrate settings
fn configure_baudrate(info: &'static Info, baudrate_bps: u32, clock_freq: u32) -> Result<()> {
    let (osr, sbr) = calculate_baudrate(baudrate_bps, clock_freq)?;

    // Configure BAUD register
    info.regs().baud().modify(|w| {
        // Clear and set OSR
        w.set_osr(osr - 1);
        // Clear and set SBR
        w.set_sbr(sbr);
        // Set BOTHEDGE if OSR is between 4 and 7
        w.set_bothedge(osr > 3 && osr < 8);
    });

    Ok(())
}

/// Configure frame format (stop bits, data bits)
fn configure_frame_format(info: &'static Info, config: &Config) {
    // Configure stop bits
    info.regs().baud().modify(|w| w.set_sbns(config.stop_bits_count));

    // Clear M10 for now (10-bit mode)
    info.regs().baud().modify(|w| w.set_m10(false));
}

/// Configure control settings (parity, data bits, idle config, pin swap)
fn configure_control_settings(info: &'static Info, config: &Config) {
    info.regs().ctrl().modify(|w| {
        // Parity configuration
        if let Some(parity) = config.parity_mode {
            w.set_pe(true);
            w.set_pt(parity);
        } else {
            w.set_pe(false);
        };

        // Data bits configuration
        match config.data_bits_count {
            DataBits::DATA8 => {
                if config.parity_mode.is_some() {
                    w.set_m(DataBits::DATA9); // 8 data + 1 parity = 9 bits
                } else {
                    w.set_m(DataBits::DATA8); // 8 data bits only
                }
            }
            DataBits::DATA9 => w.set_m(DataBits::DATA9),
        };

        // Idle configuration
        w.set_idlecfg(config.rx_idle_config);
        w.set_ilt(config.rx_idle_type);

        // Swap TXD/RXD if configured
        if config.swap_txd_rxd {
            w.set_swap(Swap::SWAP);
        } else {
            w.set_swap(Swap::STANDARD);
        }
    });
}

/// Configure FIFO settings and watermarks
fn configure_fifo(info: &'static Info, config: &Config) {
    // Configure WATER register for FIFO watermarks
    info.regs().water().write(|w| {
        w.set_rxwater(config.rx_fifo_watermark);
        w.set_txwater(config.tx_fifo_watermark);
    });

    // Enable TX/RX FIFOs
    info.regs().fifo().modify(|w| {
        w.set_txfe(true);
        w.set_rxfe(true);
    });

    // Flush FIFOs
    info.regs().fifo().modify(|w| {
        w.set_txflush(Txflush::TXFIFO_RST);
        w.set_rxflush(Rxflush::RXFIFO_RST);
    });
}

/// Clear all status flags
fn clear_all_status_flags(info: &'static Info) {
    info.regs().stat().modify(|_w| {
        // Write back all values, clearing the W1C fields implicitly.
    });
}

/// Configure hardware flow control if enabled
fn configure_flow_control(info: &'static Info, enable_tx_cts: bool, enable_rx_rts: bool, config: &Config) {
    if enable_rx_rts || enable_tx_cts {
        info.regs().modir().modify(|w| {
            w.set_txctsc(config.tx_cts_config);
            w.set_txctssrc(config.tx_cts_source);
            w.set_rxrtse(enable_rx_rts);
            w.set_txctse(enable_tx_cts);
        });
    }
}

/// Configure bit order (MSB first or LSB first)
fn configure_bit_order(info: &'static Info, msb_first: MsbFirst) {
    info.regs().stat().modify(|w| w.set_msbf(msb_first));
}

/// Enable transmitter and/or receiver based on configuration
fn enable_transceiver(info: &'static Info, enable_tx: bool, enable_rx: bool) {
    info.regs().ctrl().modify(|w| {
        if enable_tx {
            w.set_te(true);
        }
        if enable_rx {
            w.set_re(true);
        }
    });
}

fn calculate_baudrate(baudrate: u32, src_clock_hz: u32) -> Result<(u8, u16)> {
    let mut baud_diff = baudrate;
    let mut osr = 0u8;
    let mut sbr = 0u16;

    // Try OSR values from 4 to 32
    for osr_temp in 4u8..=32u8 {
        // Calculate SBR: (srcClock_Hz * 2 / (baudRate * osr) + 1) / 2
        let sbr_calc = ((src_clock_hz * 2) / (baudrate * osr_temp as u32)).div_ceil(2);

        let sbr_temp = if sbr_calc == 0 {
            1
        } else if sbr_calc > 0x1FFF {
            0x1FFF
        } else {
            sbr_calc as u16
        };

        // Calculate actual baud rate
        let calculated_baud = src_clock_hz / (osr_temp as u32 * sbr_temp as u32);

        let temp_diff = calculated_baud.abs_diff(baudrate);

        if temp_diff <= baud_diff {
            baud_diff = temp_diff;
            osr = osr_temp;
            sbr = sbr_temp;
        }
    }

    // Check if baud rate difference is within 3%
    if baud_diff > (baudrate / 100) * 3 {
        return Err(Error::UnsupportedBaudrate);
    }

    Ok((osr, sbr))
}

/// Wait for all transmit operations to complete
fn wait_for_tx_complete(info: &'static Info) {
    // Wait for TX FIFO to empty
    while info.regs().water().read().txcount() != 0 {
        // Wait for TX FIFO to drain
    }

    // Wait for last character to shift out (TC = Transmission Complete)
    while info.regs().stat().read().tc() == Tc::ACTIVE {
        // Wait for transmission to complete
    }
}

fn check_and_clear_rx_errors(info: &'static Info) -> Result<()> {
    let stat = info.regs().stat().read();
    let mut status = Ok(());

    // Check for overrun first - other error flags are prevented when OR is set
    if stat.or() {
        info.regs().stat().write(|w| w.set_or(true));

        return Err(Error::Overrun);
    }

    // Other errors are checked and cleared, but only 'most likely' error is returned.
    if stat.pf() {
        info.regs().stat().write(|w| w.set_pf(true));
        status = Err(Error::Parity);
    }

    if stat.fe() {
        info.regs().stat().write(|w| w.set_fe(true));
        status = Err(Error::Framing);
    }

    if stat.nf() {
        info.regs().stat().write(|w| w.set_nf(true));
        status = Err(Error::Noise);
    }

    status
}

fn has_data(info: &'static Info) -> bool {
    if info.regs().param().read().rxfifo() > 0 {
        // FIFO is available - check RXCOUNT in WATER register
        info.regs().water().read().rxcount() > 0
    } else {
        // No FIFO - check RDRF flag in STAT register
        info.regs().stat().read().rdrf()
    }
}

// ============================================================================
// PIN TRAITS FOR LPUART FUNCTIONALITY
// ============================================================================

impl<T: SealedPin> sealed::Sealed for T {}

/// io configuration trait for Lpuart Tx configuration
pub trait TxPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Lpuart Tx  usage
    fn as_tx(&self);
}

/// io configuration trait for Lpuart Rx configuration
pub trait RxPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Lpuart Rx  usage
    fn as_rx(&self);
}

/// io configuration trait for Lpuart Cts
pub trait CtsPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Lpuart Cts usage
    fn as_cts(&self);
}

/// io configuration trait for Lpuart Rts
pub trait RtsPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Lpuart Rts usage
    fn as_rts(&self);
}

macro_rules! impl_tx_pin {
    ($inst:ident, $pin:ident, $alt:ident) => {
        impl TxPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn as_tx(&self) {
                // TODO: Check these are right
                self.set_pull(crate::gpio::Pull::Up);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::vals::Mux::$alt);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

macro_rules! impl_rx_pin {
    ($inst:ident, $pin:ident, $alt:ident) => {
        impl RxPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn as_rx(&self) {
                // TODO: Check these are right
                self.set_pull(crate::gpio::Pull::Up);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::vals::Mux::$alt);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

// TODO: Macro and impls for CTS/RTS pins
macro_rules! impl_cts_pin {
    ($inst:ident, $pin:ident, $alt:ident) => {
        impl CtsPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn as_cts(&self) {
                todo!()
            }
        }
    };
}

macro_rules! impl_rts_pin {
    ($inst:ident, $pin:ident, $alt:ident) => {
        impl RtsPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn as_rts(&self) {
                todo!()
            }
        }
    };
}

// LPUART 0
#[cfg(feature = "jtag-extras-as-gpio")]
impl_tx_pin!(LPUART0, P0_3, MUX2);
impl_tx_pin!(LPUART0, P0_21, MUX3);
impl_tx_pin!(LPUART0, P2_1, MUX2);

#[cfg(feature = "swd-swo-as-gpio")]
impl_rx_pin!(LPUART0, P0_2, MUX2);
impl_rx_pin!(LPUART0, P0_20, MUX3);
impl_rx_pin!(LPUART0, P2_0, MUX2);

#[cfg(feature = "swd-as-gpio")]
impl_cts_pin!(LPUART0, P0_1, MUX2);
impl_cts_pin!(LPUART0, P0_23, MUX3);
impl_cts_pin!(LPUART0, P2_3, MUX2);

#[cfg(feature = "swd-as-gpio")]
impl_rts_pin!(LPUART0, P0_0, MUX2);
impl_rts_pin!(LPUART0, P0_22, MUX3);
impl_rts_pin!(LPUART0, P2_2, MUX2);

// LPUART 1
impl_tx_pin!(LPUART1, P1_9, MUX2);
impl_tx_pin!(LPUART1, P2_13, MUX3);
impl_tx_pin!(LPUART1, P3_9, MUX3);
impl_tx_pin!(LPUART1, P3_21, MUX3);

impl_rx_pin!(LPUART1, P1_8, MUX2);
impl_rx_pin!(LPUART1, P2_12, MUX3);
impl_rx_pin!(LPUART1, P3_8, MUX3);
impl_rx_pin!(LPUART1, P3_20, MUX3);

impl_cts_pin!(LPUART1, P1_11, MUX2);
impl_cts_pin!(LPUART1, P2_17, MUX3);
impl_cts_pin!(LPUART1, P3_11, MUX3);
impl_cts_pin!(LPUART1, P3_23, MUX3);

impl_rts_pin!(LPUART1, P1_10, MUX2);
impl_rts_pin!(LPUART1, P2_15, MUX3);
impl_rts_pin!(LPUART1, P2_16, MUX3);
impl_rts_pin!(LPUART1, P3_10, MUX3);

// LPUART 2
impl_tx_pin!(LPUART2, P1_5, MUX3);
impl_tx_pin!(LPUART2, P1_13, MUX3);
impl_tx_pin!(LPUART2, P2_2, MUX3);
impl_tx_pin!(LPUART2, P2_10, MUX3);
impl_tx_pin!(LPUART2, P3_15, MUX2);

impl_rx_pin!(LPUART2, P1_4, MUX3);
impl_rx_pin!(LPUART2, P1_12, MUX3);
impl_rx_pin!(LPUART2, P2_3, MUX3);
impl_rx_pin!(LPUART2, P2_11, MUX3);
impl_rx_pin!(LPUART2, P3_14, MUX2);

impl_cts_pin!(LPUART2, P1_7, MUX3);
impl_cts_pin!(LPUART2, P1_15, MUX3);
impl_cts_pin!(LPUART2, P2_4, MUX3);
impl_cts_pin!(LPUART2, P3_13, MUX2);

impl_rts_pin!(LPUART2, P1_6, MUX3);
impl_rts_pin!(LPUART2, P1_14, MUX3);
impl_rts_pin!(LPUART2, P2_5, MUX3);
impl_rts_pin!(LPUART2, P3_12, MUX2);

// LPUART 3
impl_tx_pin!(LPUART3, P3_1, MUX3);
impl_tx_pin!(LPUART3, P3_12, MUX3);
impl_tx_pin!(LPUART3, P4_5, MUX3);

impl_rx_pin!(LPUART3, P3_0, MUX3);
impl_rx_pin!(LPUART3, P3_13, MUX3);
impl_rx_pin!(LPUART3, P4_2, MUX3);

impl_cts_pin!(LPUART3, P3_7, MUX3);
impl_cts_pin!(LPUART3, P3_14, MUX3);
impl_cts_pin!(LPUART3, P4_6, MUX3);

impl_rts_pin!(LPUART3, P3_6, MUX3);
impl_rts_pin!(LPUART3, P3_15, MUX3);
impl_rts_pin!(LPUART3, P4_7, MUX3);

// LPUART 4
impl_tx_pin!(LPUART4, P2_7, MUX3);
impl_tx_pin!(LPUART4, P3_19, MUX2);
impl_tx_pin!(LPUART4, P3_27, MUX3);
impl_tx_pin!(LPUART4, P4_3, MUX3);

impl_rx_pin!(LPUART4, P2_6, MUX3);
impl_rx_pin!(LPUART4, P3_18, MUX2);
impl_rx_pin!(LPUART4, P3_28, MUX3);
impl_rx_pin!(LPUART4, P4_4, MUX3);

impl_cts_pin!(LPUART4, P2_0, MUX3);
impl_cts_pin!(LPUART4, P3_17, MUX2);
impl_cts_pin!(LPUART4, P3_31, MUX3);

impl_rts_pin!(LPUART4, P2_1, MUX3);
impl_rts_pin!(LPUART4, P3_16, MUX2);
impl_rts_pin!(LPUART4, P3_30, MUX3);

// LPUART 5
impl_tx_pin!(LPUART5, P1_10, MUX8);
impl_tx_pin!(LPUART5, P1_17, MUX8);

impl_rx_pin!(LPUART5, P1_11, MUX8);
impl_rx_pin!(LPUART5, P1_16, MUX8);

impl_cts_pin!(LPUART5, P1_12, MUX8);
impl_cts_pin!(LPUART5, P1_19, MUX8);

impl_rts_pin!(LPUART5, P1_13, MUX8);
impl_rts_pin!(LPUART5, P1_18, MUX8);

// ============================================================================
// ERROR TYPES AND RESULTS
// ============================================================================

/// LPUART error types
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Read error
    Read,
    /// Buffer overflow
    Overrun,
    /// Noise error
    Noise,
    /// Framing error
    Framing,
    /// Parity error
    Parity,
    /// Failure
    Fail,
    /// Invalid argument
    InvalidArgument,
    /// Lpuart baud rate cannot be supported with the given clock
    UnsupportedBaudrate,
    /// RX FIFO Empty
    RxFifoEmpty,
    /// TX FIFO Full
    TxFifoFull,
    /// TX Busy
    TxBusy,
    /// Clock Error
    ClockSetup(ClockError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Read => write!(f, "Read error"),
            Error::Overrun => write!(f, "Buffer overflow"),
            Error::Noise => write!(f, "Noise error"),
            Error::Framing => write!(f, "Framing error"),
            Error::Parity => write!(f, "Parity error"),
            Error::Fail => write!(f, "Failure"),
            Error::InvalidArgument => write!(f, "Invalid argument"),
            Error::UnsupportedBaudrate => write!(f, "Unsupported baud rate"),
            Error::RxFifoEmpty => write!(f, "RX FIFO empty"),
            Error::TxFifoFull => write!(f, "TX FIFO full"),
            Error::TxBusy => write!(f, "TX busy"),
            Error::ClockSetup(e) => write!(f, "Clock setup error: {:?}", e),
        }
    }
}

impl core::error::Error for Error {}

/// A specialized Result type for LPUART operations
pub type Result<T> = core::result::Result<T, Error>;

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Lpuart config
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: LpuartClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Baud rate in bits per second
    pub baudrate_bps: u32,
    /// Parity configuration
    pub parity_mode: Option<Parity>,
    /// Number of data bits
    pub data_bits_count: DataBits,
    /// MSB First or LSB First configuration
    pub msb_first: MsbFirst,
    /// Number of stop bits
    pub stop_bits_count: StopBits,
    /// TX FIFO watermark
    pub tx_fifo_watermark: u8,
    /// RX FIFO watermark
    pub rx_fifo_watermark: u8,
    /// TX CTS source
    pub tx_cts_source: TxCtsSource,
    /// TX CTS configure
    pub tx_cts_config: TxCtsConfig,
    /// RX IDLE type
    pub rx_idle_type: IdleType,
    /// RX IDLE configuration
    pub rx_idle_config: IdleConfig,
    /// Swap TXD and RXD pins
    pub swap_txd_rxd: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate_bps: 115_200u32,
            parity_mode: None,
            data_bits_count: DataBits::DATA8,
            msb_first: MsbFirst::LSB_FIRST,
            stop_bits_count: StopBits::ONE,
            tx_fifo_watermark: 0,
            rx_fifo_watermark: 1,
            tx_cts_source: TxCtsSource::CTS,
            tx_cts_config: TxCtsConfig::START,
            rx_idle_type: IdleType::FROM_START,
            rx_idle_config: IdleConfig::IDLE_1,
            swap_txd_rxd: false,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpuartClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

/// LPUART status flags
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Transmit data register empty
    pub tx_empty: bool,
    /// Transmission complete
    pub tx_complete: bool,
    /// Receive data register full
    pub rx_full: bool,
    /// Idle line detected
    pub idle: bool,
    /// Receiver overrun
    pub overrun: bool,
    /// Noise error
    pub noise: bool,
    /// Framing error
    pub framing: bool,
    /// Parity error
    pub parity: bool,
}

// ============================================================================
// MODE TRAITS (BLOCKING/ASYNC)
// ============================================================================

/// Driver move trait.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

// ============================================================================
// CORE DRIVER STRUCTURES
// ============================================================================

/// Lpuart driver.
pub struct Lpuart<'a, M: Mode> {
    info: &'static Info,
    tx: LpuartTx<'a, M>,
    rx: LpuartRx<'a, M>,
}

/// Lpuart TX driver.
pub struct LpuartTx<'a, M: Mode> {
    info: &'static Info,
    _tx_pin: Peri<'a, AnyPin>,
    _cts_pin: Option<Peri<'a, AnyPin>>,
    mode: PhantomData<(&'a (), M)>,
    _wg: Option<WakeGuard>,
}

/// Lpuart Rx driver.
pub struct LpuartRx<'a, M: Mode> {
    info: &'static Info,
    _rx_pin: Peri<'a, AnyPin>,
    _rts_pin: Option<Peri<'a, AnyPin>>,
    mode: PhantomData<(&'a (), M)>,
    _wg: Option<WakeGuard>,
}

/// Lpuart TX driver with DMA support.
pub struct LpuartTxDma<'a, T: Instance, C: DmaChannelTrait> {
    info: &'static Info,
    _tx_pin: Peri<'a, AnyPin>,
    tx_dma: DmaChannel<C>,
    _instance: core::marker::PhantomData<T>,
    _wg: Option<WakeGuard>,
}

/// Lpuart RX driver with DMA support.
pub struct LpuartRxDma<'a, T: Instance, C: DmaChannelTrait> {
    info: &'static Info,
    _rx_pin: Peri<'a, AnyPin>,
    rx_dma: DmaChannel<C>,
    _instance: core::marker::PhantomData<T>,
    _wg: Option<WakeGuard>,
}

/// Lpuart driver with DMA support for both TX and RX.
pub struct LpuartDma<'a, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> {
    tx: LpuartTxDma<'a, T, TxC>,
    rx: LpuartRxDma<'a, T, RxC>,
}

/// Lpuart RX driver with ring-buffered DMA support.
pub struct LpuartRxRingDma<'peri, 'ring, T: Instance, C: DmaChannelTrait> {
    _inner: LpuartRxDma<'peri, T, C>,
    ring: RingBuffer<'ring, u8>,
}

// ============================================================================
// LPUART CORE IMPLEMENTATION
// ============================================================================

impl<'a, M: Mode> Lpuart<'a, M> {
    fn init<T: Instance>(
        enable_tx: bool,
        enable_rx: bool,
        enable_tx_cts: bool,
        enable_rx_rts: bool,
        config: Config,
    ) -> Result<Option<WakeGuard>> {
        // Enable clocks
        let conf = LpuartConfig {
            power: config.power,
            source: config.source,
            div: config.div,
            instance: T::CLOCK_INSTANCE,
        };
        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        // Perform initialization sequence
        perform_software_reset(T::info());
        disable_transceiver(T::info());
        configure_baudrate(T::info(), config.baudrate_bps, parts.freq)?;
        configure_frame_format(T::info(), &config);
        configure_control_settings(T::info(), &config);
        configure_fifo(T::info(), &config);
        clear_all_status_flags(T::info());
        configure_flow_control(T::info(), enable_tx_cts, enable_rx_rts, &config);
        configure_bit_order(T::info(), config.msb_first);
        enable_transceiver(T::info(), enable_rx, enable_tx);

        Ok(parts.wake_guard)
    }

    /// Deinitialize the LPUART peripheral
    pub fn deinit(&self) -> Result<()> {
        // Wait for TX operations to complete
        wait_for_tx_complete(self.info);

        // Clear all status flags
        clear_all_status_flags(self.info);

        // Disable the module - clear all CTRL register bits
        self.info.regs().ctrl().write(|w| w.0 = 0);

        Ok(())
    }

    /// Split the Lpuart into a transmitter and receiver
    pub fn split(self) -> (LpuartTx<'a, M>, LpuartRx<'a, M>) {
        (self.tx, self.rx)
    }

    /// Split the Lpuart into a transmitter and receiver by mutable reference
    pub fn split_ref(&mut self) -> (&mut LpuartTx<'a, M>, &mut LpuartRx<'a, M>) {
        (&mut self.tx, &mut self.rx)
    }
}

// ============================================================================
// BLOCKING MODE IMPLEMENTATIONS
// ============================================================================

impl<'a> Lpuart<'a, Blocking> {
    /// Create a new blocking LPUART instance with RX/TX pins.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        // Configure the pins for LPUART usage
        tx_pin.as_tx();
        rx_pin.as_rx();

        // Initialize the peripheral
        let _wg = Self::init::<T>(true, true, false, false, config)?;

        Ok(Self {
            info: T::info(),
            tx: LpuartTx::new_inner(T::info(), tx_pin.into(), None, _wg.clone()),
            rx: LpuartRx::new_inner(T::info(), rx_pin.into(), None, _wg),
        })
    }

    /// Create a new blocking LPUART instance with RX, TX and RTS/CTS flow control pins.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_rtscts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        // Configure the pins for LPUART usage
        rx_pin.as_rx();
        tx_pin.as_tx();
        rts_pin.as_rts();
        cts_pin.as_cts();

        // Initialize the peripheral with flow control
        let _wg = Self::init::<T>(true, true, true, true, config)?;

        Ok(Self {
            info: T::info(),
            rx: LpuartRx::new_inner(T::info(), rx_pin.into(), Some(rts_pin.into()), _wg.clone()),
            tx: LpuartTx::new_inner(T::info(), tx_pin.into(), Some(cts_pin.into()), _wg),
        })
    }
}

// ----------------------------------------------------------------------------
// Blocking TX Implementation
// ----------------------------------------------------------------------------

impl<'a, M: Mode> LpuartTx<'a, M> {
    fn new_inner(
        info: &'static Info,
        tx_pin: Peri<'a, AnyPin>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        _wg: Option<WakeGuard>,
    ) -> Self {
        Self {
            info,
            _tx_pin: tx_pin,
            _cts_pin: cts_pin,
            mode: PhantomData,
            _wg,
        }
    }
}

impl<'a> LpuartTx<'a, Blocking> {
    /// Create a new blocking LPUART transmitter instance.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        // Configure the pins for LPUART usage
        tx_pin.as_tx();

        // Initialize the peripheral
        let _wg = Lpuart::<Blocking>::init::<T>(true, false, false, false, config)?;

        Ok(Self::new_inner(T::info(), tx_pin.into(), None, _wg))
    }

    /// Create a new blocking LPUART transmitter instance with CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        cts_pin.as_cts();

        let _wg = Lpuart::<Blocking>::init::<T>(true, false, true, false, config)?;

        Ok(Self::new_inner(T::info(), tx_pin.into(), Some(cts_pin.into()), _wg))
    }

    fn write_byte_internal(&mut self, byte: u8) -> Result<()> {
        self.info.regs().data().modify(|w| w.0 = u32::from(byte));

        Ok(())
    }

    fn blocking_write_byte(&mut self, byte: u8) -> Result<()> {
        while self.info.regs().stat().read().tdre() == Tdre::TXDATA {}
        self.write_byte_internal(byte)
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.info.regs().stat().read().tdre() == Tdre::TXDATA {
            Err(Error::TxFifoFull)
        } else {
            self.write_byte_internal(byte)
        }
    }

    /// Write data to LPUART TX blocking execution until all data is sent.
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        for x in buf {
            self.blocking_write_byte(*x)?;
        }

        Ok(())
    }

    pub fn write_str_blocking(&mut self, buf: &str) {
        let _ = self.blocking_write(buf.as_bytes());
    }

    /// Write data to LPUART TX without blocking.
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        for x in buf {
            self.write_byte(*x)?;
        }

        Ok(())
    }

    /// Flush LPUART TX blocking execution until all data has been transmitted.
    pub fn blocking_flush(&mut self) -> Result<()> {
        while self.info.regs().water().read().txcount() != 0 {
            // Wait for TX FIFO to drain
        }

        // Wait for last character to shift out
        while self.info.regs().stat().read().tc() == Tc::ACTIVE {
            // Wait for transmission to complete
        }

        Ok(())
    }

    /// Flush LPUART TX.
    pub fn flush(&mut self) -> Result<()> {
        // Check if TX FIFO is empty
        if self.info.regs().water().read().txcount() != 0 {
            return Err(Error::TxBusy);
        }

        // Check if transmission is complete
        if self.info.regs().stat().read().tc() == Tc::ACTIVE {
            return Err(Error::TxBusy);
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Blocking RX Implementation
// ----------------------------------------------------------------------------

impl<'a, M: Mode> LpuartRx<'a, M> {
    fn new_inner(
        info: &'static Info,
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        _wg: Option<WakeGuard>,
    ) -> Self {
        Self {
            info,
            _rx_pin: rx_pin,
            _rts_pin: rts_pin,
            mode: PhantomData,
            _wg,
        }
    }
}

impl<'a> LpuartRx<'a, Blocking> {
    /// Create a new blocking LPUART Receiver instance.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();

        let _wg = Lpuart::<Blocking>::init::<T>(false, true, false, false, config)?;

        Ok(Self::new_inner(T::info(), rx_pin.into(), None, _wg))
    }

    /// Create a new blocking LPUART Receiver instance with RTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();
        rts_pin.as_rts();

        let _wg = Lpuart::<Blocking>::init::<T>(false, true, false, true, config)?;

        Ok(Self::new_inner(T::info(), rx_pin.into(), Some(rts_pin.into()), _wg))
    }

    fn read_byte_internal(&mut self) -> Result<u8> {
        Ok((self.info.regs().data().read().0 & 0xFF) as u8)
    }

    fn read_byte(&mut self) -> Result<u8> {
        check_and_clear_rx_errors(self.info)?;

        if !has_data(self.info) {
            return Err(Error::RxFifoEmpty);
        }

        self.read_byte_internal()
    }

    fn blocking_read_byte(&mut self) -> Result<u8> {
        loop {
            if has_data(self.info) {
                return self.read_byte_internal();
            }

            check_and_clear_rx_errors(self.info)?;
        }
    }

    /// Read data from LPUART RX without blocking.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            *byte = self.read_byte()?;
        }
        Ok(())
    }

    /// Read data from LPUART RX blocking execution until the buffer is filled.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            *byte = self.blocking_read_byte()?;
        }
        Ok(())
    }
}

impl<'a> Lpuart<'a, Blocking> {
    /// Read data from LPUART RX blocking execution until the buffer is filled
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.rx.blocking_read(buf)
    }

    /// Read data from LPUART RX without blocking
    pub fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.rx.read(buf)
    }

    /// Write data to LPUART TX blocking execution until all data is sent
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        self.tx.blocking_write(buf)
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.tx.write_byte(byte)
    }

    pub fn read_byte_blocking(&mut self) -> u8 {
        loop {
            if let Ok(b) = self.rx.read_byte() {
                return b;
            }
        }
    }

    pub fn write_str_blocking(&mut self, buf: &str) {
        self.tx.write_str_blocking(buf);
    }

    /// Write data to LPUART TX without blocking
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.tx.write(buf)
    }

    /// Flush LPUART TX blocking execution until all data has been transmitted
    pub fn blocking_flush(&mut self) -> Result<()> {
        self.tx.blocking_flush()
    }

    /// Flush LPUART TX without blocking
    pub fn flush(&mut self) -> Result<()> {
        self.tx.flush()
    }
}

// ============================================================================
// ASYNC MODE IMPLEMENTATIONS (DMA-based)
// ============================================================================

/// Guard struct that ensures DMA is stopped if the async future is cancelled.
///
/// This implements the RAII pattern: if the future is dropped before completion
/// (e.g., due to a timeout), the DMA transfer is automatically aborted to prevent
/// use-after-free when the buffer goes out of scope.
struct TxDmaGuard<'a, C: DmaChannelTrait> {
    dma: &'a DmaChannel<C>,
    info: &'static Info,
}

impl<'a, C: DmaChannelTrait> TxDmaGuard<'a, C> {
    fn new(dma: &'a DmaChannel<C>, info: &'static Info) -> Self {
        Self { dma, info }
    }

    /// Complete the transfer normally (don't abort on drop).
    fn complete(self) {
        // Cleanup
        self.info.regs().baud().modify(|w| w.set_tdmae(false));
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
        }
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl<C: DmaChannelTrait> Drop for TxDmaGuard<'_, C> {
    fn drop(&mut self) {
        // Abort the DMA transfer if still running
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
            self.dma.clear_interrupt();
        }
        // Disable UART TX DMA request
        self.info.regs().baud().modify(|w| w.set_tdmae(false));
    }
}

/// Guard struct for RX DMA transfers.
struct RxDmaGuard<'a, C: DmaChannelTrait> {
    dma: &'a DmaChannel<C>,
    info: &'static Info,
}

impl<'a, C: DmaChannelTrait> RxDmaGuard<'a, C> {
    fn new(dma: &'a DmaChannel<C>, info: &'static Info) -> Self {
        Self { dma, info }
    }

    /// Complete the transfer normally (don't abort on drop).
    fn complete(self) {
        // Ensure DMA writes are visible to CPU
        cortex_m::asm::dsb();
        // Cleanup
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
        }
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl<C: DmaChannelTrait> Drop for RxDmaGuard<'_, C> {
    fn drop(&mut self) {
        // Abort the DMA transfer if still running
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
            self.dma.clear_interrupt();
        }
        // Disable UART RX DMA request
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
    }
}

impl<'a, T: Instance, C: DmaChannelTrait> LpuartTxDma<'a, T, C> {
    /// Create a new LPUART TX driver with DMA support.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        tx_dma_ch: Peri<'a, C>,
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();

        // Initialize LPUART with TX enabled, RX disabled, no flow control
        let _wg = Lpuart::<Async>::init::<T>(true, false, false, false, config)?;

        // Enable interrupt
        let tx_dma = DmaChannel::new(tx_dma_ch);
        tx_dma.enable_interrupt();

        Ok(Self {
            info: T::info(),
            _tx_pin: tx_pin,
            tx_dma,
            _instance: core::marker::PhantomData,
            _wg,
        })
    }

    /// Write data using DMA.
    ///
    /// This configures the DMA channel for a memory-to-peripheral transfer
    /// and waits for completion asynchronously. Large buffers are automatically
    /// split into chunks that fit within the DMA transfer limit.
    ///
    /// The DMA request source is automatically derived from the LPUART instance type.
    ///
    /// # Safety
    ///
    /// If the returned future is dropped before completion (e.g., due to a timeout),
    /// the DMA transfer is automatically aborted to prevent use-after-free.
    ///
    /// # Arguments
    /// * `buf` - Data buffer to transmit
    pub async fn write_dma(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks(DMA_MAX_TRANSFER_SIZE) {
            total += self.write_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Internal helper to write a single chunk (max 0x7FFF bytes) using DMA.
    async fn write_dma_inner(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        let peri_addr = self.info.regs().data().as_ptr() as *mut u8;

        unsafe {
            // Clean up channel state
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.tx_dma.set_request_source::<T::TxDmaRequest>();

            // Configure TCD for memory-to-peripheral transfer
            self.tx_dma
                .setup_write_to_peripheral(buf, peri_addr, EnableInterrupt::Yes);

            // Enable UART TX DMA request
            self.info.regs().baud().modify(|w| w.set_tdmae(true));

            // Enable DMA channel request
            self.tx_dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = TxDmaGuard::new(&self.tx_dma, self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            self.tx_dma.waker().register(cx.waker());
            if self.tx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking write (fallback when DMA is not needed)
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        for &byte in buf {
            while self.info.regs().stat().read().tdre() == Tdre::TXDATA {}
            self.info.regs().data().write(|w| w.0 = byte as u32);
        }
        Ok(())
    }

    /// Flush TX blocking
    pub fn blocking_flush(&mut self) -> Result<()> {
        while self.info.regs().water().read().txcount() != 0 {}
        while self.info.regs().stat().read().tc() == Tc::ACTIVE {}
        Ok(())
    }
}

impl<'a, T: Instance, C: DmaChannelTrait> LpuartRxDma<'a, T, C> {
    /// Create a new LPUART RX driver with DMA support.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rx_dma_ch: Peri<'a, C>,
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        // Initialize LPUART with TX disabled, RX enabled, no flow control
        let _wg = Lpuart::<Async>::init::<T>(false, true, false, false, config)?;

        // Enable dma interrupt
        let rx_dma = DmaChannel::new(rx_dma_ch);
        rx_dma.enable_interrupt();

        Ok(Self {
            info: T::info(),
            _rx_pin: rx_pin,
            rx_dma,
            _instance: core::marker::PhantomData,
            _wg,
        })
    }

    /// Read data using DMA.
    ///
    /// This configures the DMA channel for a peripheral-to-memory transfer
    /// and waits for completion asynchronously. Large buffers are automatically
    /// split into chunks that fit within the DMA transfer limit.
    ///
    /// The DMA request source is automatically derived from the LPUART instance type.
    ///
    /// # Safety
    ///
    /// If the returned future is dropped before completion (e.g., due to a timeout),
    /// the DMA transfer is automatically aborted to prevent use-after-free.
    ///
    /// # Arguments
    /// * `buf` - Buffer to receive data into
    pub async fn read_dma(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            total += self.read_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Internal helper to read a single chunk (max 0x7FFF bytes) using DMA.
    async fn read_dma_inner(&mut self, buf: &mut [u8]) -> Result<usize> {
        let len = buf.len();
        let peri_addr = self.info.regs().data().as_ptr() as *const u8;

        unsafe {
            // Clean up channel state
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // Configure TCD for peripheral-to-memory transfer
            self.rx_dma
                .setup_read_from_peripheral(peri_addr, buf, EnableInterrupt::Yes);

            // Enable UART RX DMA request
            self.info.regs().baud().modify(|w| w.set_rdmae(true));

            // Enable DMA channel request
            self.rx_dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = RxDmaGuard::new(&self.rx_dma, self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking read (fallback when DMA is not needed)
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            loop {
                if has_data(self.info) {
                    *byte = (self.info.regs().data().read().0 & 0xFF) as u8;
                    break;
                }
                check_and_clear_rx_errors(self.info)?;
            }
        }
        Ok(())
    }

    pub fn into_ring_dma_rx<'buf>(self, buf: &'buf mut [u8]) -> LpuartRxRingDma<'a, 'buf, T, C> {
        unsafe {
            let ring = self.setup_ring_buffer(buf);
            self.enable_dma_request();
            LpuartRxRingDma { _inner: self, ring }
        }
    }

    /// Set up a ring buffer for continuous DMA reception.
    ///
    /// This configures the DMA channel for circular operation, enabling continuous
    /// reception of data without gaps. The DMA will continuously write received
    /// bytes into the buffer, wrapping around when it reaches the end.
    ///
    /// This method encapsulates all the low-level setup:
    /// - Configures the DMA request source for this LPUART instance
    /// - Enables the RX DMA request in the LPUART peripheral
    /// - Sets up the circular DMA transfer
    /// - Enables the NVIC interrupt for async wakeups
    ///
    /// # Arguments
    ///
    /// * `buf` - Destination buffer for received data (power-of-2 size is ideal for efficiency)
    ///
    /// # Returns
    ///
    /// A [`RingBuffer`] that can be used to asynchronously read received data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// static mut RX_BUF: [u8; 64] = [0; 64];
    ///
    /// let rx = LpuartRxDma::new(p.LPUART2, p.P2_3, p.DMA_CH0, config).unwrap();
    /// let ring_buf = unsafe { rx.setup_ring_buffer(&mut RX_BUF) };
    ///
    /// // Read data as it arrives
    /// let mut buf = [0u8; 16];
    /// let n = ring_buf.read(&mut buf).await.unwrap();
    /// ```
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the lifetime of the returned RingBuffer.
    /// - Only one RingBuffer should exist per LPUART RX channel at a time.
    /// - The caller must ensure the static buffer is not accessed elsewhere while
    ///   the ring buffer is active.
    unsafe fn setup_ring_buffer<'b>(&self, buf: &'b mut [u8]) -> RingBuffer<'b, u8> {
        unsafe {
            // Get the peripheral data register address
            let peri_addr = self.info.regs().data().as_ptr() as *const u8;

            // Configure DMA request source for this LPUART instance (type-safe)
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // Enable RX DMA request in the LPUART peripheral
            self.info.regs().baud().modify(|w| w.set_rdmae(true));

            // Set up circular DMA transfer (this also enables NVIC interrupt)
            self.rx_dma.setup_circular_read(peri_addr, buf)
        }
    }

    /// Enable the DMA channel request.
    ///
    /// Call this after `setup_ring_buffer()` to start continuous reception.
    /// This is separated from setup to allow for any additional configuration
    /// before starting the transfer.
    unsafe fn enable_dma_request(&self) {
        unsafe {
            self.rx_dma.enable_request();
        }
    }
}

impl<'peri, 'buf, T: Instance, C: DmaChannelTrait> LpuartRxRingDma<'peri, 'buf, T, C> {
    /// Read from the ring buffer
    pub fn read<'d>(
        &mut self,
        dst: &'d mut [u8],
    ) -> impl Future<Output = core::result::Result<usize, crate::dma::Error>> + use<'_, 'buf, 'd, T, C> {
        self.ring.read(dst)
    }

    /// Clear the current contents of the ring buffer
    pub fn clear(&mut self) {
        self.ring.clear();
    }
}

impl<'a, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> LpuartDma<'a, T, TxC, RxC> {
    /// Create a new LPUART driver with DMA support for both TX and RX.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        tx_dma_ch: Peri<'a, TxC>,
        rx_dma_ch: Peri<'a, RxC>,
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();

        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        // Initialize LPUART with both TX and RX enabled, no flow control
        let _wg = Lpuart::<Async>::init::<T>(true, true, false, false, config)?;

        // Enable DMA interrupts
        let tx_dma = DmaChannel::new(tx_dma_ch);
        let rx_dma = DmaChannel::new(rx_dma_ch);
        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        Ok(Self {
            tx: LpuartTxDma {
                info: T::info(),
                _tx_pin: tx_pin,
                tx_dma,
                _instance: core::marker::PhantomData,
                _wg: _wg.clone(),
            },
            rx: LpuartRxDma {
                info: T::info(),
                _rx_pin: rx_pin,
                rx_dma,
                _instance: core::marker::PhantomData,
                _wg,
            },
        })
    }

    /// Split into separate TX and RX drivers
    pub fn split(self) -> (LpuartTxDma<'a, T, TxC>, LpuartRxDma<'a, T, RxC>) {
        (self.tx, self.rx)
    }

    /// Write data using DMA
    pub async fn write_dma(&mut self, buf: &[u8]) -> Result<usize> {
        self.tx.write_dma(buf).await
    }

    /// Read data using DMA
    pub async fn read_dma(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.rx.read_dma(buf).await
    }
}

// ============================================================================
// DROP TRAIT IMPLEMENTATIONS
// ============================================================================

impl<'a, M: Mode> Drop for LpuartTx<'a, M> {
    fn drop(&mut self) {
        self._tx_pin.set_as_disabled();
        if let Some(cts_pin) = &self._cts_pin {
            cts_pin.set_as_disabled();
        }
    }
}

impl<'a, M: Mode> Drop for LpuartRx<'a, M> {
    fn drop(&mut self) {
        self._rx_pin.set_as_disabled();
        if let Some(rts_pin) = &self._rts_pin {
            rts_pin.set_as_disabled();
        }
    }
}

impl<'a, T: Instance, C: DmaChannelTrait> Drop for LpuartTxDma<'a, T, C> {
    fn drop(&mut self) {
        self._tx_pin.set_as_disabled();
    }
}

impl<'a, T: Instance, C: DmaChannelTrait> Drop for LpuartRxDma<'a, T, C> {
    fn drop(&mut self) {
        self._rx_pin.set_as_disabled();
    }
}

// ============================================================================
// EMBEDDED-IO-ASYNC TRAIT IMPLEMENTATIONS
// ============================================================================

impl<T: Instance, C: DmaChannelTrait> embedded_io::ErrorType for LpuartTxDma<'_, T, C> {
    type Error = Error;
}

impl<T: Instance, C: DmaChannelTrait> embedded_io::ErrorType for LpuartRxDma<'_, T, C> {
    type Error = Error;
}

impl<T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> embedded_io::ErrorType for LpuartDma<'_, T, TxC, RxC> {
    type Error = Error;
}

// ============================================================================
// EMBEDDED-HAL 0.2 TRAIT IMPLEMENTATIONS
// ============================================================================

impl embedded_hal_02::serial::Read<u8> for LpuartRx<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        let mut buf = [0; 1];
        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::serial::Write<u8> for LpuartTx<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for LpuartTx<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_02::serial::Read<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_02::serial::Write<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::write(&mut self.tx, word)
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::flush(&mut self.tx)
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

// ============================================================================
// EMBEDDED-HAL-NB TRAIT IMPLEMENTATIONS
// ============================================================================

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match *self {
            Self::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Self::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
            _ => embedded_hal_nb::serial::ErrorKind::Other,
        }
    }
}

impl embedded_hal_nb::serial::ErrorType for LpuartRx<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for LpuartTx<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for Lpuart<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::Read for LpuartRx<'_, Blocking> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf = [0; 1];
        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_nb::serial::Write for LpuartTx<'_, Blocking> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl core::fmt::Write for LpuartTx<'_, Blocking> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

impl embedded_hal_nb::serial::Read for Lpuart<'_, Blocking> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        embedded_hal_nb::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_nb::serial::Write for Lpuart<'_, Blocking> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        embedded_hal_nb::serial::Write::write(&mut self.tx, char)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        embedded_hal_nb::serial::Write::flush(&mut self.tx)
    }
}

// ============================================================================
// EMBEDDED-IO TRAIT IMPLEMENTATIONS
// ============================================================================

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl embedded_io::ErrorType for LpuartRx<'_, Blocking> {
    type Error = Error;
}

impl embedded_io::ErrorType for LpuartTx<'_, Blocking> {
    type Error = Error;
}

impl embedded_io::ErrorType for Lpuart<'_, Blocking> {
    type Error = Error;
}

impl embedded_io::Read for LpuartRx<'_, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl embedded_io::Write for LpuartTx<'_, Blocking> {
    fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_io::Read for Lpuart<'_, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        embedded_io::Read::read(&mut self.rx, buf)
    }
}

impl embedded_io::Write for Lpuart<'_, Blocking> {
    fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        embedded_io::Write::write(&mut self.tx, buf)
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        embedded_io::Write::flush(&mut self.tx)
    }
}
