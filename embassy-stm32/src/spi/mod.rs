//! Serial Peripheral Interface (SPI)
#![macro_use]

#[cfg(feature = "exti")]
mod ringbuffered;
use core::marker::PhantomData;
use core::pin::pin;
use core::ptr;
use core::sync::atomic::{Ordering, fence};

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
pub use embedded_hal_02::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode, Phase, Polarity};
use futures_util::future::{Either, select};
#[cfg(feature = "exti")]
pub use ringbuffered::RingBufferedSpiRx;

use crate::Peri;
use crate::dma::{ChannelAndRequest, word};
#[cfg(feature = "exti")]
use crate::exti::{self, ExtiInput};
#[cfg(feature = "exti")]
use crate::gpio::ExtiPin;
use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::spi::{Spi as Regs, regs, vals};
use crate::time::Hertz;

/// SPI error.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Invalid framing.
    Framing,
    /// CRC error (only if hardware CRC checking is enabled).
    Crc,
    /// Mode fault
    ModeFault,
    /// Overrun.
    Overrun,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Framing => "Invalid Framing",
            Self::Crc => "Hardware CRC Check Failed",
            Self::ModeFault => "Mode Fault",
            Self::Overrun => "Buffer Overrun",
        };

        write!(f, "{}", message)
    }
}

impl core::error::Error for Error {}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

/// SPI bit order
#[derive(Copy, Clone)]
pub enum BitOrder {
    /// Least significant bit first.
    LsbFirst,
    /// Most significant bit first.
    MsbFirst,
}

/// SPI Direction.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Transmit
    Transmit,
    /// Receive
    Receive,
}

/// Slave Select (SS) pin polarity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlaveSelectPolarity {
    /// SS active high
    ActiveHigh,
    /// SS active low
    ActiveLow,
}

impl SlaveSelectPolarity {
    fn from_regs(_regs: Regs) -> Self {
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        match _regs.cfg2().read().ssiop() {
            vals::Ssiop::ActiveLow => SlaveSelectPolarity::ActiveLow,
            _ => SlaveSelectPolarity::ActiveHigh,
        }

        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        SlaveSelectPolarity::ActiveLow
    }
}

/// CRC configuration.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrcConfig {
    /// Hardware CRC disabled.
    Disabled,
    /// Hardware CRC enabled with 8-bit polynomial.
    Crc8 {
        /// The polynomial
        polynomial: u8,
    },
    /// Hardware CRC enabled with 16-bit polynomial.
    Crc16 {
        /// The polynomial
        polynomial: u16,
    },
}

impl Default for CrcConfig {
    fn default() -> Self {
        CrcConfig::Disabled
    }
}

/// SPI configuration.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// SPI mode.
    pub mode: Mode,
    /// Bit order.
    pub bit_order: BitOrder,
    /// Clock frequency.
    pub frequency: Hertz,
    /// Enable internal pullup on input pin.
    ///
    /// There are some ICs that require a pull-up on the input pin for some applications.
    /// If you are unsure, you probably don't need this.
    pub input_pull: Pull,
    /// Enable internal pullup on the NSS input pin when SPI is in slave mode.
    ///
    /// If you are unsure, you probably don't need this.
    pub nss_pull: Pull,
    /// signal rise/fall speed (slew rate) - defaults to `VeryHigh`.
    /// Increase for high SPI speeds. Change to `Low` to reduce ringing.
    pub gpio_speed: Speed,
    /// If True sets SSOE to zero even if SPI is in Master Mode.
    /// NSS output enabled (SSM = 0, SSOE = 1): The NSS signal is driven low when the master starts the communication and is kept low until the SPI is disabled.
    /// NSS output disabled (SSM = 0, SSOE = 0): For devices set as slave, the NSS pin acts as a classical NSS input: the slave is selected when NSS is low and deselected when NSS high.
    pub nss_output_disable: bool,
    /// Hardware CRC configuration.
    pub crc: CrcConfig,
    /// Slave Select (SS) pin polarity.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    pub nss_polarity: SlaveSelectPolarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            bit_order: BitOrder::MsbFirst,
            frequency: Hertz(1_000_000),
            input_pull: Pull::None,
            nss_pull: Pull::None,
            gpio_speed: Speed::VeryHigh,
            nss_output_disable: false,
            crc: CrcConfig::default(),
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            nss_polarity: SlaveSelectPolarity::ActiveLow,
        }
    }
}

impl Config {
    fn raw_phase(&self) -> vals::Cpha {
        match self.mode.phase {
            Phase::CaptureOnSecondTransition => vals::Cpha::SecondEdge,
            Phase::CaptureOnFirstTransition => vals::Cpha::FirstEdge,
        }
    }

    fn raw_polarity(&self) -> vals::Cpol {
        match self.mode.polarity {
            Polarity::IdleHigh => vals::Cpol::IdleHigh,
            Polarity::IdleLow => vals::Cpol::IdleLow,
        }
    }

    fn raw_byte_order(&self) -> vals::Lsbfirst {
        match self.bit_order {
            BitOrder::LsbFirst => vals::Lsbfirst::LsbFirst,
            BitOrder::MsbFirst => vals::Lsbfirst::MsbFirst,
        }
    }

    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    fn raw_nss_polarity(&self) -> vals::Ssiop {
        match self.nss_polarity {
            SlaveSelectPolarity::ActiveHigh => vals::Ssiop::ActiveHigh,
            SlaveSelectPolarity::ActiveLow => vals::Ssiop::ActiveLow,
        }
    }

    #[cfg(gpio_v1)]
    fn sck_af(&self) -> AfType {
        AfType::output(OutputType::PushPull, self.gpio_speed)
    }

    #[cfg(gpio_v2)]
    fn sck_af(&self) -> AfType {
        AfType::output_pull(
            OutputType::PushPull,
            self.gpio_speed,
            match self.mode.polarity {
                Polarity::IdleLow => Pull::Down,
                Polarity::IdleHigh => Pull::Up,
            },
        )
    }
}

/// SPI communication mode
pub mod mode {
    use stm32_metapac::spi::vals;

    trait SealedMode {}

    /// Trait for SPI communication mode operations.
    #[allow(private_bounds)]
    pub trait CommunicationMode: SealedMode {
        /// Spi communication mode
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        const MASTER: vals::Mstr;
        /// Spi communication mode
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        const MASTER: vals::Master;
    }

    /// Mode allowing for SPI master operations.
    pub struct Master;
    /// Mode allowing for SPI slave operations.
    pub struct Slave;

    impl SealedMode for Master {}
    impl CommunicationMode for Master {
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        const MASTER: vals::Mstr = vals::Mstr::Master;
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        const MASTER: vals::Master = vals::Master::Master;
    }

    impl SealedMode for Slave {}
    impl CommunicationMode for Slave {
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        const MASTER: vals::Mstr = vals::Mstr::Slave;
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        const MASTER: vals::Master = vals::Master::Slave;
    }
}
use mode::{CommunicationMode, Master, Slave};

enum CsPinType<'d> {
    #[allow(dead_code)]
    Flex(Flex<'d>),
    #[cfg(feature = "exti")]
    Exti(ExtiInput<'d, Async>),
    None,
}

impl<'d> CsPinType<'d> {
    #[cfg(feature = "exti")]
    pub fn new_with_exti<
        T: Instance,
        #[cfg(not(afio))] C: CsPin<T> + ExtiPin,
        #[cfg(afio)] C: CsPin<T, A> + ExtiPin,
        #[cfg(afio)] A,
    >(
        pin: Peri<'d, C>,
        exti: Option<Peri<'d, C::ExtiChannel>>,
        af_type: AfType,
        irq: impl crate::interrupt::typelevel::Binding<
            <<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ,
            exti::InterruptHandler<<<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ>,
        >,
    ) -> Self {
        set_as_af!(pin, af_type);

        match exti {
            Some(ch) => Self::Exti(ExtiInput::new_af(pin, ch, irq)),
            None => Self::Flex(Flex::new(pin)),
        }
    }

    pub async fn wait_for_edge(&mut self, _polarity: SlaveSelectPolarity) {
        match self {
            #[cfg(feature = "exti")]
            Self::Exti(exti) => match _polarity {
                SlaveSelectPolarity::ActiveHigh => exti.wait_for_rising_edge().await,
                SlaveSelectPolarity::ActiveLow => exti.wait_for_falling_edge().await,
            },
            Self::Flex(_) | Self::None => core::future::pending().await,
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[cfg(feature = "exti")]
    pub const unsafe fn clone_unchecked(&self) -> Self {
        match self {
            Self::Exti(exti) => Self::Exti(exti.clone_unchecked()),
            Self::Flex(flex) => Self::Flex(flex.clone_unchecked()),
            Self::None => Self::None,
        }
    }
}

/// SPI driver.
pub struct Spi<'d, M: PeriMode, CM: CommunicationMode> {
    pub(crate) info: &'static Info,
    kernel_clock: Hertz,
    _sck: Option<Flex<'d>>,
    _mosi: Option<Flex<'d>>,
    _miso: Option<Flex<'d>>,
    nss: CsPinType<'d>,
    tx_dma: Option<ChannelAndRequest<'d>>,
    rx_dma: Option<ChannelAndRequest<'d>>,
    _marker: PhantomData<(M, CM)>,
    current_word_size: word_impl::Config,
    input_pull: Pull,
    nss_pull: Pull,
    gpio_speed: Speed,
    crc_enabled: bool,
}

impl<'d, M: PeriMode, CM: CommunicationMode> Spi<'d, M, CM> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Option<Flex<'d>>,
        mosi: Option<Flex<'d>>,
        miso: Option<Flex<'d>>,
        nss: CsPinType<'d>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Self {
        let mut this = Self {
            info: T::info(),
            kernel_clock: T::frequency(),
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            nss,
            tx_dma,
            rx_dma,
            current_word_size: <u8 as SealedWord>::CONFIG,
            _marker: PhantomData,
            input_pull: config.input_pull,
            nss_pull: config.nss_pull,
            gpio_speed: config.gpio_speed,
            crc_enabled: !matches!(config.crc, CrcConfig::Disabled),
        };
        this.enable_and_init(config);
        this
    }

    fn enable_and_init(&mut self, config: Config) {
        let br = compute_baud_rate(self.kernel_clock, config.frequency);
        let cpha = config.raw_phase();
        let cpol = config.raw_polarity();
        let lsbfirst = config.raw_byte_order();

        self.info.rcc.enable_and_reset();

        /*
        - Software NSS management (SSM = 1)
        The slave select information is driven internally by the value of the SSI bit in the
        SPI_CR1 register. The external NSS pin remains free for other application uses.

        - Hardware NSS management (SSM = 0)
        Two configurations are possible depending on the NSS output configuration (SSOE bit
        in register SPI_CR1).

        -- NSS output enabled (SSM = 0, SSOE = 1)
          This configuration is used only when the device operates in master mode. The
          NSS signal is driven low when the master starts the communication and is kept
          low until the SPI is disabled.

        -- NSS output disabled (SSM = 0, SSOE = 0)
            This configuration allows multimaster capability for devices operating in master
            mode. For devices set as slave, the NSS pin acts as a classical NSS input: the
            slave is selected when NSS is low and deselected when NSS high
         */
        let ssm = self.nss.is_none();

        let regs = self.info.regs;
        #[cfg(any(spi_v1, spi_v2))]
        {
            let ssoe = CM::MASTER == vals::Mstr::Master && !config.nss_output_disable;
            regs.cr2().modify(|w| {
                w.set_ssoe(ssoe);
            });
            regs.cr1().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);

                w.set_mstr(CM::MASTER);
                w.set_br(br);
                w.set_spe(true);
                w.set_lsbfirst(lsbfirst);
                w.set_ssi(CM::MASTER == vals::Mstr::Master);
                w.set_ssm(ssm);
                w.set_crcen(self.crc_enabled);
                w.set_bidimode(vals::Bidimode::Unidirectional);
                w.set_rxonly(match (&self.rx_dma, &self.tx_dma) {
                    (Some(_), None) => vals::Rxonly::OutputDisabled,
                    // we're doing "fake rxonly", by actually writing one
                    // byte to TXDR for each byte we want to receive. if we
                    // set OUTPUTDISABLED here, this hangs.
                    _ => vals::Rxonly::FullDuplex,
                });
                w.set_dff(<u8 as SealedWord>::CONFIG)
            });
            // spi_v1 and spi_v2 (non-I2S) have no CRCL field; CRC is always 8-bit.
            match config.crc {
                CrcConfig::Crc8 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u16)),
                CrcConfig::Crc16 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial)),
                CrcConfig::Disabled => {}
            }
        }
        #[cfg(all(any(spi_v2_i2s, spi_v3), not(spi_v2)))]
        {
            let ssoe = CM::MASTER == vals::Mstr::Master && !config.nss_output_disable;
            regs.cr2().modify(|w| {
                let (ds, frxth) = <u8 as SealedWord>::CONFIG;
                w.set_frxth(frxth);
                w.set_ds(ds);
                w.set_ssoe(ssoe);
            });
            regs.cr1().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);

                w.set_mstr(CM::MASTER);
                w.set_br(br);
                w.set_lsbfirst(lsbfirst);
                w.set_ssi(CM::MASTER == vals::Mstr::Master);
                w.set_ssm(ssm);
                w.set_crcen(self.crc_enabled);
                if self.crc_enabled {
                    match config.crc {
                        CrcConfig::Crc8 { .. } => w.set_crcl(vals::Crcl::Bits8),
                        CrcConfig::Crc16 { .. } => w.set_crcl(vals::Crcl::Bits16),
                        CrcConfig::Disabled => {}
                    };
                }
                w.set_bidimode(vals::Bidimode::Unidirectional);
                w.set_spe(true);
            });
            match config.crc {
                CrcConfig::Crc8 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u16)),
                CrcConfig::Crc16 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial)),
                CrcConfig::Disabled => {}
            }
        }
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        {
            let ssoe = CM::MASTER == vals::Master::Master && !config.nss_output_disable;
            let ssiop = config.raw_nss_polarity();
            regs.ifcr().write(|w| w.0 = 0xffff_ffff);
            regs.cfg2().modify(|w| {
                w.set_ssoe(ssoe);
                w.set_cpha(cpha);
                w.set_cpol(cpol);
                w.set_lsbfirst(lsbfirst);
                w.set_ssm(ssm);
                w.set_master(CM::MASTER);
                w.set_comm(vals::Comm::FullDuplex);
                w.set_ssom(vals::Ssom::Asserted);
                w.set_midi(0);
                w.set_mssi(0);
                w.set_afcntr(true);
                w.set_ssiop(ssiop);
            });
            regs.cfg1().modify(|w| {
                w.set_crcen(self.crc_enabled);
                w.set_mbr(br);
                w.set_dsize(<u8 as SealedWord>::CONFIG);
                w.set_fthlv(vals::Fthlv::OneFrame);
            });
            regs.cr2().modify(|w| {
                w.set_tsize(0);
            });
            regs.cr1().modify(|w| {
                w.set_ssi(false);
                w.set_spe(true);
            });
            match config.crc {
                CrcConfig::Crc8 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
                CrcConfig::Crc16 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
                CrcConfig::Disabled => {}
            }
        }
    }

    /// Reconfigures it with the supplied config.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ()> {
        self.gpio_speed = config.gpio_speed;
        self.crc_enabled = !matches!(config.crc, CrcConfig::Disabled);
        #[cfg(gpio_v2)]
        set_speed(&self._sck, &self._mosi, config.gpio_speed);
        reconfigure(self.info, self.kernel_clock, config)
    }

    /// Enable or disable hardware CRC calculation.
    pub fn set_crc(&mut self, config: CrcConfig) {
        let regs = self.info.regs;
        self.crc_enabled = !matches!(config, CrcConfig::Disabled);

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        regs.cr1().modify(|w| w.set_spe(false));
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cr1().modify(|w| w.set_spe(false));

        #[cfg(any(spi_v2_i2s, spi_v3))]
        regs.cr1().modify(|w| {
            w.set_crcen(self.crc_enabled);
            if self.crc_enabled {
                match config {
                    CrcConfig::Crc8 { .. } => w.set_crcl(vals::Crcl::Bits8),
                    CrcConfig::Crc16 { .. } => w.set_crcl(vals::Crcl::Bits16),
                    CrcConfig::Disabled => {}
                };
            }
        });

        #[cfg(spi_v3)]
        regs.cr1().modify(|w| w.set_crcen(self.crc_enabled));

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cfg1().modify(|w| w.set_crcen(self.crc_enabled));

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        match config {
            CrcConfig::Crc8 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u16)),
            CrcConfig::Crc16 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial)),
            CrcConfig::Disabled => {}
        }
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        match config {
            CrcConfig::Crc8 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
            CrcConfig::Crc16 { polynomial } => regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
            CrcConfig::Disabled => {}
        }

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        regs.cr1().modify(|w| w.set_spe(true));
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cr1().modify(|w| w.set_spe(true));
    }

    /// Read the TX CRC register value.
    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    pub fn crc_value(&self) -> u16 {
        self.info.regs.txcrc().read().txcrc() as u16
    }

    /// Read the TX CRC register value.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    pub fn crc_value(&self) -> u16 {
        self.info.regs.txcrc().read().txcrc() as u16
    }

    /// Read the RX CRC register value.
    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    pub fn rx_crc_value(&self) -> u16 {
        self.info.regs.rxcrc().read().rxcrc() as u16
    }

    /// Read the TX CRC register value.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    pub fn rx_crc_value(&self) -> u16 {
        self.info.regs.rxcrc().read().rxcrc() as u16
    }

    /// Check if a CRC error occurred.
    fn check_transfer_crc(&self) -> Result<(), Error> {
        if !self.crc_enabled {
            return Ok(());
        }

        let regs = self.info.regs;

        // Read CRC from data register to clear RXNE.
        #[cfg(any(spi_v1, spi_v2))]
        {
            let _ = regs.dr().read();
        }
        #[cfg(spi_v3)]
        {
            let _ = regs.dr16().read();
        }
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        {
            let _ = regs.rxdr32().read();
        }

        // Check for CRC mismatch.
        let sr = regs.sr().read();
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        if sr.crcerr() {
            return Err(Error::Crc);
        }
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        if sr.crce() {
            return Err(Error::Crc);
        }

        Ok(())
    }

    /// Set SPI direction for bidirectional mode.
    ///
    /// This properly handles the STM32 requirement that BIDIOE cannot be changed
    /// while the SPI peripheral is enabled (SPE=1). Per the STM32 reference manual,
    /// we must wait for TXE=1 and BSY=0 before disabling SPE to ensure any ongoing
    /// transfer completes cleanly.
    ///
    /// The SPE state is preserved: if SPI was enabled before this call, it will
    /// be re-enabled after; if it was disabled, it remains disabled.
    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    pub fn set_direction(&mut self, dir: Option<Direction>) {
        let (bidimode, bidioe) = match dir {
            Some(Direction::Transmit) => (vals::Bidimode::Bidirectional, vals::Bidioe::Transmit),
            Some(Direction::Receive) => (vals::Bidimode::Bidirectional, vals::Bidioe::Receive),
            None => (vals::Bidimode::Unidirectional, vals::Bidioe::Transmit),
        };

        let was_enabled = self.info.regs.cr1().read().spe();

        // If SPE is currently enabled, wait for any ongoing transfer to complete.
        // Per STM32 reference manual: wait for TXE=1 then BSY=0 before disabling SPE.
        if was_enabled {
            while !self.info.regs.sr().read().txe() {}
            while self.info.regs.sr().read().bsy() {}
        }

        // BIDIOE cannot be changed while SPE=1, so disable first
        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });
        self.info.regs.cr1().modify(|w| {
            w.set_bidimode(bidimode);
            w.set_bidioe(bidioe);
        });

        // Restore previous SPE state
        if was_enabled {
            self.info.regs.cr1().modify(|w| {
                w.set_spe(true);
            });
        }
    }

    /// Get current SPI configuration.
    pub fn get_current_config(&self) -> Config {
        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        let cfg = self.info.regs.cr1().read();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let cfg = self.info.regs.cfg2().read();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let cfg1 = self.info.regs.cfg1().read();

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        let ssoe = self.info.regs.cr2().read().ssoe();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let ssoe = cfg.ssoe();

        let polarity = if cfg.cpol() == vals::Cpol::IdleLow {
            Polarity::IdleLow
        } else {
            Polarity::IdleHigh
        };
        let phase = if cfg.cpha() == vals::Cpha::FirstEdge {
            Phase::CaptureOnFirstTransition
        } else {
            Phase::CaptureOnSecondTransition
        };

        let bit_order = if cfg.lsbfirst() == vals::Lsbfirst::LsbFirst {
            BitOrder::LsbFirst
        } else {
            BitOrder::MsbFirst
        };

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        let br = cfg.br();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let br = cfg1.mbr();

        let frequency = compute_frequency(self.kernel_clock, br);

        // NSS output disabled if SSOE=0 or if SSM=1 software slave management enabled
        let nss_output_disable = !ssoe || cfg.ssm();

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let nss_polarity = if cfg.ssiop() == vals::Ssiop::ActiveLow {
            SlaveSelectPolarity::ActiveLow
        } else {
            SlaveSelectPolarity::ActiveHigh
        };

        #[cfg(any(spi_v1, spi_v2, spi_v3))]
        let crc_enabled = cfg.crcen();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let crc_enabled = cfg1.crcen();

        let crc = if crc_enabled {
            #[cfg(any(spi_v1, spi_v2, spi_v3))]
            let poly = self.info.regs.crcpoly().read().crcpoly() as u16;
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            let poly = self.info.regs.crcpoly().read().crcpoly() as u16;

            #[cfg(all(any(spi_v1, spi_v2), not(spi_v2_i2s)))]
            {
                // spi_v1 and spi_v2 (non-I2S) have no CRCL field; CRC is always 8-bit.
                CrcConfig::Crc8 { polynomial: poly as u8 }
            }
            #[cfg(any(spi_v2_i2s, spi_v3))]
            match cfg.crcl() {
                vals::Crcl::Bits8 => CrcConfig::Crc8 { polynomial: poly as u8 },
                vals::Crcl::Bits16 => CrcConfig::Crc16 { polynomial: poly },
            }
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            {
                // spi_v4/v5/v6 do not expose CRC width in a readable register.
                // Heuristic: if the polynomial fits in u8, assume CRC-8.
                if poly <= u8::MAX as u16 {
                    CrcConfig::Crc8 { polynomial: poly as u8 }
                } else {
                    CrcConfig::Crc16 { polynomial: poly }
                }
            }
        } else {
            CrcConfig::Disabled
        };

        Config {
            mode: Mode { polarity, phase },
            bit_order,
            frequency,
            input_pull: self.input_pull,
            nss_pull: self.nss_pull,
            gpio_speed: self.gpio_speed,
            nss_output_disable,
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            nss_polarity,
            crc,
        }
    }

    pub(crate) fn set_word_size(&mut self, word_size: word_impl::Config) {
        if self.current_word_size == word_size {
            return;
        }

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        #[cfg(any(spi_v1, spi_v2))]
        self.info.regs.cr1().modify(|reg| {
            reg.set_dff(word_size);
        });
        #[cfg(spi_v3)]
        self.info.regs.cr2().modify(|w| {
            w.set_frxth(word_size.1);
            w.set_ds(word_size.0);
        });
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cfg1().modify(|w| {
            w.set_dsize(word_size);
        });

        self.current_word_size = word_size;
    }

    /// Blocking write.
    pub fn blocking_write<W: Word>(&mut self, words: &[W]) -> Result<(), Error> {
        // needed in spi_v4+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);

        // Memory barrier after flush RX fifo to ensure register writes complete
        fence(Ordering::SeqCst);

        let last_idx = words.len().saturating_sub(1);
        for (i, word) in words.iter().enumerate() {
            if self.crc_enabled && i == last_idx {
                #[cfg(any(spi_v1, spi_v2, spi_v3))]
                self.info.regs.cr1().modify(|w| w.set_crcnext(vals::Crcnext::Crc));
            }

            // this cannot use `transfer_word` because on spi_v3 and higher,
            // the SPI RX state machine hangs if no physical pin is connected to the SCK AF.
            // This is the case when the SPI has been created with `new_(blocking_?)txonly_nosck`.
            // See https://github.com/embassy-rs/embassy/issues/2902
            // This is not documented as an errata by ST, and I've been unable to find anything online...
            #[cfg(not(any(spi_v1, spi_v2)))]
            write_word(self.info.regs, *word)?;

            // if we're doing tx only, after writing the last byte to FIFO we have to wait
            // until it's actually sent. On spi_v1 and spi_v2 you're supposed to use the BSY flag for this
            // but apparently it's broken, it clears too soon. Workaround is to wait for RXNE:
            // when it gets set you know the transfer is done, even if you don't care about rx.
            // Luckily this doesn't affect spi_v3+.
            // See http://efton.sk/STM32/gotcha/g68.html
            // ST doesn't seem to document this in errata sheets (?)
            #[cfg(any(spi_v1, spi_v2))]
            transfer_word(self.info.regs, *word)?;
        }

        // wait until last word is transmitted. (except on spi_v1 and spi_v2, see above)
        #[cfg(not(any(spi_v1, spi_v2, spi_v3)))]
        while !self.info.regs.sr().read().txc() {}
        #[cfg(spi_v3)]
        while self.info.regs.sr().read().bsy() {}

        self.check_transfer_crc()?;

        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        // needed in spi_v4+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);

        // Memory barrier after flush RX fifo to ensure register writes complete
        fence(Ordering::SeqCst);

        transfer_words(self.info.regs, words, &[])?;

        self.check_transfer_crc()?;

        Ok(())
    }

    /// Blocking in-place bidirectional transfer.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub fn blocking_transfer_in_place<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        // needed in spi_v4+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);

        // Memory barrier after flush RX fifo to ensure register writes complete
        fence(Ordering::SeqCst);

        transfer_words(self.info.regs, words, words)?;

        self.check_transfer_crc()?;

        Ok(())
    }

    /// Blocking bidirectional transfer.
    ///
    /// This transfers both buffers at the same time, so it is NOT equivalent to `write` followed by `read`.
    ///
    /// The transfer runs for `max(read.len(), write.len())` bytes. If `read` is shorter extra bytes are ignored.
    /// If `write` is shorter it is padded with zero bytes.
    pub fn blocking_transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        // needed in spi_v4+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);

        // Memory barrier after flush RX fifo to ensure register writes complete
        fence(Ordering::SeqCst);

        transfer_words(self.info.regs, read, write)?;

        self.check_transfer_crc()?;

        Ok(())
    }
}

impl<'d> Spi<'d, Blocking, Slave> {
    /// Create a new blocking SPI slave driver.
    pub fn new_blocking_slave<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        cs: Peri<'d, if_afio!(impl CsPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::input(config.input_pull)),
            new_pin!(miso, AfType::output(OutputType::PushPull, config.gpio_speed)),
            CsPinType::Flex(new_pin!(cs, AfType::input(config.nss_pull)).unwrap()),
            None,
            None,
            config,
        )
    }
}

impl<'d> Spi<'d, Blocking, Master> {
    /// Create a new blocking SPI driver.
    pub fn new_blocking<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(miso, AfType::input(config.input_pull)),
            CsPinType::None,
            None,
            None,
            config,
        )
    }

    /// Create a new blocking SPI driver, in RX-only mode (only MISO pin, no MOSI).
    pub fn new_blocking_rxonly<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            None,
            new_pin!(miso, AfType::input(config.input_pull)),
            CsPinType::None,
            None,
            None,
            config,
        )
    }

    /// Create a new blocking SPI driver, in TX-only mode (only MOSI pin, no MISO).
    pub fn new_blocking_txonly<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            None,
            CsPinType::None,
            None,
            None,
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode, without SCK pin.
    ///
    /// This can be useful for bit-banging non-SPI protocols.
    pub fn new_blocking_txonly_nosck<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            None,
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            None,
            CsPinType::None,
            None,
            None,
            config,
        )
    }
}

#[cfg(feature = "exti")]
impl<'d> Spi<'d, Async, Slave> {
    /// Create a new SPI slave driver.
    pub fn new_slave<
        T: Instance,
        D1: TxDma<T>,
        D2: RxDma<T>,
        #[cfg(not(afio))] C: CsPin<T> + ExtiPin,
        #[cfg(afio)] C: CsPin<T, A> + ExtiPin,
        #[cfg(afio)] A,
    >(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        cs: Peri<'d, C>,
        tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        exti: Option<Peri<'d, C::ExtiChannel>>,
        irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + crate::interrupt::typelevel::Binding<
            <<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ,
            exti::InterruptHandler<<<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ>,
        > + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::input(config.input_pull)),
            new_pin!(miso, AfType::output(OutputType::PushPull, config.gpio_speed)),
            CsPinType::new_with_exti(cs, exti, AfType::input(config.nss_pull), irq),
            new_dma!(tx_dma, irq),
            new_dma!(rx_dma, irq),
            config,
        )
    }

    /// Create a new SPI slave driver in RX-only mode (only MOSI pin, no MISO).
    pub fn new_rxonly_slave<
        T: Instance,
        D1: RxDma<T>,
        #[cfg(not(afio))] C: CsPin<T> + ExtiPin,
        #[cfg(afio)] C: CsPin<T, A> + ExtiPin,
        #[cfg(afio)] A,
    >(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        cs: Peri<'d, C>,
        rx_dma: Peri<'d, D1>,
        exti: Option<Peri<'d, C::ExtiChannel>>,
        irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<
            <<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ,
            exti::InterruptHandler<<<C as ExtiPin>::ExtiChannel as exti::Channel>::IRQ>,
        > + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::input(config.input_pull)),
            None,
            CsPinType::new_with_exti(cs, exti, AfType::input(config.nss_pull), irq),
            None,
            new_dma!(rx_dma, irq),
            config,
        )
    }
}

impl<'d> Spi<'d, Async, Master> {
    /// Create a new SPI driver.
    pub fn new<T: Instance, D1: TxDma<T>, D2: RxDma<T>, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            new_pin!(miso, AfType::input(config.input_pull)),
            CsPinType::None,
            new_dma!(tx_dma, _irq),
            new_dma!(rx_dma, _irq),
            config,
        )
    }

    /// Create a new SPI driver, in RX-only mode (only MISO pin, no MOSI).
    pub fn new_rxonly<T: Instance, #[cfg(any(spi_v1, spi_v2, spi_v3))] D1: TxDma<T>, D2: RxDma<T>, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        miso: Peri<'d, if_afio!(impl MisoPin<T, A>)>,
        #[cfg(any(spi_v1, spi_v2, spi_v3))] tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        #[cfg(any(spi_v1, spi_v2, spi_v3))] _irq: impl crate::interrupt::typelevel::Binding<
            D1::Interrupt,
            crate::dma::InterruptHandler<D1>,
        > + crate::interrupt::typelevel::Binding<
            D2::Interrupt,
            crate::dma::InterruptHandler<D2>,
        > + 'd,
        #[cfg(any(spi_v4, spi_v5, spi_v6))] _irq: impl crate::interrupt::typelevel::Binding<
            D2::Interrupt,
            crate::dma::InterruptHandler<D2>,
        > + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            None,
            new_pin!(miso, AfType::input(config.input_pull)),
            CsPinType::None,
            #[cfg(any(spi_v1, spi_v2, spi_v3))]
            new_dma!(tx_dma, _irq),
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            None,
            new_dma!(rx_dma, _irq),
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode (only MOSI pin, no MISO).
    pub fn new_txonly<T: Instance, D1: TxDma<T>, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        tx_dma: Peri<'d, D1>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            None,
            CsPinType::None,
            new_dma!(tx_dma, _irq),
            None,
            config,
        )
    }

    /// Create a new SPI driver, in bidirectional mode, specifically in transmit mode
    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    pub fn new_bidi<T: Instance, D1: TxDma<T>, D2: RxDma<T>, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        sck: Peri<'d, if_afio!(impl SckPin<T, A>)>,
        sdio: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
        config: Config,
    ) -> Self {
        let mut this = Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(sdio, AfType::output(OutputType::PushPull, config.gpio_speed)),
            None,
            CsPinType::None,
            new_dma!(tx_dma, _irq),
            new_dma!(rx_dma, _irq),
            config,
        );
        this.set_direction(Some(Direction::Transmit));
        this
    }

    /// Create a new SPI driver, in TX-only mode, without SCK pin.
    ///
    /// This can be useful for bit-banging non-SPI protocols.
    pub fn new_txonly_nosck<T: Instance, D1: TxDma<T>, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        mosi: Peri<'d, if_afio!(impl MosiPin<T, A>)>,
        tx_dma: Peri<'d, D1>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            None,
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.gpio_speed)),
            None,
            CsPinType::None,
            new_dma!(tx_dma, _irq),
            None,
            config,
        )
    }

    #[cfg(stm32wl)]
    /// Useful for on chip peripherals like SUBGHZ which are hardwired.
    pub fn new_subghz<T: Instance, D1: TxDma<T>, D2: RxDma<T>>(
        peri: Peri<'d, T>,
        tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        // see RM0453 rev 1 section 7.2.13 page 291
        // The SUBGHZSPI_SCK frequency is obtained by PCLK3 divided by two.
        // The SUBGHZSPI_SCK clock maximum speed must not exceed 16 MHz.
        let pclk3_freq = <crate::peripherals::SUBGHZSPI as crate::rcc::SealedRccPeripheral>::frequency().0;
        let freq = Hertz(core::cmp::min(pclk3_freq / 2, 16_000_000));
        let mut config = Config::default();
        config.mode = MODE_0;
        config.bit_order = BitOrder::MsbFirst;
        config.frequency = freq;

        Self::new_inner(
            peri,
            None,
            None,
            None,
            CsPinType::None,
            new_dma!(tx_dma, _irq),
            new_dma!(rx_dma, _irq),
            config,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_internal<T: Instance>(
        peri: Peri<'d, T>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Self {
        Self::new_inner(peri, None, None, None, CsPinType::None, tx_dma, rx_dma, config)
    }
}

impl<'d, CM: CommunicationMode> Spi<'d, Async, CM> {
    /// SPI write, using DMA.
    pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        if data.is_empty() {
            return Ok(());
        }

        #[cfg(not(spi_v1))]
        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });
        self.set_word_size(W::CONFIG);

        {
            let tx_dst = self.info.regs.tx_ptr();
            let tx_f = unsafe { self.tx_dma.as_mut().unwrap().write(data, tx_dst, Default::default()) };

            set_txdmaen(self.info.regs, true);

            // Memory barrier after DMA setup to ensure register writes complete before command
            fence(Ordering::SeqCst);

            self.info.regs.cr1().modify(|w| {
                w.set_spe(true);
            });
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            self.info.regs.cr1().modify(|w| {
                w.set_cstart(true);
            });

            let nss_fut = pin!(self.nss.wait_for_edge(SlaveSelectPolarity::from_regs(self.info.regs)));
            match select(tx_f, nss_fut).await {
                Either::Left(((), _)) => {
                    finish_dma(self.info.regs);
                }
                Either::Right(((), _)) => {
                    abort_dma(self.info.regs);

                    return Ok(());
                }
            }
        }

        self.check_transfer_crc()?;

        Ok(())
    }

    /// SPI read, using DMA.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        if data.is_empty() {
            return Ok(());
        }

        let regs = self.info.regs;

        regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        let comm = {
            let mut w = regs.cfg2().read();
            let prev = w.comm();
            w.set_comm(vals::Comm::Receiver);
            regs.cfg2().write_value(w);
            prev
        };

        #[cfg(spi_v4)]
        let i2scfg = {
            let mut w = regs.i2scfgr().read();
            let r = w.i2smod().then(|| {
                let prev = w.i2scfg();
                w.set_i2scfg(match prev {
                    vals::I2scfg::SlaveRx | vals::I2scfg::SlaveFullDuplex => vals::I2scfg::SlaveRx,
                    vals::I2scfg::MasterRx | vals::I2scfg::MasterFullDuplex => vals::I2scfg::MasterRx,
                    _ => panic!("unsupported configuration"),
                });
                prev
            });
            regs.i2scfgr().write_value(w);
            r
        };

        let rx_src = regs.rx_ptr();

        for mut chunk in data.chunks_mut(u16::MAX.into()) {
            set_rxdmaen(regs, true);
            {
                let tsize = chunk.len();

                let transfer = unsafe {
                    self.rx_dma
                        .as_mut()
                        .unwrap()
                        .read(rx_src, &mut chunk, Default::default())
                };

                regs.cr2().modify(|w| {
                    w.set_tsize(tsize as u16);
                });

                // Memory barrier after DMA setup to ensure register writes complete before command
                fence(Ordering::SeqCst);

                regs.cr1().modify(|w| {
                    w.set_spe(true);
                });

                regs.cr1().modify(|w| {
                    w.set_cstart(true);
                });

                let nss_fut = pin!(self.nss.wait_for_edge(SlaveSelectPolarity::from_regs(regs)));
                match select(transfer, nss_fut).await {
                    Either::Left(((), _)) => {
                        finish_dma(regs);
                    }
                    Either::Right(((), _)) => {
                        abort_dma(regs);
                        regs.cfg2().modify(|w| {
                            w.set_comm(comm);
                        });
                        regs.cr2().modify(|w| {
                            w.set_tsize(0);
                        });
                        #[cfg(spi_v4)]
                        if let Some(i2scfg) = i2scfg {
                            regs.i2scfgr().modify(|w| {
                                w.set_i2scfg(i2scfg);
                            });
                        }
                        fence(Ordering::Acquire);

                        return Ok(());
                    }
                }
            }

            self.check_transfer_crc()?;
        }

        regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        regs.cfg2().modify(|w| {
            w.set_comm(comm);
        });

        regs.cr2().modify(|w| {
            w.set_tsize(0);
        });

        #[cfg(spi_v4)]
        if let Some(i2scfg) = i2scfg {
            regs.i2scfgr().modify(|w| {
                w.set_i2scfg(i2scfg);
            });
        }

        // Memory barrier after DMA setup to ensure register writes complete before command
        fence(Ordering::Acquire);

        Ok(())
    }

    /// SPI read, using DMA.
    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        if data.is_empty() {
            return Ok(());
        }

        #[cfg(not(spi_v1))]
        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        flush_rx_fifo(self.info.regs);

        set_rxdmaen(self.info.regs, true);

        let clock_byte_count = data.len();

        let rx_src = self.info.regs.rx_ptr();
        let rx_f = unsafe { self.rx_dma.as_mut().unwrap().read(rx_src, data, Default::default()) };

        let tx_dst = self.info.regs.tx_ptr();
        let clock_byte = W::default();

        {
            let tx_f = self.tx_dma.as_mut().map(|tx_dma| unsafe {
                tx_dma.write_repeated(&clock_byte, clock_byte_count, tx_dst, Default::default())
            });

            if tx_f.is_some() {
                set_txdmaen(self.info.regs, true);
            }

            // Memory barrier after DMA setup to ensure register writes complete before command
            fence(Ordering::SeqCst);

            self.info.regs.cr1().modify(|w| {
                w.set_spe(true);
            });

            let nss_fut = pin!(self.nss.wait_for_edge(SlaveSelectPolarity::from_regs(self.info.regs)));

            if let Some(tx_f) = tx_f {
                match select(join(tx_f, rx_f), nss_fut).await {
                    Either::Left((((), ()), _)) => {
                        finish_dma(self.info.regs);
                    }
                    Either::Right(((), _)) => {
                        abort_dma(self.info.regs);
                        return Ok(());
                    }
                }
            } else {
                match select(rx_f, nss_fut).await {
                    Either::Left(((), _)) => {
                        // In receiving mode RXNE flag should be prefered over BSY flag.
                        // When using DMA the RXNE flag is cleared after DMA reads data.
                        // Since DMA has already finished reading previously specified
                        // amount of data, then there is no need to check for RXNE flag.

                        // The peripheral automatically disables the DMA stream on completion without error,
                        // but it does not clear the RXDMAEN flag in CR2.
                        self.info.regs.cr2().modify(|w| {
                            w.set_rxdmaen(false);
                        });
                    }
                    Either::Right(((), _)) => {
                        abort_dma(self.info.regs);
                        return Ok(());
                    }
                }
            }
        }

        self.check_transfer_crc()?;

        Ok(())
    }

    async fn transfer_inner<W: Word>(&mut self, read: *mut [W], write: *const [W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        assert_eq!(read.len(), write.len());
        if read.len() == 0 {
            return Ok(());
        }

        #[cfg(not(spi_v1))]
        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        // spi_v4 clears rxfifo on SPE=0
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        flush_rx_fifo(self.info.regs);

        set_rxdmaen(self.info.regs, true);

        {
            let rx_src = self.info.regs.rx_ptr::<W>();
            let rx_f = unsafe { self.rx_dma.as_mut().unwrap().read_raw(rx_src, read, Default::default()) };

            let tx_dst: *mut W = self.info.regs.tx_ptr();
            let tx_f = unsafe {
                self.tx_dma
                    .as_mut()
                    .unwrap()
                    .write_raw(write, tx_dst, Default::default())
            };

            set_txdmaen(self.info.regs, true);

            // Memory barrier after DMA setup to ensure register writes complete before command
            fence(Ordering::SeqCst);

            self.info.regs.cr1().modify(|w| {
                w.set_spe(true);
            });
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            self.info.regs.cr1().modify(|w| {
                w.set_cstart(true);
            });

            let nss_fut = pin!(self.nss.wait_for_edge(SlaveSelectPolarity::from_regs(self.info.regs)));
            match select(join(tx_f, rx_f), nss_fut).await {
                Either::Left((((), ()), _)) => {
                    finish_dma(self.info.regs);
                }
                Either::Right(((), _)) => {
                    abort_dma(self.info.regs);

                    return Ok(());
                }
            }
        }

        self.check_transfer_crc()?;

        Ok(())
    }

    /// Bidirectional transfer, using DMA.
    ///
    /// This transfers both buffers at the same time, so it is NOT equivalent to `write` followed by `read`.
    ///
    /// The transfer runs for `max(read.len(), write.len())` bytes. If `read` is shorter extra bytes are ignored.
    /// If `write` is shorter it is padded with zero bytes.
    pub async fn transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();

        self.transfer_inner(read, write).await
    }

    /// In-place bidirectional transfer, using DMA.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub async fn transfer_in_place<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();

        self.transfer_inner(data, data).await
    }
}

impl<'d, M: PeriMode, CM: CommunicationMode> Drop for Spi<'d, M, CM> {
    fn drop(&mut self) {
        self.info.rcc.disable();
    }
}

#[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
use vals::Br;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use vals::Mbr as Br;

fn compute_baud_rate(kernel_clock: Hertz, freq: Hertz) -> Br {
    let val = match kernel_clock.0 / freq.0 {
        0 => panic!("You are trying to reach a frequency higher than the clock"),
        1..=2 => 0b000,
        3..=5 => 0b001,
        6..=11 => 0b010,
        12..=23 => 0b011,
        24..=39 => 0b100,
        40..=95 => 0b101,
        96..=191 => 0b110,
        _ => 0b111,
    };

    Br::from_bits(val)
}

fn compute_frequency(kernel_clock: Hertz, br: Br) -> Hertz {
    let div: u16 = match br {
        Br::Div2 => 2,
        Br::Div4 => 4,
        Br::Div8 => 8,
        Br::Div16 => 16,
        Br::Div32 => 32,
        Br::Div64 => 64,
        Br::Div128 => 128,
        Br::Div256 => 256,
    };

    kernel_clock / div
}

#[cfg(gpio_v2)]
fn set_speed(sck: &Option<Flex<'_>>, mosi: &Option<Flex<'_>>, gpio_speed: Speed) {
    use crate::gpio::SealedPin;

    if let Some(sck) = sck.as_ref() {
        sck.pin.set_speed(gpio_speed);
    }
    if let Some(mosi) = mosi.as_ref() {
        mosi.pin.set_speed(gpio_speed);
    }
}

fn reconfigure(info: &Info, kernel_clock: Hertz, config: &Config) -> Result<(), ()> {
    let cpha = config.raw_phase();
    let cpol = config.raw_polarity();

    let lsbfirst = config.raw_byte_order();

    let br = compute_baud_rate(kernel_clock, config.frequency);

    #[cfg(any(spi_v1, spi_v2, spi_v3))]
    {
        let crc_enabled = !matches!(config.crc, CrcConfig::Disabled);

        info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });
        info.regs.cr1().modify(|w| {
            w.set_cpha(cpha);
            w.set_cpol(cpol);
            w.set_br(br);
            w.set_lsbfirst(lsbfirst);
            w.set_crcen(crc_enabled);
        });
        #[cfg(any(spi_v2_i2s, spi_v3))]
        if crc_enabled {
            info.regs.cr1().modify(|w| {
                match config.crc {
                    CrcConfig::Crc8 { .. } => w.set_crcl(vals::Crcl::Bits8),
                    CrcConfig::Crc16 { .. } => w.set_crcl(vals::Crcl::Bits16),
                    CrcConfig::Disabled => {}
                };
            });
        }
        match config.crc {
            CrcConfig::Crc8 { polynomial } => info.regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u16)),
            CrcConfig::Crc16 { polynomial } => info.regs.crcpoly().write(|w| w.set_crcpoly(polynomial)),
            CrcConfig::Disabled => {}
        }
        info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
    }

    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    {
        let ssiop = config.raw_nss_polarity();

        info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        info.regs.cfg2().modify(|w| {
            w.set_cpha(cpha);
            w.set_cpol(cpol);
            w.set_lsbfirst(lsbfirst);
            w.set_ssiop(ssiop);
        });
        info.regs.cfg1().modify(|w| {
            w.set_mbr(br);
            w.set_crcen(!matches!(config.crc, CrcConfig::Disabled));
        });
        match config.crc {
            CrcConfig::Crc8 { polynomial } => info.regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
            CrcConfig::Crc16 { polynomial } => info.regs.crcpoly().write(|w| w.set_crcpoly(polynomial as u32)),
            CrcConfig::Disabled => {}
        }

        info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
    }
    Ok(())
}

pub(crate) trait RegsExt {
    fn tx_ptr<W>(&self) -> *mut W;
    fn rx_ptr<W>(&self) -> *mut W;
}

impl RegsExt for Regs {
    fn tx_ptr<W>(&self) -> *mut W {
        #[cfg(any(spi_v1, spi_v2))]
        let dr = self.dr();
        #[cfg(spi_v3)]
        let dr = self.dr16();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let dr = self.txdr32();
        dr.as_ptr() as *mut W
    }

    fn rx_ptr<W>(&self) -> *mut W {
        #[cfg(any(spi_v1, spi_v2))]
        let dr = self.dr();
        #[cfg(spi_v3)]
        let dr = self.dr16();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let dr = self.rxdr32();
        dr.as_ptr() as *mut W
    }
}

fn check_error_flags(sr: regs::Sr, ovr: bool) -> Result<(), Error> {
    if sr.ovr() && ovr {
        return Err(Error::Overrun);
    }
    #[cfg(all(any(spi_v1, spi_v2, spi_v3), not(spi_v1_i2s)))]
    if sr.fre() {
        return Err(Error::Framing);
    }
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    if sr.tifre() {
        return Err(Error::Framing);
    }
    if sr.modf() {
        return Err(Error::ModeFault);
    }
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    if sr.crcerr() {
        return Err(Error::Crc);
    }
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    if sr.crce() {
        return Err(Error::Crc);
    }

    Ok(())
}

fn check_tx_ready(regs: Regs, ovr: bool) -> Result<bool, Error> {
    let sr = regs.sr().read();

    check_error_flags(sr, ovr)?;

    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    if sr.txe() {
        return Ok(true);
    }
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    if sr.txp() {
        return Ok(true);
    }

    Ok(false)
}

fn spin_until_tx_ready(regs: Regs, ovr: bool) -> Result<(), Error> {
    loop {
        if check_tx_ready(regs, ovr)? {
            return Ok(());
        }
    }
}

fn check_rx_ready(regs: Regs) -> Result<bool, Error> {
    let sr = regs.sr().read();

    check_error_flags(sr, true)?;

    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    if sr.rxne() {
        return Ok(true);
    }
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    if sr.rxp() {
        return Ok(true);
    }

    Ok(false)
}

#[cfg(any(spi_v1, spi_v2))]
fn spin_until_rx_ready(regs: Regs) -> Result<(), Error> {
    loop {
        if check_rx_ready(regs)? {
            return Ok(());
        }
    }
}

pub(crate) fn flush_rx_fifo(regs: Regs) {
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    while regs.sr().read().rxne() {
        #[cfg(not(spi_v3))]
        let _ = regs.dr().read();
        #[cfg(spi_v3)]
        let _ = regs.dr16().read();
    }
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    while regs.sr().read().rxp() {
        let _ = regs.rxdr32().read();
    }
}

pub(crate) fn set_txdmaen(regs: Regs, val: bool) {
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(val);
    });
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(val);
    });
}

pub(crate) fn set_rxdmaen(regs: Regs, val: bool) {
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    regs.cr2().modify(|reg| {
        reg.set_rxdmaen(val);
    });
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    regs.cfg1().modify(|reg| {
        reg.set_rxdmaen(val);
    });
}

fn finish_dma(regs: Regs) {
    #[cfg(spi_v3)]
    while regs.sr().read().ftlvl().to_bits() > 0 {}

    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    {
        if regs.cr2().read().tsize() == 0 {
            while !regs.sr().read().txc() {}
        } else {
            while !regs.sr().read().eot() {}
        }
    }
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    while regs.sr().read().bsy() {}

    // The peripheral automatically disables the DMA stream on completion without error,
    // but it does not clear the RXDMAEN/TXDMAEN flag in CR2.
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });
}

/// Abort an in-progress DMA transfer and disable SPI to leave hardware in a safe state.
fn abort_dma(regs: Regs) {
    // Disable DMA requests first so the peripheral stops requesting DMA.
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });

    // Disable SPI to abort any ongoing transfer.
    regs.cr1().modify(|w| {
        w.set_spe(false);
    });

    // Flush any stale data in RX FIFO.
    flush_rx_fifo(regs);
}

#[inline]
fn transfer_words<W: Word>(regs: Regs, read: *mut [W], write: *const [W]) -> Result<(), Error> {
    unsafe {
        let ndt = read.len().max(write.len());
        let mut read = read.as_mut().unwrap().iter_mut();
        let mut write = write.as_ref().unwrap().iter();

        let mut w = 0usize;
        let mut r = 0usize;

        if ndt == 0 {
            return Ok(());
        }

        spin_until_tx_ready(regs, true)?;

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        while w < ndt && check_tx_ready(regs, true)? {
            if let Some(word_out) = write.next() {
                ptr::write_volatile(regs.tx_ptr(), *word_out);
            } else {
                ptr::write_volatile(regs.tx_ptr(), W::default());
            }

            w += 1;
        }

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cr1().modify(|reg| reg.set_cstart(true));

        #[cfg(any(spi_v1, spi_v2))]
        let fifo_size = 1;

        #[cfg(spi_v3)]
        let fifo_size = 4 / size_of::<W>();

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let fifo_size = w;

        while w < ndt || r < ndt {
            if w < ndt && (w - r) < fifo_size && check_tx_ready(regs, true)? {
                if let Some(word_out) = write.next() {
                    ptr::write_volatile(regs.tx_ptr(), *word_out);
                } else {
                    ptr::write_volatile(regs.tx_ptr(), W::default());
                }

                w += 1;
            }

            if r < ndt && r < w && check_rx_ready(regs)? {
                if let Some(word_in) = read.next() {
                    *word_in = ptr::read_volatile(regs.rx_ptr());
                } else {
                    ptr::read_volatile::<W>(regs.rx_ptr());
                }

                r += 1;
            }
        }

        Ok(())
    }
}

#[cfg(any(spi_v1, spi_v2))]
fn transfer_word<W: Word>(regs: Regs, tx_word: W) -> Result<W, Error> {
    spin_until_tx_ready(regs, true)?;

    unsafe {
        ptr::write_volatile(regs.tx_ptr(), tx_word);

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cr1().modify(|reg| reg.set_cstart(true));
    }

    spin_until_rx_ready(regs)?;

    let rx_word = unsafe { ptr::read_volatile(regs.rx_ptr()) };
    Ok(rx_word)
}

#[cfg(any(spi_v3, spi_v4, spi_v5, spi_v6))]
fn write_word<W: Word>(regs: Regs, tx_word: W) -> Result<(), Error> {
    // for write, we intentionally ignore the rx fifo, which will cause
    // overrun errors that we have to ignore.
    spin_until_tx_ready(regs, false)?;

    unsafe {
        ptr::write_volatile(regs.tx_ptr(), tx_word);

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        regs.cr1().modify(|reg| reg.set_cstart(true));
    }
    Ok(())
}

// Note: It is not possible to impl these traits generically in embedded-hal 0.2 due to a conflict with
// some marker traits. For details, see https://github.com/rust-embedded/embedded-hal/pull/289
macro_rules! impl_blocking {
    ($w:ident) => {
        impl<'d, M: PeriMode, CM: CommunicationMode> embedded_hal_02::blocking::spi::Write<$w> for Spi<'d, M, CM> {
            type Error = Error;

            fn write(&mut self, words: &[$w]) -> Result<(), Self::Error> {
                self.blocking_write(words)
            }
        }

        impl<'d, M: PeriMode, CM: CommunicationMode> embedded_hal_02::blocking::spi::Transfer<$w> for Spi<'d, M, CM> {
            type Error = Error;

            fn transfer<'w>(&mut self, words: &'w mut [$w]) -> Result<&'w [$w], Self::Error> {
                self.blocking_transfer_in_place(words)?;
                Ok(words)
            }
        }
    };
}

impl_blocking!(u8);
impl_blocking!(u16);

impl<'d, M: PeriMode, CM: CommunicationMode> embedded_hal_1::spi::ErrorType for Spi<'d, M, CM> {
    type Error = Error;
}

impl<'d, W: Word, M: PeriMode, CM: CommunicationMode> embedded_hal_1::spi::SpiBus<W> for Spi<'d, M, CM> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[W]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {
            Self::Framing => embedded_hal_1::spi::ErrorKind::FrameFormat,
            Self::Crc => embedded_hal_1::spi::ErrorKind::Other,
            Self::ModeFault => embedded_hal_1::spi::ErrorKind::ModeFault,
            Self::Overrun => embedded_hal_1::spi::ErrorKind::Overrun,
        }
    }
}

impl<'d, W: Word, CM: CommunicationMode> embedded_hal_async::spi::SpiBus<W> for Spi<'d, Async, CM> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, words: &[W]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    async fn read(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        self.read(words).await
    }

    async fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Self::Error> {
        self.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        self.transfer_in_place(words).await
    }
}

pub(crate) trait SealedWord {
    const CONFIG: word_impl::Config;
}

/// Word sizes usable for SPI.
#[allow(private_bounds)]
pub trait Word: word::Word + SealedWord + Default {}

macro_rules! impl_word {
    ($T:ty, $config:expr) => {
        impl SealedWord for $T {
            const CONFIG: Config = $config;
        }
        impl Word for $T {}
    };
}

#[cfg(any(spi_v1, spi_v2))]
mod word_impl {
    use super::*;

    pub type Config = vals::Dff;

    impl_word!(u8, vals::Dff::Bits8);
    impl_word!(u16, vals::Dff::Bits16);
}

#[cfg(spi_v3)]
mod word_impl {
    use super::*;

    pub type Config = (vals::Ds, vals::Frxth);

    impl_word!(word::U4, (vals::Ds::Bits4, vals::Frxth::Quarter));
    impl_word!(word::U5, (vals::Ds::Bits5, vals::Frxth::Quarter));
    impl_word!(word::U6, (vals::Ds::Bits6, vals::Frxth::Quarter));
    impl_word!(word::U7, (vals::Ds::Bits7, vals::Frxth::Quarter));
    impl_word!(u8, (vals::Ds::Bits8, vals::Frxth::Quarter));
    impl_word!(word::U9, (vals::Ds::Bits9, vals::Frxth::Half));
    impl_word!(word::U10, (vals::Ds::Bits10, vals::Frxth::Half));
    impl_word!(word::U11, (vals::Ds::Bits11, vals::Frxth::Half));
    impl_word!(word::U12, (vals::Ds::Bits12, vals::Frxth::Half));
    impl_word!(word::U13, (vals::Ds::Bits13, vals::Frxth::Half));
    impl_word!(word::U14, (vals::Ds::Bits14, vals::Frxth::Half));
    impl_word!(word::U15, (vals::Ds::Bits15, vals::Frxth::Half));
    impl_word!(u16, (vals::Ds::Bits16, vals::Frxth::Half));
}

#[cfg(any(spi_v4, spi_v5, spi_v6))]
mod word_impl {
    use super::*;

    pub type Config = u8;

    impl_word!(word::U4, 4 - 1);
    impl_word!(word::U5, 5 - 1);
    impl_word!(word::U6, 6 - 1);
    impl_word!(word::U7, 7 - 1);
    impl_word!(u8, 8 - 1);
    impl_word!(word::U9, 9 - 1);
    impl_word!(word::U10, 10 - 1);
    impl_word!(word::U11, 11 - 1);
    impl_word!(word::U12, 12 - 1);
    impl_word!(word::U13, 13 - 1);
    impl_word!(word::U14, 14 - 1);
    impl_word!(word::U15, 15 - 1);
    impl_word!(u16, 16 - 1);
    impl_word!(word::U17, 17 - 1);
    impl_word!(word::U18, 18 - 1);
    impl_word!(word::U19, 19 - 1);
    impl_word!(word::U20, 20 - 1);
    impl_word!(word::U21, 21 - 1);
    impl_word!(word::U22, 22 - 1);
    impl_word!(word::U23, 23 - 1);
    impl_word!(word::U24, 24 - 1);
    impl_word!(word::U25, 25 - 1);
    impl_word!(word::U26, 26 - 1);
    impl_word!(word::U27, 27 - 1);
    impl_word!(word::U28, 28 - 1);
    impl_word!(word::U29, 29 - 1);
    impl_word!(word::U30, 30 - 1);
    impl_word!(word::U31, 31 - 1);
    impl_word!(u32, 32 - 1);
}

struct State {}

impl State {
    #[allow(unused)]
    const fn new() -> Self {
        Self {}
    }
}

peri_trait!();

pin_trait!(SdExtPin, Instance);
pin_trait!(SckPin, Instance, @A);
pin_trait!(MosiPin, Instance, @A);
pin_trait!(MisoPin, Instance, @A);
pin_trait!(CsPin, Instance, @A);
pin_trait!(MckPin, Instance, @A);
pin_trait!(CkPin, Instance, @A);
pin_trait!(WsPin, Instance, @A);
pin_trait!(I2sSdPin, Instance, @A);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);
dma_trait!(RxDmaExt, Instance);

foreach_peripheral!(
    (spi, $inst:ident) => {
        peri_trait_impl!($inst);
    };
);

impl<'d, M: PeriMode, CM: CommunicationMode> SetConfig for Spi<'d, M, CM> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config)
    }
}
