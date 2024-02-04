//! Inter-IC Sound (I2S)
use embassy_hal_internal::into_ref;

use crate::gpio::sealed::{AFType, Pin as _};
use crate::gpio::AnyPin;
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

/// I2S function
#[derive(Copy, Clone)]
pub enum Function {
    /// Transmit audio data
    Transmit,
    /// Receive audio data
    Receive,
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
    #[cfg(any(spi_v1, spi_f1))]
    const fn i2sstd(&self) -> vals::I2sstd {
        match self {
            Standard::Philips => vals::I2sstd::PHILIPS,
            Standard::MsbFirst => vals::I2sstd::MSB,
            Standard::LsbFirst => vals::I2sstd::LSB,
            Standard::PcmLongSync => vals::I2sstd::PCM,
            Standard::PcmShortSync => vals::I2sstd::PCM,
        }
    }

    #[cfg(any(spi_v1, spi_f1))]
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
    #[cfg(any(spi_v1, spi_f1))]
    const fn datlen(&self) -> vals::Datlen {
        match self {
            Format::Data16Channel16 => vals::Datlen::SIXTEENBIT,
            Format::Data16Channel32 => vals::Datlen::SIXTEENBIT,
            Format::Data24Channel32 => vals::Datlen::TWENTYFOURBIT,
            Format::Data32Channel32 => vals::Datlen::THIRTYTWOBIT,
        }
    }

    #[cfg(any(spi_v1, spi_f1))]
    const fn chlen(&self) -> vals::Chlen {
        match self {
            Format::Data16Channel16 => vals::Chlen::SIXTEENBIT,
            Format::Data16Channel32 => vals::Chlen::THIRTYTWOBIT,
            Format::Data24Channel32 => vals::Chlen::THIRTYTWOBIT,
            Format::Data32Channel32 => vals::Chlen::THIRTYTWOBIT,
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
    #[cfg(any(spi_v1, spi_f1))]
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
    /// Function (transmit, receive)
    pub function: Function,
    /// Which I2S standard to use.
    pub standard: Standard,
    /// Data format.
    pub format: Format,
    /// Clock polarity.
    pub clock_polarity: ClockPolarity,
    /// True to eanble master clock output from this instance.
    pub master_clock: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Master,
            function: Function::Transmit,
            standard: Standard::Philips,
            format: Format::Data16Channel16,
            clock_polarity: ClockPolarity::IdleLow,
            master_clock: true,
        }
    }
}

/// I2S driver.
pub struct I2S<'d, T: Instance, Tx, Rx> {
    _peri: Spi<'d, T, Tx, Rx>,
    sd: Option<PeripheralRef<'d, AnyPin>>,
    ws: Option<PeripheralRef<'d, AnyPin>>,
    ck: Option<PeripheralRef<'d, AnyPin>>,
    mck: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance, Tx, Rx> I2S<'d, T, Tx, Rx> {
    /// Note: Full-Duplex modes are not supported at this time
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        sd: impl Peripheral<P = impl MosiPin<T>> + 'd,
        ws: impl Peripheral<P = impl WsPin<T>> + 'd,
        ck: impl Peripheral<P = impl CkPin<T>> + 'd,
        mck: impl Peripheral<P = impl MckPin<T>> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(sd, ws, ck, mck);

        sd.set_as_af(sd.af_num(), AFType::OutputPushPull);
        sd.set_speed(crate::gpio::Speed::VeryHigh);

        ws.set_as_af(ws.af_num(), AFType::OutputPushPull);
        ws.set_speed(crate::gpio::Speed::VeryHigh);

        ck.set_as_af(ck.af_num(), AFType::OutputPushPull);
        ck.set_speed(crate::gpio::Speed::VeryHigh);

        mck.set_as_af(mck.af_num(), AFType::OutputPushPull);
        mck.set_speed(crate::gpio::Speed::VeryHigh);

        let mut spi_cfg = SpiConfig::default();
        spi_cfg.frequency = freq;
        let spi = Spi::new_internal(peri, txdma, rxdma, spi_cfg);

        // TODO move i2s to the new mux infra.
        //#[cfg(all(rcc_f4, not(stm32f410)))]
        //let pclk = unsafe { get_freqs() }.plli2s1_q.unwrap();
        //#[cfg(stm32f410)]
        let pclk = T::frequency();

        let (odd, div) = compute_baud_rate(pclk, freq, config.master_clock, config.format);

        #[cfg(any(spi_v1, spi_f1))]
        {
            use stm32_metapac::spi::vals::{I2scfg, Odd};

            // 1. Select the I2SDIV[7:0] bits in the SPI_I2SPR register to define the serial clock baud
            // rate to reach the proper audio sample frequency. The ODD bit in the SPI_I2SPR
            // register also has to be defined.

            T::REGS.i2spr().modify(|w| {
                w.set_i2sdiv(div);
                w.set_odd(match odd {
                    true => Odd::ODD,
                    false => Odd::EVEN,
                });

                w.set_mckoe(config.master_clock);
            });

            // 2. Select the CKPOL bit to define the steady level for the communication clock. Set the
            // MCKOE bit in the SPI_I2SPR register if the master clock MCK needs to be provided to
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

            T::REGS.i2scfgr().modify(|w| {
                w.set_ckpol(config.clock_polarity.ckpol());

                w.set_i2smod(true);
                w.set_i2sstd(config.standard.i2sstd());
                w.set_pcmsync(config.standard.pcmsync());

                w.set_datlen(config.format.datlen());
                w.set_chlen(config.format.chlen());

                w.set_i2scfg(match (config.mode, config.function) {
                    (Mode::Master, Function::Transmit) => I2scfg::MASTERTX,
                    (Mode::Master, Function::Receive) => I2scfg::MASTERRX,
                    (Mode::Slave, Function::Transmit) => I2scfg::SLAVETX,
                    (Mode::Slave, Function::Receive) => I2scfg::SLAVERX,
                });

                w.set_i2se(true)
            });
        }

        Self {
            _peri: spi,
            sd: Some(sd.map_into()),
            ws: Some(ws.map_into()),
            ck: Some(ck.map_into()),
            mck: Some(mck.map_into()),
        }
    }

    /// Write audio data.
    pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
    {
        self._peri.write(data).await
    }

    /// Read audio data.
    pub async fn read<W: Word>(&mut self, data: &mut [W]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        self._peri.read(data).await
    }
}

impl<'d, T: Instance, Tx, Rx> Drop for I2S<'d, T, Tx, Rx> {
    fn drop(&mut self) {
        self.sd.as_ref().map(|x| x.set_as_disconnected());
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
