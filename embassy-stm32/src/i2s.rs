//! Inter-IC Sound (I2S)

use embassy_futures::join::join;
use stm32_metapac::spi::vals;

use crate::dma::{ringbuffer, ChannelAndRequest, ReadableRingBuffer, TransferOptions, WritableRingBuffer};
use crate::gpio::{AfType, AnyPin, OutputType, SealedPin, Speed};
use crate::mode::Async;
use crate::spi::{Config as SpiConfig, RegsExt as _, *};
use crate::time::Hertz;
use crate::Peri;

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

/// SAI error
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// `write` called on a SAI in receive mode.
    NotATransmitter,
    /// `read` called on a SAI in transmit mode.
    NotAReceiver,
    /// Overrun
    Overrun,
}

impl From<ringbuffer::Error> for Error {
    fn from(#[allow(unused)] err: ringbuffer::Error) -> Self {
        #[cfg(feature = "defmt")]
        {
            if err == ringbuffer::Error::DmaUnsynced {
                defmt::error!("Ringbuffer broken invariants detected!");
            }
        }
        Self::Overrun
    }
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
            ClockPolarity::IdleHigh => vals::Ckpol::IDLE_HIGH,
            ClockPolarity::IdleLow => vals::Ckpol::IDLE_LOW,
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

/// I2S driver writer. Useful for moving write functionality across tasks.
pub struct Writer<'s, 'd, W: Word>(&'s mut WritableRingBuffer<'d, W>);

impl<'s, 'd, W: Word> Writer<'s, 'd, W> {
    /// Write data to the I2S ringbuffer.
    /// This appends the data to the buffer and returns immediately. The data will be transmitted in the background.
    /// If thfre’s no space in the buffer, this waits until there is.
    pub async fn write(&mut self, data: &[W]) -> Result<(), Error> {
        self.0.write_exact(data).await?;
        Ok(())
    }

    /// Reset the ring buffer to its initial state.
    /// Can be used to recover from overrun.
    /// The ringbuffer will always auto-reset on Overrun in any case.
    pub fn reset(&mut self) {
        self.0.clear();
    }
}

/// I2S driver reader. Useful for moving read functionality across tasks.
pub struct Reader<'s, 'd, W: Word>(&'s mut ReadableRingBuffer<'d, W>);

impl<'s, 'd, W: Word> Reader<'s, 'd, W> {
    /// Read data from the I2S ringbuffer.
    /// SAI is always receiving data in the background. This function pops already-received data from the buffer.
    /// If there’s less than data.len() data in the buffer, this waits until there is.
    pub async fn read(&mut self, data: &mut [W]) -> Result<(), Error> {
        self.0.read_exact(data).await?;
        Ok(())
    }

    /// Reset the ring buffer to its initial state.
    /// Can be used to prevent overrun.
    /// The ringbuffer will always auto-reset on Overrun in any case.
    pub fn reset(&mut self) {
        self.0.clear();
    }
}

/// I2S driver.
pub struct I2S<'d, W: Word> {
    #[allow(dead_code)]
    mode: Mode,
    spi: Spi<'d, Async>,
    txsd: Option<Peri<'d, AnyPin>>,
    rxsd: Option<Peri<'d, AnyPin>>,
    ws: Option<Peri<'d, AnyPin>>,
    ck: Option<Peri<'d, AnyPin>>,
    mck: Option<Peri<'d, AnyPin>>,
    tx_ring_buffer: Option<WritableRingBuffer<'d, W>>,
    rx_ring_buffer: Option<ReadableRingBuffer<'d, W>>,
}

impl<'d, W: Word> I2S<'d, W> {
    /// Create a transmitter driver.
    pub fn new_txonly<T: Instance>(
        peri: Peri<'d, T>,
        sd: Peri<'d, impl MosiPin<T>>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        mck: Peri<'d, impl MckPin<T>>,
        txdma: Peri<'d, impl TxDma<T>>,
        txdma_buf: &'d mut [W],
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sd, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            ws,
            ck,
            new_pin!(mck, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_dma!(txdma).map(|d| (d, txdma_buf)),
            None,
            freq,
            config,
            Function::Transmit,
        )
    }

    /// Create a transmitter driver without a master clock pin.
    pub fn new_txonly_nomck<T: Instance>(
        peri: Peri<'d, T>,
        sd: Peri<'d, impl MosiPin<T>>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        txdma: Peri<'d, impl TxDma<T>>,
        txdma_buf: &'d mut [W],
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(sd, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            ws,
            ck,
            None,
            new_dma!(txdma).map(|d| (d, txdma_buf)),
            None,
            freq,
            config,
            Function::Transmit,
        )
    }

    /// Create a receiver driver.
    pub fn new_rxonly<T: Instance>(
        peri: Peri<'d, T>,
        sd: Peri<'d, impl MisoPin<T>>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        mck: Peri<'d, impl MckPin<T>>,
        rxdma: Peri<'d, impl RxDma<T>>,
        rxdma_buf: &'d mut [W],
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            None,
            new_pin!(sd, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ws,
            ck,
            new_pin!(mck, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            new_dma!(rxdma).map(|d| (d, rxdma_buf)),
            freq,
            config,
            Function::Receive,
        )
    }

    #[cfg(spi_v3)]
    /// Create a full duplex driver.
    pub fn new_full_duplex<T: Instance>(
        peri: Peri<'d, T>,
        txsd: Peri<'d, impl MosiPin<T>>,
        rxsd: Peri<'d, impl MisoPin<T>>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        mck: Peri<'d, impl MckPin<T>>,
        txdma: Peri<'d, impl TxDma<T>>,
        txdma_buf: &'d mut [W],
        rxdma: Peri<'d, impl RxDma<T>>,
        rxdma_buf: &'d mut [W],
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(txsd, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(rxsd, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ws,
            ck,
            new_pin!(mck, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_dma!(txdma).map(|d| (d, txdma_buf)),
            new_dma!(rxdma).map(|d| (d, rxdma_buf)),
            freq,
            config,
            Function::FullDuplex,
        )
    }

    /// Start I2S driver.
    pub fn start(&mut self) {
        self.spi.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });
        self.spi.set_word_size(W::CONFIG);
        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            tx_ring_buffer.start();

            set_txdmaen(self.spi.info.regs, true);
        }
        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            rx_ring_buffer.start();
            // SPIv3 clears rxfifo on SPE=0
            #[cfg(not(any(spi_v3, spi_v4, spi_v5)))]
            flush_rx_fifo(self.spi.info.regs);

            set_rxdmaen(self.spi.info.regs, true);
        }
        self.spi.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        self.spi.info.regs.cr1().modify(|w| {
            w.set_cstart(true);
        });
    }

    /// Reset the ring buffer to its initial state.
    /// Can be used to recover from overrun.
    pub fn clear(&mut self) {
        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            rx_ring_buffer.clear();
        }
        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            tx_ring_buffer.clear();
        }
    }

    /// Stop I2S driver.
    pub async fn stop(&mut self) {
        let regs = self.spi.info.regs;

        let tx_f = async {
            if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
                tx_ring_buffer.stop().await;

                set_txdmaen(regs, false);
            }
        };

        let rx_f = async {
            if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
                rx_ring_buffer.stop().await;

                set_rxdmaen(regs, false);
            }
        };

        join(rx_f, tx_f).await;

        #[cfg(any(spi_v3, spi_v4, spi_v5))]
        {
            if let Mode::Master = self.mode {
                regs.cr1().modify(|w| {
                    w.set_csusp(true);
                });

                while regs.cr1().read().cstart() {}
            }
        }

        regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.clear();
    }

    /// Split the driver into a Reader/Writer pair.
    /// Useful for splitting the reader/writer functionality across tasks or
    /// for calling the read/write methods in parallel.
    pub fn split<'s>(&'s mut self) -> Result<(Reader<'s, 'd, W>, Writer<'s, 'd, W>), Error> {
        match (&mut self.rx_ring_buffer, &mut self.tx_ring_buffer) {
            (None, _) => Err(Error::NotAReceiver),
            (_, None) => Err(Error::NotATransmitter),
            (Some(rx_ring), Some(tx_ring)) => Ok((Reader(rx_ring), Writer(tx_ring))),
        }
    }

    /// Read data from the I2S ringbuffer.
    /// SAI is always receiving data in the background. This function pops already-received data from the buffer.
    /// If there’s less than data.len() data in the buffer, this waits until there is.
    pub async fn read(&mut self, data: &mut [W]) -> Result<(), Error> {
        match &mut self.rx_ring_buffer {
            Some(ring) => Reader(ring).read(data).await,
            _ => Err(Error::NotAReceiver),
        }
    }

    /// Write data to the I2S ringbuffer.
    /// This appends the data to the buffer and returns immediately. The data will be transmitted in the background.
    /// If thfre’s no space in the buffer, this waits until there is.
    pub async fn write(&mut self, data: &[W]) -> Result<(), Error> {
        match &mut self.tx_ring_buffer {
            Some(ring) => Writer(ring).write(data).await,
            _ => Err(Error::NotATransmitter),
        }
    }

    /// Write data directly to the raw I2S ringbuffer.
    /// This can be used to fill the buffer before starting the DMA transfer.
    pub async fn write_immediate(&mut self, data: &[W]) -> Result<(usize, usize), Error> {
        match &mut self.tx_ring_buffer {
            Some(ring) => Ok(ring.write_immediate(data)?),
            _ => return Err(Error::NotATransmitter),
        }
    }

    fn new_inner<T: Instance>(
        peri: Peri<'d, T>,
        txsd: Option<Peri<'d, AnyPin>>,
        rxsd: Option<Peri<'d, AnyPin>>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        mck: Option<Peri<'d, AnyPin>>,
        txdma: Option<(ChannelAndRequest<'d>, &'d mut [W])>,
        rxdma: Option<(ChannelAndRequest<'d>, &'d mut [W])>,
        freq: Hertz,
        config: Config,
        function: Function,
    ) -> Self {
        ws.set_as_af(ws.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ck.set_as_af(ck.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let spi = Spi::new_internal(peri, None, None, {
            let mut config = SpiConfig::default();
            config.frequency = freq;
            config
        });

        let regs = T::info().regs;

        #[cfg(all(rcc_f4, not(stm32f410)))]
        let pclk = unsafe { crate::rcc::get_freqs() }.plli2s1_r.to_hertz().unwrap();
        #[cfg(not(all(rcc_f4, not(stm32f410))))]
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
                    (Mode::Master, Function::Transmit) => I2scfg::MASTER_TX,
                    (Mode::Master, Function::Receive) => I2scfg::MASTER_RX,
                    #[cfg(spi_v3)]
                    (Mode::Master, Function::FullDuplex) => I2scfg::MASTER_FULL_DUPLEX,
                    (Mode::Slave, Function::Transmit) => I2scfg::SLAVE_TX,
                    (Mode::Slave, Function::Receive) => I2scfg::SLAVE_RX,
                    #[cfg(spi_v3)]
                    (Mode::Slave, Function::FullDuplex) => I2scfg::SLAVE_FULL_DUPLEX,
                });

                #[cfg(any(spi_v1, spi_f1))]
                w.set_i2se(true);
            });

            let mut opts = TransferOptions::default();
            opts.half_transfer_ir = true;

            Self {
                mode: config.mode,
                spi,
                txsd: txsd.map(|w| w.into()),
                rxsd: rxsd.map(|w| w.into()),
                ws: Some(ws.into()),
                ck: Some(ck.into()),
                mck: mck.map(|w| w.into()),
                tx_ring_buffer: txdma.map(|(ch, buf)| unsafe {
                    WritableRingBuffer::new(ch.channel, ch.request, regs.tx_ptr(), buf, opts)
                }),
                rx_ring_buffer: rxdma.map(|(ch, buf)| unsafe {
                    ReadableRingBuffer::new(ch.channel, ch.request, regs.rx_ptr(), buf, opts)
                }),
            }
        }
    }
}

impl<'d, W: Word> Drop for I2S<'d, W> {
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
