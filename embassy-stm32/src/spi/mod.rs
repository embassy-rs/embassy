//! Serial Peripheral Interface (SPI)
#![macro_use]

use core::marker::PhantomData;
use core::ptr;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use crate::dma::{word, ChannelAndRequest};
use crate::gpio::{AfType, AnyPin, OutputType, Pull, SealedPin as _, Speed};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::spi::{regs, vals, Spi as Regs};
use crate::rcc::{RccInfo, SealedRccPeripheral};
use crate::time::Hertz;
use crate::Peri;

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

/// SPI bit order
#[derive(Copy, Clone)]
pub enum BitOrder {
    /// Least significant bit first.
    LsbFirst,
    /// Most significant bit first.
    MsbFirst,
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
    /// Enable internal pullup on MISO.
    ///
    /// There are some ICs that require a pull-up on the MISO pin for some applications.
    /// If you  are unsure, you probably don't need this.
    pub miso_pull: Pull,
    /// signal rise/fall speed (slew rate) - defaults to `Medium`.
    /// Increase for high SPI speeds. Change to `Low` to reduce ringing.
    pub rise_fall_speed: Speed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            bit_order: BitOrder::MsbFirst,
            frequency: Hertz(1_000_000),
            miso_pull: Pull::None,
            rise_fall_speed: Speed::VeryHigh,
        }
    }
}

impl Config {
    fn raw_phase(&self) -> vals::Cpha {
        match self.mode.phase {
            Phase::CaptureOnSecondTransition => vals::Cpha::SECOND_EDGE,
            Phase::CaptureOnFirstTransition => vals::Cpha::FIRST_EDGE,
        }
    }

    fn raw_polarity(&self) -> vals::Cpol {
        match self.mode.polarity {
            Polarity::IdleHigh => vals::Cpol::IDLE_HIGH,
            Polarity::IdleLow => vals::Cpol::IDLE_LOW,
        }
    }

    fn raw_byte_order(&self) -> vals::Lsbfirst {
        match self.bit_order {
            BitOrder::LsbFirst => vals::Lsbfirst::LSBFIRST,
            BitOrder::MsbFirst => vals::Lsbfirst::MSBFIRST,
        }
    }

    #[cfg(gpio_v1)]
    fn sck_af(&self) -> AfType {
        AfType::output(OutputType::PushPull, self.rise_fall_speed)
    }

    #[cfg(gpio_v2)]
    fn sck_af(&self) -> AfType {
        AfType::output_pull(
            OutputType::PushPull,
            self.rise_fall_speed,
            match self.mode.polarity {
                Polarity::IdleLow => Pull::Down,
                Polarity::IdleHigh => Pull::Up,
            },
        )
    }
}
/// SPI driver.
pub struct Spi<'d, M: PeriMode> {
    pub(crate) info: &'static Info,
    kernel_clock: Hertz,
    sck: Option<Peri<'d, AnyPin>>,
    mosi: Option<Peri<'d, AnyPin>>,
    miso: Option<Peri<'d, AnyPin>>,
    tx_dma: Option<ChannelAndRequest<'d>>,
    rx_dma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
    current_word_size: word_impl::Config,
    rise_fall_speed: Speed,
}

impl<'d, M: PeriMode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Option<Peri<'d, AnyPin>>,
        mosi: Option<Peri<'d, AnyPin>>,
        miso: Option<Peri<'d, AnyPin>>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Self {
        let mut this = Self {
            info: T::info(),
            kernel_clock: T::frequency(),
            sck,
            mosi,
            miso,
            tx_dma,
            rx_dma,
            current_word_size: <u8 as SealedWord>::CONFIG,
            _phantom: PhantomData,
            rise_fall_speed: config.rise_fall_speed,
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

        let regs = self.info.regs;
        #[cfg(any(spi_v1, spi_f1))]
        {
            regs.cr2().modify(|w| {
                w.set_ssoe(false);
            });
            regs.cr1().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);

                w.set_mstr(vals::Mstr::MASTER);
                w.set_br(br);
                w.set_spe(true);
                w.set_lsbfirst(lsbfirst);
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(vals::Bidimode::UNIDIRECTIONAL);
                // we're doing "fake rxonly", by actually writing one
                // byte to TXDR for each byte we want to receive. if we
                // set OUTPUTDISABLED here, this hangs.
                w.set_rxonly(vals::Rxonly::FULL_DUPLEX);
                w.set_dff(<u8 as SealedWord>::CONFIG)
            });
        }
        #[cfg(spi_v2)]
        {
            regs.cr2().modify(|w| {
                let (ds, frxth) = <u8 as SealedWord>::CONFIG;
                w.set_frxth(frxth);
                w.set_ds(ds);
                w.set_ssoe(false);
            });
            regs.cr1().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);

                w.set_mstr(vals::Mstr::MASTER);
                w.set_br(br);
                w.set_lsbfirst(lsbfirst);
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(vals::Bidimode::UNIDIRECTIONAL);
                w.set_spe(true);
            });
        }
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        {
            regs.ifcr().write(|w| w.0 = 0xffff_ffff);
            regs.cfg2().modify(|w| {
                //w.set_ssoe(true);
                w.set_ssoe(false);
                w.set_cpha(cpha);
                w.set_cpol(cpol);
                w.set_lsbfirst(lsbfirst);
                w.set_ssm(true);
                w.set_master(vals::Master::MASTER);
                w.set_comm(vals::Comm::FULL_DUPLEX);
                w.set_ssom(vals::Ssom::ASSERTED);
                w.set_midi(0);
                w.set_mssi(0);
                w.set_afcntr(true);
                w.set_ssiop(vals::Ssiop::ACTIVE_HIGH);
            });
            regs.cfg1().modify(|w| {
                w.set_crcen(false);
                w.set_mbr(br);
                w.set_dsize(<u8 as SealedWord>::CONFIG);
                w.set_fthlv(vals::Fthlv::ONE_FRAME);
            });
            regs.cr2().modify(|w| {
                w.set_tsize(0);
            });
            regs.cr1().modify(|w| {
                w.set_ssi(false);
                w.set_spe(true);
            });
        }
    }

    /// Reconfigures it with the supplied config.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ()> {
        let cpha = config.raw_phase();
        let cpol = config.raw_polarity();

        let lsbfirst = config.raw_byte_order();

        let br = compute_baud_rate(self.kernel_clock, config.frequency);

        #[cfg(gpio_v2)]
        {
            self.rise_fall_speed = config.rise_fall_speed;
            if let Some(sck) = self.sck.as_ref() {
                sck.set_speed(config.rise_fall_speed);
            }
            if let Some(mosi) = self.mosi.as_ref() {
                mosi.set_speed(config.rise_fall_speed);
            }
        }

        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        self.info.regs.cr1().modify(|w| {
            w.set_cpha(cpha);
            w.set_cpol(cpol);
            w.set_br(br);
            w.set_lsbfirst(lsbfirst);
        });

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        {
            self.info.regs.cr1().modify(|w| {
                w.set_spe(false);
            });

            self.info.regs.cfg2().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);
                w.set_lsbfirst(lsbfirst);
            });
            self.info.regs.cfg1().modify(|w| {
                w.set_mbr(br);
            });

            self.info.regs.cr1().modify(|w| {
                w.set_spe(true);
            });
        }
        Ok(())
    }

    /// Get current SPI configuration.
    pub fn get_current_config(&self) -> Config {
        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        let cfg = self.info.regs.cr1().read();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let cfg = self.info.regs.cfg2().read();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let cfg1 = self.info.regs.cfg1().read();

        let polarity = if cfg.cpol() == vals::Cpol::IDLE_LOW {
            Polarity::IdleLow
        } else {
            Polarity::IdleHigh
        };
        let phase = if cfg.cpha() == vals::Cpha::FIRST_EDGE {
            Phase::CaptureOnFirstTransition
        } else {
            Phase::CaptureOnSecondTransition
        };

        let bit_order = if cfg.lsbfirst() == vals::Lsbfirst::LSBFIRST {
            BitOrder::LsbFirst
        } else {
            BitOrder::MsbFirst
        };

        let miso_pull = match &self.miso {
            None => Pull::None,
            Some(pin) => pin.pull(),
        };

        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        let br = cfg.br();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let br = cfg1.mbr();

        let frequency = compute_frequency(self.kernel_clock, br);

        Config {
            mode: Mode { polarity, phase },
            bit_order,
            frequency,
            miso_pull,
            rise_fall_speed: self.rise_fall_speed,
        }
    }

    pub(crate) fn set_word_size(&mut self, word_size: word_impl::Config) {
        if self.current_word_size == word_size {
            return;
        }

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        #[cfg(any(spi_v1, spi_f1))]
        self.info.regs.cr1().modify(|reg| {
            reg.set_dff(word_size);
        });
        #[cfg(spi_v2)]
        self.info.regs.cr2().modify(|w| {
            w.set_frxth(word_size.1);
            w.set_ds(word_size.0);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cfg1().modify(|w| {
            w.set_dsize(word_size);
        });

        self.current_word_size = word_size;
    }

    /// Blocking write.
    pub fn blocking_write<W: Word>(&mut self, words: &[W]) -> Result<(), Error> {
        // needed in v3+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);
        for word in words.iter() {
            // this cannot use `transfer_word` because on SPIv2 and higher,
            // the SPI RX state machine hangs if no physical pin is connected to the SCK AF.
            // This is the case when the SPI has been created with `new_(blocking_?)txonly_nosck`.
            // See https://github.com/embassy-rs/embassy/issues/2902
            // This is not documented as an errata by ST, and I've been unable to find anything online...
            #[cfg(not(any(spi_v1, spi_f1)))]
            write_word(self.info.regs, *word)?;

            // if we're doing tx only, after writing the last byte to FIFO we have to wait
            // until it's actually sent. On SPIv1 you're supposed to use the BSY flag for this
            // but apparently it's broken, it clears too soon. Workaround is to wait for RXNE:
            // when it gets set you know the transfer is done, even if you don't care about rx.
            // Luckily this doesn't affect SPIv2+.
            // See http://efton.sk/STM32/gotcha/g68.html
            // ST doesn't seem to document this in errata sheets (?)
            #[cfg(any(spi_v1, spi_f1))]
            transfer_word(self.info.regs, *word)?;
        }

        // wait until last word is transmitted. (except on v1, see above)
        #[cfg(not(any(spi_v1, spi_f1, spi_v2)))]
        while !self.info.regs.sr().read().txc() {}
        #[cfg(spi_v2)]
        while self.info.regs.sr().read().bsy() {}

        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        // needed in v3+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);
        for word in words.iter_mut() {
            *word = transfer_word(self.info.regs, W::default())?;
        }
        Ok(())
    }

    /// Blocking in-place bidirectional transfer.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub fn blocking_transfer_in_place<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        // needed in v3+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);
        for word in words.iter_mut() {
            *word = transfer_word(self.info.regs, *word)?;
        }
        Ok(())
    }

    /// Blocking bidirectional transfer.
    ///
    /// This transfers both buffers at the same time, so it is NOT equivalent to `write` followed by `read`.
    ///
    /// The transfer runs for `max(read.len(), write.len())` bytes. If `read` is shorter extra bytes are ignored.
    /// If `write` is shorter it is padded with zero bytes.
    pub fn blocking_transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        // needed in v3+ to avoid overrun causing the SPI RX state machine to get stuck...?
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        self.info.regs.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(self.info.regs);
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or_default();
            let rb = transfer_word(self.info.regs, wb)?;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        Ok(())
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create a new blocking SPI driver.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            new_pin!(miso, AfType::input(config.miso_pull)),
            None,
            None,
            config,
        )
    }

    /// Create a new blocking SPI driver, in RX-only mode (only MISO pin, no MOSI).
    pub fn new_blocking_rxonly<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            None,
            new_pin!(miso, AfType::input(config.miso_pull)),
            None,
            None,
            config,
        )
    }

    /// Create a new blocking SPI driver, in TX-only mode (only MOSI pin, no MISO).
    pub fn new_blocking_txonly<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode, without SCK pin.
    ///
    /// This can be useful for bit-banging non-SPI protocols.
    pub fn new_blocking_txonly_nosck<T: Instance>(
        peri: Peri<'d, T>,
        mosi: Peri<'d, impl MosiPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            None,
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            None,
            None,
            None,
            config,
        )
    }
}

impl<'d> Spi<'d, Async> {
    /// Create a new SPI driver.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            new_pin!(miso, AfType::input(config.miso_pull)),
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }

    /// Create a new SPI driver, in RX-only mode (only MISO pin, no MOSI).
    pub fn new_rxonly<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        #[cfg(any(spi_v1, spi_f1, spi_v2))] tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            None,
            new_pin!(miso, AfType::input(config.miso_pull)),
            #[cfg(any(spi_v1, spi_f1, spi_v2))]
            new_dma!(tx_dma),
            #[cfg(any(spi_v3, spi_v4, spi_v5))]
            None,
            new_dma!(rx_dma),
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode (only MOSI pin, no MISO).
    pub fn new_txonly<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sck, config.sck_af()),
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            None,
            new_dma!(tx_dma),
            None,
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode, without SCK pin.
    ///
    /// This can be useful for bit-banging non-SPI protocols.
    pub fn new_txonly_nosck<T: Instance>(
        peri: Peri<'d, T>,
        mosi: Peri<'d, impl MosiPin<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            None,
            new_pin!(mosi, AfType::output(OutputType::PushPull, config.rise_fall_speed)),
            None,
            new_dma!(tx_dma),
            None,
            config,
        )
    }

    #[cfg(stm32wl)]
    /// Useful for on chip peripherals like SUBGHZ which are hardwired.
    pub fn new_subghz<T: Instance>(
        peri: Peri<'d, T>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
    ) -> Self {
        // see RM0453 rev 1 section 7.2.13 page 291
        // The SUBGHZSPI_SCK frequency is obtained by PCLK3 divided by two.
        // The SUBGHZSPI_SCK clock maximum speed must not exceed 16 MHz.
        let pclk3_freq = <crate::peripherals::SUBGHZSPI as SealedRccPeripheral>::frequency().0;
        let freq = Hertz(core::cmp::min(pclk3_freq / 2, 16_000_000));
        let mut config = Config::default();
        config.mode = MODE_0;
        config.bit_order = BitOrder::MsbFirst;
        config.frequency = freq;

        Self::new_inner(peri, None, None, None, new_dma!(tx_dma), new_dma!(rx_dma), config)
    }

    #[allow(dead_code)]
    pub(crate) fn new_internal<T: Instance>(
        peri: Peri<'d, T>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Self {
        Self::new_inner(peri, None, None, None, tx_dma, rx_dma, config)
    }

    /// SPI write, using DMA.
    pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });
        self.set_word_size(W::CONFIG);

        let tx_dst = self.info.regs.tx_ptr();
        let tx_f = unsafe { self.tx_dma.as_mut().unwrap().write(data, tx_dst, Default::default()) };

        set_txdmaen(self.info.regs, true);
        self.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| {
            w.set_cstart(true);
        });

        tx_f.await;

        finish_dma(self.info.regs);

        Ok(())
    }

    /// SPI read, using DMA.
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        let regs = self.info.regs;

        regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        let comm = regs.cfg2().modify(|w| {
            let prev = w.comm();
            w.set_comm(vals::Comm::RECEIVER);
            prev
        });

        #[cfg(spi_v3)]
        let i2scfg = regs.i2scfgr().modify(|w| {
            w.i2smod().then(|| {
                let prev = w.i2scfg();
                w.set_i2scfg(match prev {
                    vals::I2scfg::SLAVE_RX | vals::I2scfg::SLAVE_FULL_DUPLEX => vals::I2scfg::SLAVE_RX,
                    vals::I2scfg::MASTER_RX | vals::I2scfg::MASTER_FULL_DUPLEX => vals::I2scfg::MASTER_RX,
                    _ => panic!("unsupported configuration"),
                });
                prev
            })
        });

        let rx_src = regs.rx_ptr();

        for mut chunk in data.chunks_mut(u16::max_value().into()) {
            set_rxdmaen(regs, true);

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

            regs.cr1().modify(|w| {
                w.set_spe(true);
            });

            regs.cr1().modify(|w| {
                w.set_cstart(true);
            });

            transfer.await;

            finish_dma(regs);
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

        #[cfg(spi_v3)]
        if let Some(i2scfg) = i2scfg {
            regs.i2scfgr().modify(|w| {
                w.set_i2scfg(i2scfg);
            });
        }

        Ok(())
    }

    /// SPI read, using DMA.
    #[cfg(any(spi_v1, spi_f1, spi_v2))]
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        // SPIv3 clears rxfifo on SPE=0
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        flush_rx_fifo(self.info.regs);

        set_rxdmaen(self.info.regs, true);

        let clock_byte_count = data.len();

        let rx_src = self.info.regs.rx_ptr();
        let rx_f = unsafe { self.rx_dma.as_mut().unwrap().read(rx_src, data, Default::default()) };

        let tx_dst = self.info.regs.tx_ptr();
        let clock_byte = W::default();
        let tx_f = unsafe {
            self.tx_dma
                .as_mut()
                .unwrap()
                .write_repeated(&clock_byte, clock_byte_count, tx_dst, Default::default())
        };

        set_txdmaen(self.info.regs, true);
        self.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| {
            w.set_cstart(true);
        });

        join(tx_f, rx_f).await;

        finish_dma(self.info.regs);

        Ok(())
    }

    async fn transfer_inner<W: Word>(&mut self, read: *mut [W], write: *const [W]) -> Result<(), Error> {
        assert_eq!(read.len(), write.len());
        if read.len() == 0 {
            return Ok(());
        }

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        // SPIv3 clears rxfifo on SPE=0
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        flush_rx_fifo(self.info.regs);

        set_rxdmaen(self.info.regs, true);

        let rx_src = self.info.regs.rx_ptr();
        let rx_f = unsafe { self.rx_dma.as_mut().unwrap().read_raw(rx_src, read, Default::default()) };

        let tx_dst: *mut W = self.info.regs.tx_ptr();
        let tx_f = unsafe {
            self.tx_dma
                .as_mut()
                .unwrap()
                .write_raw(write, tx_dst, Default::default())
        };

        set_txdmaen(self.info.regs, true);
        self.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.info.regs.cr1().modify(|w| {
            w.set_cstart(true);
        });

        join(tx_f, rx_f).await;

        finish_dma(self.info.regs);

        Ok(())
    }

    /// Bidirectional transfer, using DMA.
    ///
    /// This transfers both buffers at the same time, so it is NOT equivalent to `write` followed by `read`.
    ///
    /// The transfer runs for `max(read.len(), write.len())` bytes. If `read` is shorter extra bytes are ignored.
    /// If `write` is shorter it is padded with zero bytes.
    pub async fn transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        self.transfer_inner(read, write).await
    }

    /// In-place bidirectional transfer, using DMA.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub async fn transfer_in_place<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        self.transfer_inner(data, data).await
    }
}

impl<'d, M: PeriMode> Drop for Spi<'d, M> {
    fn drop(&mut self) {
        self.sck.as_ref().map(|x| x.set_as_disconnected());
        self.mosi.as_ref().map(|x| x.set_as_disconnected());
        self.miso.as_ref().map(|x| x.set_as_disconnected());

        self.info.rcc.disable();
    }
}

#[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
use vals::Br;
#[cfg(any(spi_v3, spi_v4, spi_v5))]
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
        Br::DIV2 => 2,
        Br::DIV4 => 4,
        Br::DIV8 => 8,
        Br::DIV16 => 16,
        Br::DIV32 => 32,
        Br::DIV64 => 64,
        Br::DIV128 => 128,
        Br::DIV256 => 256,
    };

    kernel_clock / div
}

pub(crate) trait RegsExt {
    fn tx_ptr<W>(&self) -> *mut W;
    fn rx_ptr<W>(&self) -> *mut W;
}

impl RegsExt for Regs {
    fn tx_ptr<W>(&self) -> *mut W {
        #[cfg(any(spi_v1, spi_f1))]
        let dr = self.dr();
        #[cfg(spi_v2)]
        let dr = self.dr16();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let dr = self.txdr32();
        dr.as_ptr() as *mut W
    }

    fn rx_ptr<W>(&self) -> *mut W {
        #[cfg(any(spi_v1, spi_f1))]
        let dr = self.dr();
        #[cfg(spi_v2)]
        let dr = self.dr16();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let dr = self.rxdr32();
        dr.as_ptr() as *mut W
    }
}

fn check_error_flags(sr: regs::Sr, ovr: bool) -> Result<(), Error> {
    if sr.ovr() && ovr {
        return Err(Error::Overrun);
    }
    #[cfg(not(any(spi_f1, spi_v3, spi_v4, spi_v5)))]
    if sr.fre() {
        return Err(Error::Framing);
    }
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    if sr.tifre() {
        return Err(Error::Framing);
    }
    if sr.modf() {
        return Err(Error::ModeFault);
    }
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    if sr.crcerr() {
        return Err(Error::Crc);
    }
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    if sr.crce() {
        return Err(Error::Crc);
    }

    Ok(())
}

fn spin_until_tx_ready(regs: Regs, ovr: bool) -> Result<(), Error> {
    loop {
        let sr = regs.sr().read();

        check_error_flags(sr, ovr)?;

        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        if sr.txe() {
            return Ok(());
        }
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        if sr.txp() {
            return Ok(());
        }
    }
}

fn spin_until_rx_ready(regs: Regs) -> Result<(), Error> {
    loop {
        let sr = regs.sr().read();

        check_error_flags(sr, true)?;

        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        if sr.rxne() {
            return Ok(());
        }
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        if sr.rxp() {
            return Ok(());
        }
    }
}

pub(crate) fn flush_rx_fifo(regs: Regs) {
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    while regs.sr().read().rxne() {
        #[cfg(not(spi_v2))]
        let _ = regs.dr().read();
        #[cfg(spi_v2)]
        let _ = regs.dr16().read();
    }
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    while regs.sr().read().rxp() {
        let _ = regs.rxdr32().read();
    }
}

pub(crate) fn set_txdmaen(regs: Regs, val: bool) {
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(val);
    });
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(val);
    });
}

pub(crate) fn set_rxdmaen(regs: Regs, val: bool) {
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    regs.cr2().modify(|reg| {
        reg.set_rxdmaen(val);
    });
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    regs.cfg1().modify(|reg| {
        reg.set_rxdmaen(val);
    });
}

fn finish_dma(regs: Regs) {
    #[cfg(spi_v2)]
    while regs.sr().read().ftlvl().to_bits() > 0 {}

    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    {
        if regs.cr2().read().tsize() == 0 {
            while !regs.sr().read().txc() {}
        } else {
            while !regs.sr().read().eot() {}
        }
    }
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    while regs.sr().read().bsy() {}

    // Disable the spi peripheral
    regs.cr1().modify(|w| {
        w.set_spe(false);
    });

    // The peripheral automatically disables the DMA stream on completion without error,
    // but it does not clear the RXDMAEN/TXDMAEN flag in CR2.
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(false);
        reg.set_rxdmaen(false);
    });
}

fn transfer_word<W: Word>(regs: Regs, tx_word: W) -> Result<W, Error> {
    spin_until_tx_ready(regs, true)?;

    unsafe {
        ptr::write_volatile(regs.tx_ptr(), tx_word);

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        regs.cr1().modify(|reg| reg.set_cstart(true));
    }

    spin_until_rx_ready(regs)?;

    let rx_word = unsafe { ptr::read_volatile(regs.rx_ptr()) };
    Ok(rx_word)
}

#[allow(unused)] // unused in SPIv1
fn write_word<W: Word>(regs: Regs, tx_word: W) -> Result<(), Error> {
    // for write, we intentionally ignore the rx fifo, which will cause
    // overrun errors that we have to ignore.
    spin_until_tx_ready(regs, false)?;

    unsafe {
        ptr::write_volatile(regs.tx_ptr(), tx_word);

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        regs.cr1().modify(|reg| reg.set_cstart(true));
    }
    Ok(())
}

// Note: It is not possible to impl these traits generically in embedded-hal 0.2 due to a conflict with
// some marker traits. For details, see https://github.com/rust-embedded/embedded-hal/pull/289
macro_rules! impl_blocking {
    ($w:ident) => {
        impl<'d, M: PeriMode> embedded_hal_02::blocking::spi::Write<$w> for Spi<'d, M> {
            type Error = Error;

            fn write(&mut self, words: &[$w]) -> Result<(), Self::Error> {
                self.blocking_write(words)
            }
        }

        impl<'d, M: PeriMode> embedded_hal_02::blocking::spi::Transfer<$w> for Spi<'d, M> {
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

impl<'d, M: PeriMode> embedded_hal_1::spi::ErrorType for Spi<'d, M> {
    type Error = Error;
}

impl<'d, W: Word, M: PeriMode> embedded_hal_1::spi::SpiBus<W> for Spi<'d, M> {
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

impl<'d, W: Word> embedded_hal_async::spi::SpiBus<W> for Spi<'d, Async> {
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

#[cfg(any(spi_v1, spi_f1))]
mod word_impl {
    use super::*;

    pub type Config = vals::Dff;

    impl_word!(u8, vals::Dff::BITS8);
    impl_word!(u16, vals::Dff::BITS16);
}

#[cfg(spi_v2)]
mod word_impl {
    use super::*;

    pub type Config = (vals::Ds, vals::Frxth);

    impl_word!(word::U4, (vals::Ds::BITS4, vals::Frxth::QUARTER));
    impl_word!(word::U5, (vals::Ds::BITS5, vals::Frxth::QUARTER));
    impl_word!(word::U6, (vals::Ds::BITS6, vals::Frxth::QUARTER));
    impl_word!(word::U7, (vals::Ds::BITS7, vals::Frxth::QUARTER));
    impl_word!(u8, (vals::Ds::BITS8, vals::Frxth::QUARTER));
    impl_word!(word::U9, (vals::Ds::BITS9, vals::Frxth::HALF));
    impl_word!(word::U10, (vals::Ds::BITS10, vals::Frxth::HALF));
    impl_word!(word::U11, (vals::Ds::BITS11, vals::Frxth::HALF));
    impl_word!(word::U12, (vals::Ds::BITS12, vals::Frxth::HALF));
    impl_word!(word::U13, (vals::Ds::BITS13, vals::Frxth::HALF));
    impl_word!(word::U14, (vals::Ds::BITS14, vals::Frxth::HALF));
    impl_word!(word::U15, (vals::Ds::BITS15, vals::Frxth::HALF));
    impl_word!(u16, (vals::Ds::BITS16, vals::Frxth::HALF));
}

#[cfg(any(spi_v3, spi_v4, spi_v5))]
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

pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) rcc: RccInfo,
}

struct State {}

impl State {
    #[allow(unused)]
    const fn new() -> Self {
        Self {}
    }
}

peri_trait!();

pin_trait!(SckPin, Instance);
pin_trait!(MosiPin, Instance);
pin_trait!(MisoPin, Instance);
pin_trait!(CsPin, Instance);
pin_trait!(MckPin, Instance);
pin_trait!(CkPin, Instance);
pin_trait!(WsPin, Instance);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);

foreach_peripheral!(
    (spi, $inst:ident) => {
        peri_trait_impl!($inst, Info {
            regs: crate::pac::$inst,
            rcc: crate::peripherals::$inst::RCC_INFO,
        });
    };
);

impl<'d, M: PeriMode> SetConfig for Spi<'d, M> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config)
    }
}
