//! Inter-IC Sound (I2S)

use embassy_hal_internal::into_ref;

use crate::dma::ChannelAndRequest;
use crate::gpio::{AFType, AnyPin, SealedPin, Speed};
use crate::mode::Async;
use crate::pac::spi::vals;
use crate::spi::{Config as SpiConfig, *};
use crate::time::Hertz;
use crate::{Peripheral, PeripheralRef};

/// I2S mode
#[derive(Copy, Clone)]
pub enum Mode {
    /// Master mode
    Master,
    /// Slave mode
    Slave,
}

/// I2C standard
#[derive(Copy, Clone)]
pub enum Standard {
    /// Philips
    Philips,
    /// Most significant bit first.
    MsbFirst,
    /// Least significant bit first.
    LsbFirst,
    /// PCM with long sync.
    PcmLongSync,
    /// PCM with short sync.
    PcmShortSync,
}

impl Standard {
    #[cfg(any(spi_v1, spi_v3, spi_f1))]
    const fn i2sstd(&self) -> vals::I2sstd {
        match self {
            Standard::Philips => vals::I2sstd::PHILIPS,
            Standard::MsbFirst => vals::I2sstd::MSB,
            Standard::LsbFirst => vals::I2sstd::LSB,
            Standard::PcmLongSync => vals::I2sstd::PCM,
            Standard::PcmShortSync => vals::I2sstd::PCM,
        }
    }

    #[cfg(any(spi_v1, spi_v3, spi_f1))]
    const fn pcmsync(&self) -> vals::Pcmsync {
        match self {
            Standard::PcmLongSync => vals::Pcmsync::LONG,
            _ => vals::Pcmsync::SHORT,
        }
    }
}

/// I2S data format.
#[derive(Copy, Clone)]
pub enum Format {
    /// 16 bit data length on 16 bit wide channel
    Data16Channel16,
    /// 16 bit data length on 32 bit wide channel
    Data16Channel32,
    /// 24 bit data length on 32 bit wide channel
    Data24Channel32,
    /// 32 bit data length on 32 bit wide channel
    Data32Channel32,
}

impl Format {
    #[cfg(any(spi_v1, spi_v3, spi_f1))]
    const fn datlen(&self) -> vals::Datlen {
        match self {
            Format::Data16Channel16 => vals::Datlen::BITS16,
            Format::Data16Channel32 => vals::Datlen::BITS16,
            Format::Data24Channel32 => vals::Datlen::BITS24,
            Format::Data32Channel32 => vals::Datlen::BITS32,
        }
    }

    #[cfg(any(spi_v1, spi_v3, spi_f1))]
    const fn chlen(&self) -> vals::Chlen {
        match self {
            Format::Data16Channel16 => vals::Chlen::BITS16,
            Format::Data16Channel32 => vals::Chlen::BITS32,
            Format::Data24Channel32 => vals::Chlen::BITS32,
            Format::Data32Channel32 => vals::Chlen::BITS32,
        }
    }
}

/// Clock polarity
#[derive(Copy, Clone)]
pub enum ClockPolarity {
    /// Low on idle.
    IdleLow,
    /// High on idle.
    IdleHigh,
}

impl ClockPolarity {
    #[cfg(any(spi_v1, spi_v3, spi_f1))]
    const fn ckpol(&self) -> vals::Ckpol {
        match self {
            ClockPolarity::IdleHigh => vals::Ckpol::IDLEHIGH,
            ClockPolarity::IdleLow => vals::Ckpol::IDLELOW,
        }
    }
}

/// [`I2S`] configuration.
///
///  - `MS`: `Master` or `Slave`
///  - `TR`: `Transmit` or `Receive`
///  - `STD`: I2S standard, eg `Philips`
///  - `FMT`: Frame Format marker, eg `Data16Channel16`
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Mode
    pub mode: Mode,
    /// Which I2S standard to use.
    pub standard: Standard,
    /// Data format.
    pub format: Format,
    /// Clock polarity.
    pub clock_polarity: ClockPolarity,
    /// True to enable master clock output from this instance.
    pub master_clock: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Master,
            standard: Standard::Philips,
            format: Format::Data16Channel16,
            clock_polarity: ClockPolarity::IdleLow,
            master_clock: true,
        }
    }
}

/// I2S driver.
pub struct I2S<'d> {
    _peri: Spi<'d, Async>,
    txsd: Option<PeripheralRef<'d, AnyPin>>,
    rxsd: Option<PeripheralRef<'d, AnyPin>>,
    ws: Option<PeripheralRef<'d, AnyPin>>,
    ck: Option<PeripheralRef<'d, AnyPin>>,
    mck: Option<PeripheralRef<'d, AnyPin>>,
}

/// I2S function
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum Function {
    /// Transmit audio data
    Transmit,
    /// Receive audio data
    Receive,
    #[cfg(spi_v3)]
    /// Transmit and Receive audio data
    FullDuplex,
}

impl<'d> I2S<'d> {
    /// Create a transmitter driver
    pub fn new_txonly<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        sd: impl Peripheral<P = impl MosiPin<T>> + 'd,
        ws: impl Peripheral<P = impl WsPin<T>> + 'd,
        ck: impl Peripheral<P = impl CkPin<T>> + 'd,
        mck: impl Peripheral<P = impl MckPin<T>> + 'd,
        txdma: impl Peripheral<P = impl TxDma<T>> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(sd);
        Self::new_inner(
            peri,
            new_pin!(sd, AFType::OutputPushPull, Speed::VeryHigh),
            None,
            ws,
            ck,
            mck,
            new_dma!(txdma),
            None,
            freq,
            config,
            Function::Transmit,
        )
    }

    /// Create a receiver driver
    pub fn new_rxonly<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        sd: impl Peripheral<P = impl MisoPin<T>> + 'd,
        ws: impl Peripheral<P = impl WsPin<T>> + 'd,
        ck: impl Peripheral<P = impl CkPin<T>> + 'd,
        mck: impl Peripheral<P = impl MckPin<T>> + 'd,
        #[cfg(not(spi_v3))] txdma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rxdma: impl Peripheral<P = impl RxDma<T>> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(sd);
        Self::new_inner(
            peri,
            None,
            new_pin!(sd, AFType::OutputPushPull, Speed::VeryHigh),
            ws,
            ck,
            mck,
            #[cfg(not(spi_v3))]
            new_dma!(txdma),
            #[cfg(spi_v3)]
            None,
            new_dma!(rxdma),
            freq,
            config,
            #[cfg(not(spi_v3))]
            Function::Transmit,
            #[cfg(spi_v3)]
            Function::Receive,
        )
    }

    #[cfg(spi_v3)]
    /// Create a full duplex transmitter driver
    pub fn new_full_duplex<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        txsd: impl Peripheral<P = impl MosiPin<T>> + 'd,
        rxsd: impl Peripheral<P = impl MisoPin<T>> + 'd,
        ws: impl Peripheral<P = impl WsPin<T>> + 'd,
        ck: impl Peripheral<P = impl CkPin<T>> + 'd,
        mck: impl Peripheral<P = impl MckPin<T>> + 'd,
        txdma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rxdma: impl Peripheral<P = impl RxDma<T>> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(txsd, rxsd);
        Self::new_inner(
            peri,
            new_pin!(txsd, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(rxsd, AFType::OutputPushPull, Speed::VeryHigh),
            ws,
            ck,
            mck,
            new_dma!(txdma),
            new_dma!(rxdma),
            freq,
            config,
            Function::FullDuplex,
        )
    }

    /// Write audio data.
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        self._peri.read(data).await
    }

    /// Write audio data.
    pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        self._peri.write(data).await
    }

    /// Transfer audio data.
    pub async fn transfer<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        self._peri.transfer(read, write).await
    }

    /// Transfer audio data in place.
    pub async fn transfer_in_place<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error> {
        self._peri.transfer_in_place(data).await
    }

    fn new_inner<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        txsd: Option<PeripheralRef<'d, AnyPin>>,
        rxsd: Option<PeripheralRef<'d, AnyPin>>,
        ws: impl Peripheral<P = impl WsPin<T>> + 'd,
        ck: impl Peripheral<P = impl CkPin<T>> + 'd,
        mck: impl Peripheral<P = impl MckPin<T>> + 'd,
        txdma: Option<ChannelAndRequest<'d>>,
        rxdma: Option<ChannelAndRequest<'d>>,
        freq: Hertz,
        config: Config,
        function: Function,
    ) -> Self {
        into_ref!(ws, ck, mck);

        ws.set_as_af(ws.af_num(), AFType::OutputPushPull);
        ws.set_speed(Speed::VeryHigh);

        ck.set_as_af(ck.af_num(), AFType::OutputPushPull);
        ck.set_speed(Speed::VeryHigh);

        mck.set_as_af(mck.af_num(), AFType::OutputPushPull);
        mck.set_speed(Speed::VeryHigh);

        let mut spi_cfg = SpiConfig::default();
        spi_cfg.frequency = freq;

        let spi = Spi::new_internal(peri, txdma, rxdma, spi_cfg);

        let regs = T::info().regs;

        // TODO move i2s to the new mux infra.
        //#[cfg(all(rcc_f4, not(stm32f410)))]
        //let pclk = unsafe { get_freqs() }.plli2s1_q.unwrap();
        //#[cfg(stm32f410)]
        let pclk = T::frequency();

        let (odd, div) = compute_baud_rate(pclk, freq, config.master_clock, config.format);

        #[cfg(any(spi_v1, spi_v3, spi_f1))]
        {
            #[cfg(spi_v3)]
            {
                regs.cr1().modify(|w| w.set_spe(false));

                reset_incompatible_bitfields::<T>();
            }

            use stm32_metapac::spi::vals::{I2scfg, Odd};

            // 1. Select the I2SDIV[7:0] bits in the SPI_I2SPR/SPI_I2SCFGR register to define the serial clock baud
            // rate to reach the proper audio sample frequency. The ODD bit in the
            // SPI_I2SPR/SPI_I2SCFGR register also has to be defined.

            // 2. Select the CKPOL bit to define the steady level for the communication clock. Set the
            // MCKOE bit in the SPI_I2SPR/SPI_I2SCFGR register if the master clock MCK needs to be provided to
            // the external DAC/ADC audio component (the I2SDIV and ODD values should be
            // computed depending on the state of the MCK output, for more details refer to
            // Section 28.4.4: Clock generator).

            // 3. Set the I2SMOD bit in SPI_I2SCFGR to activate the I2S functionalities and choose the
            // I2S standard through the I2SSTD[1:0] and PCMSYNC bits, the data length through the
            // DATLEN[1:0] bits and the number of bits per channel by configuring the CHLEN bit.
            // Select also the I2S master mode and direction (Transmitter or Receiver) through the
            // I2SCFG[1:0] bits in the SPI_I2SCFGR register.

            // 4. If needed, select all the potential interruption sources and the DMA capabilities by
            // writing the SPI_CR2 register.

            // 5. The I2SE bit in SPI_I2SCFGR register must be set.

            let clk_reg = {
                #[cfg(any(spi_v1, spi_f1))]
                {
                    regs.i2spr()
                }
                #[cfg(spi_v3)]
                {
                    regs.i2scfgr()
                }
            };

            clk_reg.modify(|w| {
                w.set_i2sdiv(div);
                w.set_odd(match odd {
                    true => Odd::ODD,
                    false => Odd::EVEN,
                });

                w.set_mckoe(config.master_clock);
            });

            regs.i2scfgr().modify(|w| {
                w.set_ckpol(config.clock_polarity.ckpol());

                w.set_i2smod(true);

                w.set_i2sstd(config.standard.i2sstd());
                w.set_pcmsync(config.standard.pcmsync());

                w.set_datlen(config.format.datlen());
                w.set_chlen(config.format.chlen());

                w.set_i2scfg(match (config.mode, function) {
                    (Mode::Master, Function::Transmit) => I2scfg::MASTERTX,
                    (Mode::Master, Function::Receive) => I2scfg::MASTERRX,
                    #[cfg(spi_v3)]
                    (Mode::Master, Function::FullDuplex) => I2scfg::MASTERFULLDUPLEX,
                    (Mode::Slave, Function::Transmit) => I2scfg::SLAVETX,
                    (Mode::Slave, Function::Receive) => I2scfg::SLAVERX,
                    #[cfg(spi_v3)]
                    (Mode::Slave, Function::FullDuplex) => I2scfg::SLAVEFULLDUPLEX,
                });

                #[cfg(any(spi_v1, spi_f1))]
                w.set_i2se(true);
            });

            #[cfg(spi_v3)]
            regs.cr1().modify(|w| w.set_spe(true));
        }

        Self {
            _peri: spi,
            txsd: txsd.map(|w| w.map_into()),
            rxsd: rxsd.map(|w| w.map_into()),
            ws: Some(ws.map_into()),
            ck: Some(ck.map_into()),
            mck: Some(mck.map_into()),
        }
    }
}

impl<'d> Drop for I2S<'d> {
    fn drop(&mut self) {
        self.txsd.as_ref().map(|x| x.set_as_disconnected());
        self.rxsd.as_ref().map(|x| x.set_as_disconnected());
        self.ws.as_ref().map(|x| x.set_as_disconnected());
        self.ck.as_ref().map(|x| x.set_as_disconnected());
        self.mck.as_ref().map(|x| x.set_as_disconnected());
    }
}

// Note, calculation details:
// Fs = i2s_clock / [256 * ((2 * div) + odd)] when master clock is enabled
// Fs = i2s_clock / [(channel_length * 2) * ((2 * div) + odd)]` when master clock is disabled
// channel_length is 16 or 32
//
// can be rewritten as
// Fs = i2s_clock / (coef * division)
// where coef is a constant equal to 256, 64 or 32 depending channel length and master clock
// and where division = (2 * div) + odd
//
// Equation can be rewritten as
// division = i2s_clock/ (coef * Fs)
//
// note: division = (2 * div) + odd = (div << 1) + odd
// in other word, from bits point of view, division[8:1] = div[7:0] and division[0] = odd
fn compute_baud_rate(i2s_clock: Hertz, request_freq: Hertz, mclk: bool, data_format: Format) -> (bool, u8) {
    let coef = if mclk {
        256
    } else if let Format::Data16Channel16 = data_format {
        32
    } else {
        64
    };

    let (n, d) = (i2s_clock.0, coef * request_freq.0);
    let division = (n + (d >> 1)) / d;

    if division < 4 {
        (false, 2)
    } else if division > 511 {
        (true, 255)
    } else {
        ((division & 1) == 1, (division >> 1) as u8)
    }
}

#[cfg(spi_v3)]
// The STM32H7 reference manual specifies that any incompatible bitfields should be reset
// to their reset values while operating in I2S mode.
fn reset_incompatible_bitfields<T: Instance>() {
    let regs = T::info().regs;
    regs.cr1().modify(|w| {
        let iolock = w.iolock();
        let csusp = w.csusp();
        let spe = w.cstart();
        let cstart = w.cstart();
        w.0 = 0;
        w.set_iolock(iolock);
        w.set_csusp(csusp);
        w.set_spe(spe);
        w.set_cstart(cstart);
    });

    regs.cr2().write(|w| w.0 = 0);

    regs.cfg1().modify(|w| {
        let txdmaen = w.txdmaen();
        let rxdmaen = w.rxdmaen();
        let fthlv = w.fthlv();
        w.0 = 0;
        w.set_txdmaen(txdmaen);
        w.set_rxdmaen(rxdmaen);
        w.set_fthlv(fthlv);
    });

    regs.cfg2().modify(|w| {
        let afcntr = w.afcntr();
        let lsbfirst = w.lsbfirst();
        let ioswp = w.ioswp();
        w.0 = 0;
        w.set_afcntr(afcntr);
        w.set_lsbfirst(lsbfirst);
        w.set_ioswp(ioswp);
    });

    regs.ier().modify(|w| {
        let tifreie = w.tifreie();
        let ovrie = w.ovrie();
        let udrie = w.udrie();
        let txpie = w.txpie();
        let rxpie = w.rxpie();

        w.0 = 0;

        w.set_tifreie(tifreie);
        w.set_ovrie(ovrie);
        w.set_udrie(udrie);
        w.set_txpie(txpie);
        w.set_rxpie(rxpie);
    });

    regs.ifcr().write(|w| {
        w.set_suspc(true);
        w.set_tifrec(true);
        w.set_ovrc(true);
        w.set_udrc(true);
    });

    regs.crcpoly().write(|w| w.0 = 0x107);
    regs.txcrc().write(|w| w.0 = 0);
    regs.rxcrc().write(|w| w.0 = 0);
    regs.udrdr().write(|w| w.0 = 0);
}
