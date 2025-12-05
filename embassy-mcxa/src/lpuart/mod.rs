use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use paste::paste;

use crate::clocks::periph_helpers::{Div4, LpuartClockSel, LpuartConfig};
use crate::clocks::{ClockError, Gate, PoweredClock, enable_and_reset};
use crate::gpio::SealedPin;
use crate::pac::lpuart0::baud::Sbns as StopBits;
use crate::pac::lpuart0::ctrl::{Idlecfg as IdleConfig, Ilt as IdleType, M as DataBits, Pt as Parity};
use crate::pac::lpuart0::modir::{Txctsc as TxCtsConfig, Txctssrc as TxCtsSource};
use crate::pac::lpuart0::stat::Msbf as MsbFirst;
use crate::{AnyPin, interrupt, pac};

pub mod buffered;

// ============================================================================
// STUB IMPLEMENTATION
// ============================================================================

// Stub implementation for LIB (Peripherals), GPIO, DMA and CLOCK until stable API
// Pin and Clock initialization is currently done at the examples level.

// --- START DMA ---
mod dma {
    pub struct Channel<'d> {
        pub(super) _lifetime: core::marker::PhantomData<&'d ()>,
    }
}

use dma::Channel;

// --- END DMA ---

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

pub type Regs = &'static crate::pac::lpuart0::RegisterBlock;

pub trait SealedInstance {
    fn info() -> Info;
    fn index() -> usize;
    fn buffered_state() -> &'static buffered::State;
}

pub struct Info {
    pub regs: Regs,
}

/// Trait for LPUART peripheral instances
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpuartConfig> {
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpuartInstance;
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPUART $n>] {
                    fn info() -> Info {
                        Info {
                            regs: unsafe { &*pac::[<Lpuart $n>]::ptr() },
                        }
                    }

                    #[inline]
                    fn index() -> usize {
                        $n
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
                }
            }
        )*
    };
}

impl_instance!(0, 1, 2, 3, 4, 5);

// ============================================================================
// INSTANCE HELPER FUNCTIONS
// ============================================================================

/// Perform software reset on the LPUART peripheral
pub fn perform_software_reset(regs: Regs) {
    // Software reset - set and clear RST bit (Global register)
    regs.global().write(|w| w.rst().reset());
    regs.global().write(|w| w.rst().no_effect());
}

/// Disable both transmitter and receiver
pub fn disable_transceiver(regs: Regs) {
    regs.ctrl().modify(|_, w| w.te().disabled().re().disabled());
}

/// Calculate and configure baudrate settings
pub fn configure_baudrate(regs: Regs, baudrate_bps: u32, clock_freq: u32) -> Result<()> {
    let (osr, sbr) = calculate_baudrate(baudrate_bps, clock_freq)?;

    // Configure BAUD register
    regs.baud().modify(|_, w| unsafe {
        // Clear and set OSR
        w.osr().bits(osr - 1);
        // Clear and set SBR
        w.sbr().bits(sbr);
        // Set BOTHEDGE if OSR is between 4 and 7
        if osr > 3 && osr < 8 {
            w.bothedge().enabled()
        } else {
            w.bothedge().disabled()
        }
    });

    Ok(())
}

/// Configure frame format (stop bits, data bits)
pub fn configure_frame_format(regs: Regs, config: &Config) {
    // Configure stop bits
    regs.baud().modify(|_, w| w.sbns().variant(config.stop_bits_count));

    // Clear M10 for now (10-bit mode)
    regs.baud().modify(|_, w| w.m10().disabled());
}

/// Configure control settings (parity, data bits, idle config, pin swap)
pub fn configure_control_settings(regs: Regs, config: &Config) {
    regs.ctrl().modify(|_, w| {
        // Parity configuration
        let mut w = if let Some(parity) = config.parity_mode {
            w.pe().enabled().pt().variant(parity)
        } else {
            w.pe().disabled()
        };

        // Data bits configuration
        w = match config.data_bits_count {
            DataBits::Data8 => {
                if config.parity_mode.is_some() {
                    w.m().data9() // 8 data + 1 parity = 9 bits
                } else {
                    w.m().data8() // 8 data bits only
                }
            }
            DataBits::Data9 => w.m().data9(),
        };

        // Idle configuration
        w = w.idlecfg().variant(config.rx_idle_config);
        w = w.ilt().variant(config.rx_idle_type);

        // Swap TXD/RXD if configured
        if config.swap_txd_rxd {
            w.swap().swap()
        } else {
            w.swap().standard()
        }
    });
}

/// Configure FIFO settings and watermarks
pub fn configure_fifo(regs: Regs, config: &Config) {
    // Configure WATER register for FIFO watermarks
    regs.water().write(|w| unsafe {
        w.rxwater()
            .bits(config.rx_fifo_watermark)
            .txwater()
            .bits(config.tx_fifo_watermark)
    });

    // Enable TX/RX FIFOs
    regs.fifo().modify(|_, w| w.txfe().enabled().rxfe().enabled());

    // Flush FIFOs
    regs.fifo()
        .modify(|_, w| w.txflush().txfifo_rst().rxflush().rxfifo_rst());
}

/// Clear all status flags
pub fn clear_all_status_flags(regs: Regs) {
    regs.stat().reset();
}

/// Configure hardware flow control if enabled
pub fn configure_flow_control(regs: Regs, enable_tx_cts: bool, enable_rx_rts: bool, config: &Config) {
    if enable_rx_rts || enable_tx_cts {
        regs.modir().modify(|_, w| {
            let mut w = w;

            // Configure TX CTS
            w = w.txctsc().variant(config.tx_cts_config);
            w = w.txctssrc().variant(config.tx_cts_source);

            if enable_rx_rts {
                w = w.rxrtse().enabled();
            } else {
                w = w.rxrtse().disabled();
            }

            if enable_tx_cts {
                w = w.txctse().enabled();
            } else {
                w = w.txctse().disabled();
            }

            w
        });
    }
}

/// Configure bit order (MSB first or LSB first)
pub fn configure_bit_order(regs: Regs, msb_first: MsbFirst) {
    regs.stat().modify(|_, w| w.msbf().variant(msb_first));
}

/// Enable transmitter and/or receiver based on configuration
pub fn enable_transceiver(regs: Regs, enable_tx: bool, enable_rx: bool) {
    regs.ctrl().modify(|_, w| {
        let mut w = w;
        if enable_tx {
            w = w.te().enabled();
        }
        if enable_rx {
            w = w.re().enabled();
        }
        w
    });
}

pub fn calculate_baudrate(baudrate: u32, src_clock_hz: u32) -> Result<(u8, u16)> {
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
pub fn wait_for_tx_complete(regs: Regs) {
    // Wait for TX FIFO to empty
    while regs.water().read().txcount().bits() != 0 {
        // Wait for TX FIFO to drain
    }

    // Wait for last character to shift out (TC = Transmission Complete)
    while regs.stat().read().tc().is_active() {
        // Wait for transmission to complete
    }
}

pub fn check_and_clear_rx_errors(regs: Regs) -> Result<()> {
    let stat = regs.stat().read();
    let mut status = Ok(());

    // Check for overrun first - other error flags are prevented when OR is set
    if stat.or().is_overrun() {
        regs.stat().write(|w| w.or().clear_bit_by_one());

        return Err(Error::Overrun);
    }

    if stat.pf().is_parity() {
        regs.stat().write(|w| w.pf().clear_bit_by_one());
        status = Err(Error::Parity);
    }

    if stat.fe().is_error() {
        regs.stat().write(|w| w.fe().clear_bit_by_one());
        status = Err(Error::Framing);
    }

    if stat.nf().is_noise() {
        regs.stat().write(|w| w.nf().clear_bit_by_one());
        status = Err(Error::Noise);
    }

    status
}

pub fn has_data(regs: Regs) -> bool {
    if regs.param().read().rxfifo().bits() > 0 {
        // FIFO is available - check RXCOUNT in WATER register
        regs.water().read().rxcount().bits() > 0
    } else {
        // No FIFO - check RDRF flag in STAT register
        regs.stat().read().rdrf().is_rxdata()
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
                self.set_function(crate::pac::port0::pcr0::Mux::$alt);
                self.set_enable_input_buffer();
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
                self.set_function(crate::pac::port0::pcr0::Mux::$alt);
                self.set_enable_input_buffer();
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
impl_tx_pin!(LPUART0, P0_3, Mux2);
impl_tx_pin!(LPUART0, P0_21, Mux3);
impl_tx_pin!(LPUART0, P2_1, Mux2);

impl_rx_pin!(LPUART0, P0_2, Mux2);
impl_rx_pin!(LPUART0, P0_20, Mux3);
impl_rx_pin!(LPUART0, P2_0, Mux2);

impl_cts_pin!(LPUART0, P0_1, Mux2);
impl_cts_pin!(LPUART0, P0_23, Mux3);
impl_cts_pin!(LPUART0, P2_3, Mux2);

impl_rts_pin!(LPUART0, P0_0, Mux2);
impl_rts_pin!(LPUART0, P0_22, Mux3);
impl_rts_pin!(LPUART0, P2_2, Mux2);

// LPUART 1
impl_tx_pin!(LPUART1, P1_9, Mux2);
impl_tx_pin!(LPUART1, P2_13, Mux3);
impl_tx_pin!(LPUART1, P3_9, Mux3);
impl_tx_pin!(LPUART1, P3_21, Mux3);

impl_rx_pin!(LPUART1, P1_8, Mux2);
impl_rx_pin!(LPUART1, P2_12, Mux3);
impl_rx_pin!(LPUART1, P3_8, Mux3);
impl_rx_pin!(LPUART1, P3_20, Mux3);

impl_cts_pin!(LPUART1, P1_11, Mux2);
impl_cts_pin!(LPUART1, P2_17, Mux3);
impl_cts_pin!(LPUART1, P3_11, Mux3);
impl_cts_pin!(LPUART1, P3_23, Mux3);

impl_rts_pin!(LPUART1, P1_10, Mux2);
impl_rts_pin!(LPUART1, P2_15, Mux3);
impl_rts_pin!(LPUART1, P2_16, Mux3);
impl_rts_pin!(LPUART1, P3_10, Mux3);

// LPUART 2
impl_tx_pin!(LPUART2, P1_5, Mux3);
impl_tx_pin!(LPUART2, P1_13, Mux3);
impl_tx_pin!(LPUART2, P2_2, Mux3);
impl_tx_pin!(LPUART2, P2_10, Mux3);
impl_tx_pin!(LPUART2, P3_15, Mux2);

impl_rx_pin!(LPUART2, P1_4, Mux3);
impl_rx_pin!(LPUART2, P1_12, Mux3);
impl_rx_pin!(LPUART2, P2_3, Mux3);
impl_rx_pin!(LPUART2, P2_11, Mux3);
impl_rx_pin!(LPUART2, P3_14, Mux2);

impl_cts_pin!(LPUART2, P1_7, Mux3);
impl_cts_pin!(LPUART2, P1_15, Mux3);
impl_cts_pin!(LPUART2, P2_4, Mux3);
impl_cts_pin!(LPUART2, P3_13, Mux2);

impl_rts_pin!(LPUART2, P1_6, Mux3);
impl_rts_pin!(LPUART2, P1_14, Mux3);
impl_rts_pin!(LPUART2, P2_5, Mux3);
impl_rts_pin!(LPUART2, P3_12, Mux2);

// LPUART 3
impl_tx_pin!(LPUART3, P3_1, Mux3);
impl_tx_pin!(LPUART3, P3_12, Mux3);
impl_tx_pin!(LPUART3, P4_5, Mux3);

impl_rx_pin!(LPUART3, P3_0, Mux3);
impl_rx_pin!(LPUART3, P3_13, Mux3);
impl_rx_pin!(LPUART3, P4_2, Mux3);

impl_cts_pin!(LPUART3, P3_7, Mux3);
impl_cts_pin!(LPUART3, P3_14, Mux3);
impl_cts_pin!(LPUART3, P4_6, Mux3);

impl_rts_pin!(LPUART3, P3_6, Mux3);
impl_rts_pin!(LPUART3, P3_15, Mux3);
impl_rts_pin!(LPUART3, P4_7, Mux3);

// LPUART 4
impl_tx_pin!(LPUART4, P2_7, Mux3);
impl_tx_pin!(LPUART4, P3_19, Mux2);
impl_tx_pin!(LPUART4, P3_27, Mux3);
impl_tx_pin!(LPUART4, P4_3, Mux3);

impl_rx_pin!(LPUART4, P2_6, Mux3);
impl_rx_pin!(LPUART4, P3_18, Mux2);
impl_rx_pin!(LPUART4, P3_28, Mux3);
impl_rx_pin!(LPUART4, P4_4, Mux3);

impl_cts_pin!(LPUART4, P2_0, Mux3);
impl_cts_pin!(LPUART4, P3_17, Mux2);
impl_cts_pin!(LPUART4, P3_31, Mux3);

impl_rts_pin!(LPUART4, P2_1, Mux3);
impl_rts_pin!(LPUART4, P3_16, Mux2);
impl_rts_pin!(LPUART4, P3_30, Mux3);

// LPUART 5
impl_tx_pin!(LPUART5, P1_10, Mux8);
impl_tx_pin!(LPUART5, P1_17, Mux8);

impl_rx_pin!(LPUART5, P1_11, Mux8);
impl_rx_pin!(LPUART5, P1_16, Mux8);

impl_cts_pin!(LPUART5, P1_12, Mux8);
impl_cts_pin!(LPUART5, P1_19, Mux8);

impl_rts_pin!(LPUART5, P1_13, Mux8);
impl_rts_pin!(LPUART5, P1_18, Mux8);

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
            data_bits_count: DataBits::Data8,
            msb_first: MsbFirst::LsbFirst,
            stop_bits_count: StopBits::One,
            tx_fifo_watermark: 0,
            rx_fifo_watermark: 1,
            tx_cts_source: TxCtsSource::Cts,
            tx_cts_config: TxCtsConfig::Start,
            rx_idle_type: IdleType::FromStart,
            rx_idle_config: IdleConfig::Idle1,
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
    info: Info,
    tx: LpuartTx<'a, M>,
    rx: LpuartRx<'a, M>,
}

/// Lpuart TX driver.
pub struct LpuartTx<'a, M: Mode> {
    info: Info,
    _tx_pin: Peri<'a, AnyPin>,
    _cts_pin: Option<Peri<'a, AnyPin>>,
    _tx_dma: Option<Channel<'a>>,
    mode: PhantomData<(&'a (), M)>,
}

/// Lpuart Rx driver.
pub struct LpuartRx<'a, M: Mode> {
    info: Info,
    _rx_pin: Peri<'a, AnyPin>,
    _rts_pin: Option<Peri<'a, AnyPin>>,
    _rx_dma: Option<Channel<'a>>,
    mode: PhantomData<(&'a (), M)>,
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
    ) -> Result<()> {
        let regs = T::info().regs;

        // Enable clocks
        let conf = LpuartConfig {
            power: config.power,
            source: config.source,
            div: config.div,
            instance: T::CLOCK_INSTANCE,
        };
        let clock_freq = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        // Perform initialization sequence
        perform_software_reset(regs);
        disable_transceiver(regs);
        configure_baudrate(regs, config.baudrate_bps, clock_freq)?;
        configure_frame_format(regs, &config);
        configure_control_settings(regs, &config);
        configure_fifo(regs, &config);
        clear_all_status_flags(regs);
        configure_flow_control(regs, enable_tx_cts, enable_rx_rts, &config);
        configure_bit_order(regs, config.msb_first);
        enable_transceiver(regs, enable_rx, enable_tx);

        Ok(())
    }

    /// Deinitialize the LPUART peripheral
    pub fn deinit(&self) -> Result<()> {
        let regs = self.info.regs;

        // Wait for TX operations to complete
        wait_for_tx_complete(regs);

        // Clear all status flags
        clear_all_status_flags(regs);

        // Disable the module - clear all CTRL register bits
        regs.ctrl().reset();

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
        Self::init::<T>(true, true, false, false, config)?;

        Ok(Self {
            info: T::info(),
            tx: LpuartTx::new_inner(T::info(), tx_pin.into(), None, None),
            rx: LpuartRx::new_inner(T::info(), rx_pin.into(), None, None),
        })
    }

    /// Create a new blocking LPUART instance with RX, TX and RTS/CTS flow control pins
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
        Self::init::<T>(true, true, true, true, config)?;

        Ok(Self {
            info: T::info(),
            rx: LpuartRx::new_inner(T::info(), rx_pin.into(), Some(rts_pin.into()), None),
            tx: LpuartTx::new_inner(T::info(), tx_pin.into(), Some(cts_pin.into()), None),
        })
    }
}

// ----------------------------------------------------------------------------
// Blocking TX Implementation
// ----------------------------------------------------------------------------

impl<'a, M: Mode> LpuartTx<'a, M> {
    fn new_inner(
        info: Info,
        tx_pin: Peri<'a, AnyPin>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        tx_dma: Option<Channel<'a>>,
    ) -> Self {
        Self {
            info,
            _tx_pin: tx_pin,
            _cts_pin: cts_pin,
            _tx_dma: tx_dma,
            mode: PhantomData,
        }
    }
}

impl<'a> LpuartTx<'a, Blocking> {
    /// Create a new blocking LPUART transmitter instance
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        // Configure the pins for LPUART usage
        tx_pin.as_tx();

        // Initialize the peripheral
        Lpuart::<Blocking>::init::<T>(true, false, false, false, config)?;

        Ok(Self::new_inner(T::info(), tx_pin.into(), None, None))
    }

    /// Create a new blocking LPUART transmitter instance with CTS flow control
    pub fn new_blocking_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        cts_pin.as_cts();

        Lpuart::<Blocking>::init::<T>(true, false, true, false, config)?;

        Ok(Self::new_inner(T::info(), tx_pin.into(), Some(cts_pin.into()), None))
    }

    fn write_byte_internal(&mut self, byte: u8) -> Result<()> {
        self.info.regs.data().modify(|_, w| unsafe { w.bits(u32::from(byte)) });

        Ok(())
    }

    fn blocking_write_byte(&mut self, byte: u8) -> Result<()> {
        while self.info.regs.stat().read().tdre().is_txdata() {}
        self.write_byte_internal(byte)
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.info.regs.stat().read().tdre().is_txdata() {
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
        while self.info.regs.water().read().txcount().bits() != 0 {
            // Wait for TX FIFO to drain
        }

        // Wait for last character to shift out
        while self.info.regs.stat().read().tc().is_active() {
            // Wait for transmission to complete
        }

        Ok(())
    }

    /// Flush LPUART TX.
    pub fn flush(&mut self) -> Result<()> {
        // Check if TX FIFO is empty
        if self.info.regs.water().read().txcount().bits() != 0 {
            return Err(Error::TxBusy);
        }

        // Check if transmission is complete
        if self.info.regs.stat().read().tc().is_active() {
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
        info: Info,
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        rx_dma: Option<Channel<'a>>,
    ) -> Self {
        Self {
            info,
            _rx_pin: rx_pin,
            _rts_pin: rts_pin,
            _rx_dma: rx_dma,
            mode: PhantomData,
        }
    }
}

impl<'a> LpuartRx<'a, Blocking> {
    /// Create a new blocking LPUART Receiver instance
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();

        Lpuart::<Blocking>::init::<T>(false, true, false, false, config)?;

        Ok(Self::new_inner(T::info(), rx_pin.into(), None, None))
    }

    /// Create a new blocking LPUART Receiver instance with RTS flow control
    pub fn new_blocking_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();
        rts_pin.as_rts();

        Lpuart::<Blocking>::init::<T>(false, true, false, true, config)?;

        Ok(Self::new_inner(T::info(), rx_pin.into(), Some(rts_pin.into()), None))
    }

    fn read_byte_internal(&mut self) -> Result<u8> {
        let data = self.info.regs.data().read();

        Ok((data.bits() & 0xFF) as u8)
    }

    fn read_byte(&mut self) -> Result<u8> {
        check_and_clear_rx_errors(self.info.regs)?;

        if !has_data(self.info.regs) {
            return Err(Error::RxFifoEmpty);
        }

        self.read_byte_internal()
    }

    fn blocking_read_byte(&mut self) -> Result<u8> {
        loop {
            if has_data(self.info.regs) {
                return self.read_byte_internal();
            }

            check_and_clear_rx_errors(self.info.regs)?;
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
// ASYNC MODE IMPLEMENTATIONS
// ============================================================================

// TODO: Implement async mode for LPUART

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
