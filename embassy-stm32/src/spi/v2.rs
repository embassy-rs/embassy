#![macro_use]

use crate::dma::NoDma;
use crate::gpio::{AnyPin, Pin};
use crate::pac::gpio::vals::{Afr, Moder};
use crate::pac::gpio::Gpio;
use crate::pac::spi;
use crate::spi::{
    ByteOrder, Config, Error, Instance, MisoPin, MosiPin, RxDmaChannel, SckPin, TxDmaChannel,
    WordSize,
};
use crate::time::Hertz;
use core::future::Future;
use core::marker::PhantomData;
use core::ptr;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embassy_traits::spi as traits;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use futures::future::join3;

impl WordSize {
    fn ds(&self) -> spi::vals::Ds {
        match self {
            WordSize::EightBit => spi::vals::Ds::EIGHTBIT,
            WordSize::SixteenBit => spi::vals::Ds::SIXTEENBIT,
        }
    }

    fn frxth(&self) -> spi::vals::Frxth {
        match self {
            WordSize::EightBit => spi::vals::Frxth::QUARTER,
            WordSize::SixteenBit => spi::vals::Frxth::HALF,
        }
    }
}

pub struct Spi<'d, T: Instance, Tx, Rx> {
    sck: Option<AnyPin>,
    mosi: Option<AnyPin>,
    miso: Option<AnyPin>,
    txdma: Tx,
    rxdma: Rx,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub fn new<F>(
        _peri: impl Unborrow<Target = T> + 'd,
        sck: impl Unborrow<Target = impl SckPin<T>>,
        mosi: impl Unborrow<Target = impl MosiPin<T>>,
        miso: impl Unborrow<Target = impl MisoPin<T>>,
        txdma: impl Unborrow<Target = Tx>,
        rxdma: impl Unborrow<Target = Rx>,
        freq: F,
        config: Config,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        unborrow!(sck, mosi, miso, txdma, rxdma);

        let sck_af = sck.af_num();
        let mosi_af = mosi.af_num();
        let miso_af = miso.af_num();
        let sck = sck.degrade_optional();
        let mosi = mosi.degrade_optional();
        let miso = miso.degrade_optional();

        unsafe {
            sck.as_ref()
                .map(|x| Self::configure_pin(x.block(), x.pin() as _, sck_af));
            mosi.as_ref()
                .map(|x| Self::configure_pin(x.block(), x.pin() as _, mosi_af));
            miso.as_ref()
                .map(|x| Self::configure_pin(x.block(), x.pin() as _, miso_af));
        }

        let pclk = T::frequency();
        let freq = freq.into();
        let br = Self::compute_baud_rate(pclk, freq);

        unsafe {
            T::enable();
            T::reset();
            T::regs().cr2().modify(|w| {
                w.set_ssoe(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_cpha(
                    match config.mode.phase == Phase::CaptureOnSecondTransition {
                        true => spi::vals::Cpha::SECONDEDGE,
                        false => spi::vals::Cpha::FIRSTEDGE,
                    },
                );
                w.set_cpol(match config.mode.polarity == Polarity::IdleHigh {
                    true => spi::vals::Cpol::IDLEHIGH,
                    false => spi::vals::Cpol::IDLELOW,
                });

                w.set_mstr(spi::vals::Mstr::MASTER);
                w.set_br(spi::vals::Br(br));
                w.set_lsbfirst(match config.byte_order {
                    ByteOrder::LsbFirst => spi::vals::Lsbfirst::LSBFIRST,
                    ByteOrder::MsbFirst => spi::vals::Lsbfirst::MSBFIRST,
                });
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(spi::vals::Bidimode::UNIDIRECTIONAL);
                w.set_spe(true);
            });
        }

        Self {
            sck,
            mosi,
            miso,
            txdma,
            rxdma,
            phantom: PhantomData,
        }
    }

    unsafe fn configure_pin(block: Gpio, pin: usize, af_num: u8) {
        let (afr, n_af) = if pin < 8 { (0, pin) } else { (1, pin - 8) };
        block.moder().modify(|w| w.set_moder(pin, Moder::ALTERNATE));
        block.afr(afr).modify(|w| w.set_afr(n_af, Afr(af_num)));
    }

    unsafe fn unconfigure_pin(block: Gpio, pin: usize) {
        block.moder().modify(|w| w.set_moder(pin, Moder::ANALOG));
    }

    fn compute_baud_rate(clocks: Hertz, freq: Hertz) -> u8 {
        match clocks.0 / freq.0 {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=5 => 0b001,
            6..=11 => 0b010,
            12..=23 => 0b011,
            24..=39 => 0b100,
            40..=95 => 0b101,
            96..=191 => 0b110,
            _ => 0b111,
        }
    }

    fn set_word_size(word_size: WordSize) {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|w| {
                w.set_frxth(word_size.frxth());
                w.set_ds(word_size.ds());
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }
    }

    #[allow(unused)]
    async fn write_dma_u8(&mut self, write: &[u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
    {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }
        Self::set_word_size(WordSize::EightBit);

        let request = self.txdma.request();
        let dst = T::regs().dr().ptr() as *mut u8;
        let f = self.txdma.write(request, write, dst);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        f.await;
        Ok(())
    }

    #[allow(unused)]
    async fn read_dma_u8(&mut self, read: &mut [u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_rxdmaen(true);
            });
        }
        Self::set_word_size(WordSize::EightBit);

        let clock_byte_count = read.len();

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().dr().ptr() as *mut u8;
        let rx_f = self.rxdma.read(rx_request, rx_src, read);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().dr().ptr() as *mut u8;
        let clock_byte = 0x00;
        let tx_f = self
            .txdma
            .write_x(tx_request, &clock_byte, clock_byte_count, tx_dst);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(false);
                reg.set_rxdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        Ok(())
    }

    #[allow(unused)]
    async fn read_write_dma_u8(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        assert!(read.len() >= write.len());

        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_rxdmaen(true);
            });
        }
        Self::set_word_size(WordSize::EightBit);

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().dr().ptr() as *mut u8;
        let rx_f = self
            .rxdma
            .read(rx_request, rx_src, &mut read[0..write.len()]);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().dr().ptr() as *mut u8;
        let tx_f = self.txdma.write(tx_request, write, tx_dst);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(false);
                reg.set_rxdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        Ok(())
    }

    async fn wait_for_idle() {
        unsafe {
            while T::regs().sr().read().ftlvl() > 0 {
                // spin
            }
            while T::regs().sr().read().frlvl() > 0 {
                // spin
            }
            while T::regs().sr().read().bsy() {
                // spin
            }
        }
    }
}

impl<'d, T: Instance, Tx, Rx> Drop for Spi<'d, T, Tx, Rx> {
    fn drop(&mut self) {
        unsafe {
            self.sck
                .as_ref()
                .map(|x| Self::unconfigure_pin(x.block(), x.pin() as _));
            self.mosi
                .as_ref()
                .map(|x| Self::unconfigure_pin(x.block(), x.pin() as _));
            self.miso
                .as_ref()
                .map(|x| Self::unconfigure_pin(x.block(), x.pin() as _));
        }
    }
}

trait Word {}

impl Word for u8 {}
impl Word for u16 {}

/// Write a single word blocking. Assumes word size have already been set.
fn write_word<W: Word>(regs: &'static crate::pac::spi::Spi, word: W) -> Result<(), Error> {
    loop {
        let sr = unsafe { regs.sr().read() };
        if sr.ovr() {
            return Err(Error::Overrun);
        } else if sr.fre() {
            return Err(Error::Framing);
        } else if sr.modf() {
            return Err(Error::ModeFault);
        } else if sr.crcerr() {
            return Err(Error::Crc);
        } else if sr.txe() {
            unsafe {
                let dr = regs.dr().ptr() as *mut W;
                ptr::write_volatile(dr, word);
            }
            return Ok(());
        }
    }
}

/// Read a single word blocking. Assumes word size have already been set.
fn read_word<W: Word>(regs: &'static crate::pac::spi::Spi) -> Result<W, Error> {
    loop {
        let sr = unsafe { regs.sr().read() };
        if sr.ovr() {
            return Err(Error::Overrun);
        } else if sr.modf() {
            return Err(Error::ModeFault);
        } else if sr.fre() {
            return Err(Error::Framing);
        } else if sr.crcerr() {
            return Err(Error::Crc);
        } else if sr.rxne() {
            unsafe {
                let dr = regs.dr().ptr() as *const W;
                return Ok(ptr::read_volatile(dr));
            }
        }
    }
}

impl<'d, T: Instance, Rx> embedded_hal::blocking::spi::Write<u8> for Spi<'d, T, NoDma, Rx> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Self::set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter() {
            write_word(regs, *word)?;
            let _: u8 = read_word(regs)?;
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u8> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Self::set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            write_word(regs, *word)?;
            *word = read_word(regs)?;
        }

        Ok(words)
    }
}

impl<'d, T: Instance, Rx> embedded_hal::blocking::spi::Write<u16> for Spi<'d, T, NoDma, Rx> {
    type Error = Error;

    fn write(&mut self, words: &[u16]) -> Result<(), Self::Error> {
        Self::set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter() {
            write_word(regs, *word)?;
            let _: u16 = read_word(regs)?;
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u16> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u16]) -> Result<&'w [u16], Self::Error> {
        Self::set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            write_word(regs, *word)?;
            *word = read_word(regs)?;
        }

        Ok(words)
    }
}

impl<'d, T: Instance, Tx, Rx> traits::Spi<u8> for Spi<'d, T, Tx, Rx> {
    type Error = super::Error;
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx> traits::Write<u8> for Spi<'d, T, Tx, Rx> {
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
        self.write_dma_u8(data)
    }
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx: RxDmaChannel<T>> traits::Read<u8>
    for Spi<'d, T, Tx, Rx>
{
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.read_dma_u8(data)
    }
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx: RxDmaChannel<T>> traits::FullDuplex<u8>
    for Spi<'d, T, Tx, Rx>
{
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read_write<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        self.read_write_dma_u8(read, write)
    }
}
