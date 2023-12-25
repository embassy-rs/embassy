//! Serial Peripheral Interface (SPI)
#![macro_use]

use core::ptr;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use crate::dma::{slice_ptr_parts, word, Transfer};
use crate::gpio::sealed::{AFType, Pin as _};
use crate::gpio::{AnyPin, Pull};
use crate::pac::spi::{regs, vals, Spi as Regs};
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

/// SPI error.
#[derive(Debug, PartialEq, Eq)]
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            bit_order: BitOrder::MsbFirst,
            frequency: Hertz(1_000_000),
        }
    }
}

impl Config {
    fn raw_phase(&self) -> vals::Cpha {
        match self.mode.phase {
            Phase::CaptureOnSecondTransition => vals::Cpha::SECONDEDGE,
            Phase::CaptureOnFirstTransition => vals::Cpha::FIRSTEDGE,
        }
    }

    fn raw_polarity(&self) -> vals::Cpol {
        match self.mode.polarity {
            Polarity::IdleHigh => vals::Cpol::IDLEHIGH,
            Polarity::IdleLow => vals::Cpol::IDLELOW,
        }
    }

    fn raw_byte_order(&self) -> vals::Lsbfirst {
        match self.bit_order {
            BitOrder::LsbFirst => vals::Lsbfirst::LSBFIRST,
            BitOrder::MsbFirst => vals::Lsbfirst::MSBFIRST,
        }
    }
}

/// SPI driver.
pub struct Spi<'d, T: Instance, Tx, Rx> {
    _peri: PeripheralRef<'d, T>,
    sck: Option<PeripheralRef<'d, AnyPin>>,
    mosi: Option<PeripheralRef<'d, AnyPin>>,
    miso: Option<PeripheralRef<'d, AnyPin>>,
    txdma: PeripheralRef<'d, Tx>,
    rxdma: PeripheralRef<'d, Rx>,
    current_word_size: word_impl::Config,
}

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    /// Create a new SPI driver.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T>> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T>> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, sck, mosi, miso);

        let sck_pull_mode = match config.mode.polarity {
            Polarity::IdleLow => Pull::Down,
            Polarity::IdleHigh => Pull::Up,
        };

        sck.set_as_af_pull(sck.af_num(), AFType::OutputPushPull, sck_pull_mode);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        mosi.set_as_af(mosi.af_num(), AFType::OutputPushPull);
        mosi.set_speed(crate::gpio::Speed::VeryHigh);
        miso.set_as_af(miso.af_num(), AFType::Input);
        miso.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(sck.map_into()),
            Some(mosi.map_into()),
            Some(miso.map_into()),
            txdma,
            rxdma,
            config,
        )
    }

    /// Create a new SPI driver, in RX-only mode (only MISO pin, no MOSI).
    pub fn new_rxonly(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T>> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd, // TODO remove
        rxdma: impl Peripheral<P = Rx> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, miso);
        sck.set_as_af(sck.af_num(), AFType::OutputPushPull);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        miso.set_as_af(miso.af_num(), AFType::Input);
        miso.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(sck.map_into()),
            None,
            Some(miso.map_into()),
            txdma,
            rxdma,
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode (only MOSI pin, no MISO).
    pub fn new_txonly(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T>> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd, // TODO remove
        config: Config,
    ) -> Self {
        into_ref!(sck, mosi);
        sck.set_as_af(sck.af_num(), AFType::OutputPushPull);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        mosi.set_as_af(mosi.af_num(), AFType::OutputPushPull);
        mosi.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(sck.map_into()),
            Some(mosi.map_into()),
            None,
            txdma,
            rxdma,
            config,
        )
    }

    /// Create a new SPI driver, in TX-only mode, without SCK pin.
    ///
    /// This can be useful for bit-banging non-SPI protocols.
    pub fn new_txonly_nosck(
        peri: impl Peripheral<P = T> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T>> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd, // TODO: remove
        config: Config,
    ) -> Self {
        into_ref!(mosi);
        mosi.set_as_af_pull(mosi.af_num(), AFType::OutputPushPull, Pull::Down);
        mosi.set_speed(crate::gpio::Speed::Medium);

        Self::new_inner(peri, None, Some(mosi.map_into()), None, txdma, rxdma, config)
    }

    #[cfg(stm32wl)]
    /// Useful for on chip peripherals like SUBGHZ which are hardwired.
    pub fn new_subghz(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
    ) -> Self {
        // see RM0453 rev 1 section 7.2.13 page 291
        // The SUBGHZSPI_SCK frequency is obtained by PCLK3 divided by two.
        // The SUBGHZSPI_SCK clock maximum speed must not exceed 16 MHz.
        let pclk3_freq = <peripherals::SUBGHZSPI as crate::rcc::sealed::RccPeripheral>::frequency().0;
        let freq = Hertz(core::cmp::min(pclk3_freq / 2, 16_000_000));
        let mut config = Config::default();
        config.mode = MODE_0;
        config.bit_order = BitOrder::MsbFirst;
        config.frequency = freq;
        Self::new_inner(peri, None, None, None, txdma, rxdma, config)
    }

    #[allow(dead_code)]
    pub(crate) fn new_internal(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(peri, None, None, None, txdma, rxdma, config)
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, txdma, rxdma);

        let pclk = T::frequency();
        let freq = config.frequency;
        let br = compute_baud_rate(pclk, freq);

        let cpha = config.raw_phase();
        let cpol = config.raw_polarity();

        let lsbfirst = config.raw_byte_order();

        T::enable_and_reset();

        #[cfg(any(spi_v1, spi_f1))]
        {
            T::REGS.cr2().modify(|w| {
                w.set_ssoe(false);
            });
            T::REGS.cr1().modify(|w| {
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
                if mosi.is_none() {
                    w.set_rxonly(vals::Rxonly::OUTPUTDISABLED);
                }
                w.set_dff(<u8 as sealed::Word>::CONFIG)
            });
        }
        #[cfg(spi_v2)]
        {
            T::REGS.cr2().modify(|w| {
                let (ds, frxth) = <u8 as sealed::Word>::CONFIG;
                w.set_frxth(frxth);
                w.set_ds(ds);
                w.set_ssoe(false);
            });
            T::REGS.cr1().modify(|w| {
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
            T::REGS.ifcr().write(|w| w.0 = 0xffff_ffff);
            T::REGS.cfg2().modify(|w| {
                //w.set_ssoe(true);
                w.set_ssoe(false);
                w.set_cpha(cpha);
                w.set_cpol(cpol);
                w.set_lsbfirst(lsbfirst);
                w.set_ssm(true);
                w.set_master(vals::Master::MASTER);
                w.set_comm(vals::Comm::FULLDUPLEX);
                w.set_ssom(vals::Ssom::ASSERTED);
                w.set_midi(0);
                w.set_mssi(0);
                w.set_afcntr(true);
                w.set_ssiop(vals::Ssiop::ACTIVEHIGH);
            });
            T::REGS.cfg1().modify(|w| {
                w.set_crcen(false);
                w.set_mbr(br);
                w.set_dsize(<u8 as sealed::Word>::CONFIG);
                w.set_fthlv(vals::Fthlv::ONEFRAME);
            });
            T::REGS.cr2().modify(|w| {
                w.set_tsize(0);
            });
            T::REGS.cr1().modify(|w| {
                w.set_ssi(false);
                w.set_spe(true);
            });
        }

        Self {
            _peri: peri,
            sck,
            mosi,
            miso,
            txdma,
            rxdma,
            current_word_size: <u8 as sealed::Word>::CONFIG,
        }
    }

    /// Reconfigures it with the supplied config.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ()> {
        let cpha = config.raw_phase();
        let cpol = config.raw_polarity();

        let lsbfirst = config.raw_byte_order();

        let pclk = T::frequency();
        let freq = config.frequency;
        let br = compute_baud_rate(pclk, freq);

        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        T::REGS.cr1().modify(|w| {
            w.set_cpha(cpha);
            w.set_cpol(cpol);
            w.set_br(br);
            w.set_lsbfirst(lsbfirst);
        });

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        {
            T::REGS.cfg2().modify(|w| {
                w.set_cpha(cpha);
                w.set_cpol(cpol);
                w.set_lsbfirst(lsbfirst);
            });
            T::REGS.cfg1().modify(|w| {
                w.set_mbr(br);
            });
        }
        Ok(())
    }

    /// Get current SPI configuration.
    pub fn get_current_config(&self) -> Config {
        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        let cfg = T::REGS.cr1().read();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let cfg = T::REGS.cfg2().read();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let cfg1 = T::REGS.cfg1().read();

        let polarity = if cfg.cpol() == vals::Cpol::IDLELOW {
            Polarity::IdleLow
        } else {
            Polarity::IdleHigh
        };
        let phase = if cfg.cpha() == vals::Cpha::FIRSTEDGE {
            Phase::CaptureOnFirstTransition
        } else {
            Phase::CaptureOnSecondTransition
        };

        let bit_order = if cfg.lsbfirst() == vals::Lsbfirst::LSBFIRST {
            BitOrder::LsbFirst
        } else {
            BitOrder::MsbFirst
        };

        #[cfg(any(spi_v1, spi_f1, spi_v2))]
        let br = cfg.br();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let br = cfg1.mbr();

        let pclk = T::frequency();
        let frequency = compute_frequency(pclk, br);

        Config {
            mode: Mode { polarity, phase },
            bit_order,
            frequency,
        }
    }

    fn set_word_size(&mut self, word_size: word_impl::Config) {
        if self.current_word_size == word_size {
            return;
        }

        #[cfg(any(spi_v1, spi_f1))]
        {
            T::REGS.cr1().modify(|reg| {
                reg.set_spe(false);
                reg.set_dff(word_size)
            });
            T::REGS.cr1().modify(|reg| {
                reg.set_spe(true);
            });
        }
        #[cfg(spi_v2)]
        {
            T::REGS.cr1().modify(|w| {
                w.set_spe(false);
            });
            T::REGS.cr2().modify(|w| {
                w.set_frxth(word_size.1);
                w.set_ds(word_size.0);
            });
            T::REGS.cr1().modify(|w| {
                w.set_spe(true);
            });
        }
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        {
            T::REGS.cr1().modify(|w| {
                w.set_csusp(true);
            });
            while T::REGS.sr().read().eot() {}
            T::REGS.cr1().modify(|w| {
                w.set_spe(false);
            });
            T::REGS.cfg1().modify(|w| {
                w.set_dsize(word_size);
            });
            T::REGS.cr1().modify(|w| {
                w.set_csusp(false);
                w.set_spe(true);
            });
        }

        self.current_word_size = word_size;
    }

    /// SPI write, using DMA.
    pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
    {
        if data.is_empty() {
            return Ok(());
        }

        self.set_word_size(W::CONFIG);
        T::REGS.cr1().modify(|w| {
            w.set_spe(false);
        });

        let tx_request = self.txdma.request();
        let tx_dst = T::REGS.tx_ptr();
        let tx_f = unsafe { Transfer::new_write(&mut self.txdma, tx_request, data, tx_dst, Default::default()) };

        set_txdmaen(T::REGS, true);
        T::REGS.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        T::REGS.cr1().modify(|w| {
            w.set_cstart(true);
        });

        tx_f.await;

        finish_dma(T::REGS);

        Ok(())
    }

    /// SPI read, using DMA.
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        if data.is_empty() {
            return Ok(());
        }

        self.set_word_size(W::CONFIG);
        T::REGS.cr1().modify(|w| {
            w.set_spe(false);
        });

        // SPIv3 clears rxfifo on SPE=0
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        flush_rx_fifo(T::REGS);

        set_rxdmaen(T::REGS, true);

        let clock_byte_count = data.len();

        let rx_request = self.rxdma.request();
        let rx_src = T::REGS.rx_ptr();
        let rx_f = unsafe { Transfer::new_read(&mut self.rxdma, rx_request, rx_src, data, Default::default()) };

        let tx_request = self.txdma.request();
        let tx_dst = T::REGS.tx_ptr();
        let clock_byte = 0x00u8;
        let tx_f = unsafe {
            Transfer::new_write_repeated(
                &mut self.txdma,
                tx_request,
                &clock_byte,
                clock_byte_count,
                tx_dst,
                Default::default(),
            )
        };

        set_txdmaen(T::REGS, true);
        T::REGS.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        T::REGS.cr1().modify(|w| {
            w.set_cstart(true);
        });

        join(tx_f, rx_f).await;

        finish_dma(T::REGS);

        Ok(())
    }

    async fn transfer_inner<W: Word>(&mut self, read: *mut [W], write: *const [W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        let (_, rx_len) = slice_ptr_parts(read);
        let (_, tx_len) = slice_ptr_parts(write);
        assert_eq!(rx_len, tx_len);
        if rx_len == 0 {
            return Ok(());
        }

        self.set_word_size(W::CONFIG);
        T::REGS.cr1().modify(|w| {
            w.set_spe(false);
        });

        // SPIv3 clears rxfifo on SPE=0
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        flush_rx_fifo(T::REGS);

        set_rxdmaen(T::REGS, true);

        let rx_request = self.rxdma.request();
        let rx_src = T::REGS.rx_ptr();
        let rx_f = unsafe { Transfer::new_read_raw(&mut self.rxdma, rx_request, rx_src, read, Default::default()) };

        let tx_request = self.txdma.request();
        let tx_dst = T::REGS.tx_ptr();
        let tx_f = unsafe { Transfer::new_write_raw(&mut self.txdma, tx_request, write, tx_dst, Default::default()) };

        set_txdmaen(T::REGS, true);
        T::REGS.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        T::REGS.cr1().modify(|w| {
            w.set_cstart(true);
        });

        join(tx_f, rx_f).await;

        finish_dma(T::REGS);

        Ok(())
    }

    /// Bidirectional transfer, using DMA.
    ///
    /// This transfers both buffers at the same time, so it is NOT equivalent to `write` followed by `read`.
    ///
    /// The transfer runs for `max(read.len(), write.len())` bytes. If `read` is shorter extra bytes are ignored.
    /// If `write` is shorter it is padded with zero bytes.
    pub async fn transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        self.transfer_inner(read, write).await
    }

    /// In-place bidirectional transfer, using DMA.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub async fn transfer_in_place<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        self.transfer_inner(data, data).await
    }

    /// Blocking write.
    pub fn blocking_write<W: Word>(&mut self, words: &[W]) -> Result<(), Error> {
        T::REGS.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(T::REGS);
        self.set_word_size(W::CONFIG);
        for word in words.iter() {
            let _ = transfer_word(T::REGS, *word)?;
        }
        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        T::REGS.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(T::REGS);
        self.set_word_size(W::CONFIG);
        for word in words.iter_mut() {
            *word = transfer_word(T::REGS, W::default())?;
        }
        Ok(())
    }

    /// Blocking in-place bidirectional transfer.
    ///
    /// This writes the contents of `data` on MOSI, and puts the received data on MISO in `data`, at the same time.
    pub fn blocking_transfer_in_place<W: Word>(&mut self, words: &mut [W]) -> Result<(), Error> {
        T::REGS.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(T::REGS);
        self.set_word_size(W::CONFIG);
        for word in words.iter_mut() {
            *word = transfer_word(T::REGS, *word)?;
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
        T::REGS.cr1().modify(|w| w.set_spe(true));
        flush_rx_fifo(T::REGS);
        self.set_word_size(W::CONFIG);
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or_default();
            let rb = transfer_word(T::REGS, wb)?;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance, Tx, Rx> Drop for Spi<'d, T, Tx, Rx> {
    fn drop(&mut self) {
        self.sck.as_ref().map(|x| x.set_as_disconnected());
        self.mosi.as_ref().map(|x| x.set_as_disconnected());
        self.miso.as_ref().map(|x| x.set_as_disconnected());

        T::disable();
    }
}

#[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
use vals::Br;
#[cfg(any(spi_v3, spi_v4, spi_v5))]
use vals::Mbr as Br;

fn compute_baud_rate(clocks: Hertz, freq: Hertz) -> Br {
    let val = match clocks.0 / freq.0 {
        0 => unreachable!(),
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

fn compute_frequency(clocks: Hertz, br: Br) -> Hertz {
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

    clocks / div
}

trait RegsExt {
    fn tx_ptr<W>(&self) -> *mut W;
    fn rx_ptr<W>(&self) -> *mut W;
}

impl RegsExt for Regs {
    fn tx_ptr<W>(&self) -> *mut W {
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        let dr = self.dr();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let dr = self.txdr();
        dr.as_ptr() as *mut W
    }

    fn rx_ptr<W>(&self) -> *mut W {
        #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
        let dr = self.dr();
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        let dr = self.rxdr();
        dr.as_ptr() as *mut W
    }
}

fn check_error_flags(sr: regs::Sr) -> Result<(), Error> {
    if sr.ovr() {
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

fn spin_until_tx_ready(regs: Regs) -> Result<(), Error> {
    loop {
        let sr = regs.sr().read();

        check_error_flags(sr)?;

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

        check_error_flags(sr)?;

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

fn flush_rx_fifo(regs: Regs) {
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    while regs.sr().read().rxne() {
        let _ = regs.dr().read();
    }
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    while regs.sr().read().rxp() {
        let _ = regs.rxdr().read();
    }
}

fn set_txdmaen(regs: Regs, val: bool) {
    #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
    regs.cr2().modify(|reg| {
        reg.set_txdmaen(val);
    });
    #[cfg(any(spi_v3, spi_v4, spi_v5))]
    regs.cfg1().modify(|reg| {
        reg.set_txdmaen(val);
    });
}

fn set_rxdmaen(regs: Regs, val: bool) {
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
    while !regs.sr().read().txc() {}
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
    spin_until_tx_ready(regs)?;

    unsafe {
        ptr::write_volatile(regs.tx_ptr(), tx_word);

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        regs.cr1().modify(|reg| reg.set_cstart(true));
    }

    spin_until_rx_ready(regs)?;

    let rx_word = unsafe { ptr::read_volatile(regs.rx_ptr()) };
    Ok(rx_word)
}

// Note: It is not possible to impl these traits generically in embedded-hal 0.2 due to a conflict with
// some marker traits. For details, see https://github.com/rust-embedded/embedded-hal/pull/289
macro_rules! impl_blocking {
    ($w:ident) => {
        impl<'d, T: Instance, Tx, Rx> embedded_hal_02::blocking::spi::Write<$w> for Spi<'d, T, Tx, Rx> {
            type Error = Error;

            fn write(&mut self, words: &[$w]) -> Result<(), Self::Error> {
                self.blocking_write(words)
            }
        }

        impl<'d, T: Instance, Tx, Rx> embedded_hal_02::blocking::spi::Transfer<$w> for Spi<'d, T, Tx, Rx> {
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

impl<'d, T: Instance, Tx, Rx> embedded_hal_1::spi::ErrorType for Spi<'d, T, Tx, Rx> {
    type Error = Error;
}

impl<'d, T: Instance, W: Word, Tx, Rx> embedded_hal_1::spi::SpiBus<W> for Spi<'d, T, Tx, Rx> {
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

impl<'d, T: Instance, Tx: TxDma<T>, Rx: RxDma<T>, W: Word> embedded_hal_async::spi::SpiBus<W> for Spi<'d, T, Tx, Rx> {
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

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }

    pub trait Word {
        const CONFIG: word_impl::Config;
    }
}

/// Word sizes usable for SPI.
pub trait Word: word::Word + sealed::Word {}

macro_rules! impl_word {
    ($T:ty, $config:expr) => {
        impl sealed::Word for $T {
            const CONFIG: Config = $config;
        }
        impl Word for $T {}
    };
}

#[cfg(any(spi_v1, spi_f1))]
mod word_impl {
    use super::*;

    pub type Config = vals::Dff;

    impl_word!(u8, vals::Dff::EIGHTBIT);
    impl_word!(u16, vals::Dff::SIXTEENBIT);
}

#[cfg(spi_v2)]
mod word_impl {
    use super::*;

    pub type Config = (vals::Ds, vals::Frxth);

    impl_word!(word::U4, (vals::Ds::FOURBIT, vals::Frxth::QUARTER));
    impl_word!(word::U5, (vals::Ds::FIVEBIT, vals::Frxth::QUARTER));
    impl_word!(word::U6, (vals::Ds::SIXBIT, vals::Frxth::QUARTER));
    impl_word!(word::U7, (vals::Ds::SEVENBIT, vals::Frxth::QUARTER));
    impl_word!(u8, (vals::Ds::EIGHTBIT, vals::Frxth::QUARTER));
    impl_word!(word::U9, (vals::Ds::NINEBIT, vals::Frxth::HALF));
    impl_word!(word::U10, (vals::Ds::TENBIT, vals::Frxth::HALF));
    impl_word!(word::U11, (vals::Ds::ELEVENBIT, vals::Frxth::HALF));
    impl_word!(word::U12, (vals::Ds::TWELVEBIT, vals::Frxth::HALF));
    impl_word!(word::U13, (vals::Ds::THIRTEENBIT, vals::Frxth::HALF));
    impl_word!(word::U14, (vals::Ds::FOURTEENBIT, vals::Frxth::HALF));
    impl_word!(word::U15, (vals::Ds::FIFTEENBIT, vals::Frxth::HALF));
    impl_word!(u16, (vals::Ds::SIXTEENBIT, vals::Frxth::HALF));
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

/// SPI instance trait.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + RccPeripheral {}

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
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

impl<'d, T: Instance, Tx, Rx> SetConfig for Spi<'d, T, Tx, Rx> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config)
    }
}
