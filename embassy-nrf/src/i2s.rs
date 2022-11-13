#![macro_use]

//! I2S

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::interrupt::{InterruptExt, Priority};
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

//use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::Interrupt;
use crate::pac::i2s::{RegisterBlock, CONFIG, PSEL};
use crate::Peripheral;

// TODO: Define those in lib.rs somewhere else
//
// I2S EasyDMA MAXCNT bit length = 14
const MAX_DMA_MAXCNT: u32 = 1 << 14;

// Limits for Easy DMA - it can only read from data ram
pub const SRAM_LOWER: usize = 0x2000_0000;
pub const SRAM_UPPER: usize = 0x3000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    BufferTooLong,
    BufferZeroLength,
    DMABufferNotInDataMemory,
    BufferMisaligned,
    // TODO: add other error variants.
}

pub const MODE_MASTER_8000: Mode = Mode::Master {
    freq: MckFreq::_32MDiv125,
    ratio: Ratio::_32x,
}; // error = 0
pub const MODE_MASTER_11025: Mode = Mode::Master {
    freq: MckFreq::_32MDiv15,
    ratio: Ratio::_192x,
}; // error = 86
pub const MODE_MASTER_16000: Mode = Mode::Master {
    freq: MckFreq::_32MDiv21,
    ratio: Ratio::_96x,
}; // error = 127
pub const MODE_MASTER_22050: Mode = Mode::Master {
    freq: MckFreq::_32MDiv15,
    ratio: Ratio::_96x,
}; // error = 172
pub const MODE_MASTER_32000: Mode = Mode::Master {
    freq: MckFreq::_32MDiv21,
    ratio: Ratio::_48x,
}; // error = 254
pub const MODE_MASTER_44100: Mode = Mode::Master {
    freq: MckFreq::_32MDiv15,
    ratio: Ratio::_48x,
}; // error = 344
pub const MODE_MASTER_48000: Mode = Mode::Master {
    freq: MckFreq::_32MDiv21,
    ratio: Ratio::_32x,
}; // error = 381

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub mode: Mode,
    pub swidth: SampleWidth,
    pub align: Align,
    pub format: Format,
    pub channels: Channels,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_MASTER_32000,
            swidth: SampleWidth::_16bit,
            align: Align::Left,
            format: Format::I2S,
            channels: Channels::Stereo,
        }
    }
}

/// I2S Mode
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Master { freq: MckFreq, ratio: Ratio },
    Slave,
}

impl Mode {
    pub fn sample_rate(&self) -> Option<u32> {
        match self {
            Mode::Master { freq, ratio } => Some(freq.to_frequency() / ratio.to_divisor()),
            Mode::Slave => None,
        }
    }
}

/// Master clock generator frequency.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MckFreq {
    _32MDiv8,
    _32MDiv10,
    _32MDiv11,
    _32MDiv15,
    _32MDiv16,
    _32MDiv21,
    _32MDiv23,
    _32MDiv30,
    _32MDiv31,
    _32MDiv32,
    _32MDiv42,
    _32MDiv63,
    _32MDiv125,
}

impl MckFreq {
    const REGISTER_VALUES: &[u32] = &[
        0x20000000, 0x18000000, 0x16000000, 0x11000000, 0x10000000, 0x0C000000, 0x0B000000, 0x08800000, 0x08400000,
        0x08000000, 0x06000000, 0x04100000, 0x020C0000,
    ];

    const FREQUENCIES: &[u32] = &[
        4000000, 3200000, 2909090, 2133333, 2000000, 1523809, 1391304, 1066666, 1032258, 1000000, 761904, 507936,
        256000,
    ];

    pub fn to_register_value(&self) -> u32 {
        Self::REGISTER_VALUES[usize::from(*self)]
    }

    pub fn to_frequency(&self) -> u32 {
        Self::FREQUENCIES[usize::from(*self)]
    }
}

impl From<MckFreq> for usize {
    fn from(variant: MckFreq) -> Self {
        variant as _
    }
}

/// MCK / LRCK ratio.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Ratio {
    _32x,
    _48x,
    _64x,
    _96x,
    _128x,
    _192x,
    _256x,
    _384x,
    _512x,
}

impl Ratio {
    const RATIOS: &[u32] = &[32, 48, 64, 96, 128, 192, 256, 384, 512];

    pub fn to_divisor(&self) -> u32 {
        Self::RATIOS[u8::from(*self) as usize]
    }
}

impl From<Ratio> for u8 {
    fn from(variant: Ratio) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SampleWidth {
    _8bit,
    _16bit,
    _24bit,
}

impl From<SampleWidth> for u8 {
    fn from(variant: SampleWidth) -> Self {
        variant as _
    }
}

/// Alignment of sample within a frame.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Align {
    Left,
    Right,
}

impl From<Align> for bool {
    fn from(variant: Align) -> Self {
        match variant {
            Align::Left => false,
            Align::Right => true,
        }
    }
}

/// Frame format.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Format {
    I2S,
    Aligned,
}

impl From<Format> for bool {
    fn from(variant: Format) -> Self {
        match variant {
            Format::I2S => false,
            Format::Aligned => true,
        }
    }
}

/// Enable channels.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Channels {
    Stereo,
    Left,
    Right,
}

impl From<Channels> for u8 {
    fn from(variant: Channels) -> Self {
        variant as _
    }
}

/// Interface to the UARTE peripheral using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct I2S<'d, T: Instance> {
    output: I2sOutput<'d, T>,
    input: I2sInput<'d, T>,
}

/// Transmitter interface to the UARTE peripheral obtained
/// via [Uarte]::split.
pub struct I2sOutput<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

/// Receiver interface to the UARTE peripheral obtained
/// via [Uarte]::split.
pub struct I2sInput<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> I2S<'d, T> {
    /// Create a new I2S
    pub fn new(
        i2s: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        mck: impl Peripheral<P = impl GpioPin> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        lrck: impl Peripheral<P = impl GpioPin> + 'd,
        sdin: impl Peripheral<P = impl GpioPin> + 'd,
        sdout: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(mck, sck, lrck, sdin, sdout);
        Self::new_inner(
            i2s,
            irq,
            mck.map_into(),
            sck.map_into(),
            lrck.map_into(),
            sdin.map_into(),
            sdout.map_into(),
            config,
        )
    }

    fn new_inner(
        i2s: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        mck: PeripheralRef<'d, AnyPin>,
        sck: PeripheralRef<'d, AnyPin>,
        lrck: PeripheralRef<'d, AnyPin>,
        sdin: PeripheralRef<'d, AnyPin>,
        sdout: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(i2s, irq, mck, sck, lrck, sdin, sdout);

        let r = T::regs();
        Self::apply_config(&r.config, &config);
        Self::select_pins(&r.psel, mck, sck, lrck, sdin, sdout);
        Self::setup_interrupt(irq, r);

        r.enable.write(|w| w.enable().enabled());

        Self {
            output: I2sOutput {
                _p: unsafe { i2s.clone_unchecked() },
            },
            input: I2sInput { _p: i2s },
        }
    }

    /// Enables the I2S module.
    #[inline(always)]
    pub fn enable(&self) -> &Self {
        let r = T::regs();
        r.enable.write(|w| w.enable().enabled());
        self
    }

    /// Disables the I2S module.
    #[inline(always)]
    pub fn disable(&self) -> &Self {
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());
        self
    }

    /// Starts I2S transfer.
    #[inline(always)]
    pub fn start(&self) -> &Self {
        let r = T::regs();
        self.enable();
        r.tasks_start.write(|w| unsafe { w.bits(1) });
        self
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        todo!()
    }

    /// Enables/disables I2S transmission (TX).
    #[inline(always)]
    pub fn set_tx_enabled(&self, enabled: bool) -> &Self {
        let r = T::regs();
        r.config.txen.write(|w| w.txen().bit(enabled));
        self
    }

    /// Enables/disables I2S reception (RX).
    #[inline(always)]
    pub fn set_rx_enabled(&self, enabled: bool) -> &Self {
        let r = T::regs();
        r.config.rxen.write(|w| w.rxen().bit(enabled));
        self
    }

    /// Transmits the given `buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    pub async fn tx<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        self.output.tx(buffer).await
    }

    /// Receives data into the given `buffer` until it's filled.
    /// Buffer address must be 4 byte aligned and located in RAM.
    pub async fn rx<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        self.input.rx(buffer).await
    }

    fn apply_config(c: &CONFIG, config: &Config) {
        match config.mode {
            Mode::Master { freq, ratio } => {
                c.mode.write(|w| w.mode().master());
                c.mcken.write(|w| w.mcken().enabled());
                c.mckfreq
                    .write(|w| unsafe { w.mckfreq().bits(freq.to_register_value()) });
                c.ratio.write(|w| unsafe { w.ratio().bits(ratio.into()) });
            }
            Mode::Slave => {
                c.mode.write(|w| w.mode().slave());
            }
        };

        c.swidth.write(|w| unsafe { w.swidth().bits(config.swidth.into()) });
        c.align.write(|w| w.align().bit(config.align.into()));
        c.format.write(|w| w.format().bit(config.format.into()));
        c.channels
            .write(|w| unsafe { w.channels().bits(config.channels.into()) });
    }

    fn select_pins(
        psel: &PSEL,
        mck: PeripheralRef<'d, AnyPin>,
        sck: PeripheralRef<'d, AnyPin>,
        lrck: PeripheralRef<'d, AnyPin>,
        sdin: PeripheralRef<'d, AnyPin>,
        sdout: PeripheralRef<'d, AnyPin>,
    ) {
        psel.mck.write(|w| {
            unsafe { w.bits(mck.psel_bits()) };
            w.connect().connected()
        });

        psel.sck.write(|w| {
            unsafe { w.bits(sck.psel_bits()) };
            w.connect().connected()
        });

        psel.lrck.write(|w| {
            unsafe { w.bits(lrck.psel_bits()) };
            w.connect().connected()
        });

        psel.sdin.write(|w| {
            unsafe { w.bits(sdin.psel_bits()) };
            w.connect().connected()
        });

        psel.sdout.write(|w| {
            unsafe { w.bits(sdout.psel_bits()) };
            w.connect().connected()
        });
    }

    fn setup_interrupt(irq: PeripheralRef<'d, T::Interrupt>, r: &RegisterBlock) {
        irq.set_handler(Self::on_interrupt);
        // irq.set_priority(Priority::P1); // TODO review priorities
        irq.unpend();
        irq.enable();

        r.intenclr.write(|w| w.rxptrupd().clear());
        r.intenclr.write(|w| w.txptrupd().clear());

        r.events_rxptrupd.reset();
        r.events_txptrupd.reset();

        r.intenset.write(|w| w.rxptrupd().set());
        r.intenset.write(|w| w.txptrupd().set());
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_txptrupd.read().bits() != 0 {
            trace!("[{}] INT", s.seq.load(Ordering::Relaxed));
            s.tx_waker.wake();
            r.intenclr.write(|w| w.txptrupd().clear());
        }

        if r.events_rxptrupd.read().bits() != 0 {
            s.rx_waker.wake();
            r.intenclr.write(|w| w.rxptrupd().clear());
        }

        s.overruns.fetch_add(1, Ordering::Relaxed);
    }
}

impl<'d, T: Instance> I2sOutput<'d, T> {
    /// Transmits the given `buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    #[allow(unused_mut)]
    pub async fn tx<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let ptr = buffer.bytes_ptr();
        let len = buffer.bytes_len();

        if ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }
        let maxcnt = ((len + core::mem::size_of::<u32>() - 1) / core::mem::size_of::<u32>()) as u32;
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }

        let r = T::regs();
        let s = T::state();

        let seq = s.seq.fetch_add(1, Ordering::Relaxed);
        if r.events_txptrupd.read().bits() != 0 && seq > 0 {
            info!("XRUN!");
            loop {}
        }

        let drop = OnDrop::new(move || {
            trace!("write drop: stopping");

            r.intenclr.write(|w| w.txptrupd().clear());
            r.events_txptrupd.reset();
            r.config.txen.write(|w| w.txen().disabled());

            // TX is stopped almost instantly, spinning is fine.
            while r.events_txptrupd.read().bits() == 0 {}
            trace!("write drop: stopped");
        });

        trace!("[{}] PTR", s.seq.load(Ordering::Relaxed));
        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            s.tx_waker.register(cx.waker());
            if r.events_txptrupd.read().bits() != 0 || seq == 0 {
                trace!("[{}] POLL Ready", s.seq.load(Ordering::Relaxed));
                r.events_txptrupd.reset();
                r.intenset.write(|w| w.txptrupd().set());
                let overruns = s.overruns.fetch_sub(1, Ordering::Relaxed);
                if overruns - 1 != 0 {
                    warn!("XRUN: {}", overruns);
                    s.overruns.store(0, Ordering::Relaxed)
                }
                Poll::Ready(())
            } else {
                trace!("[{}] POLL Pending", s.seq.load(Ordering::Relaxed));
                Poll::Pending
            }
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        drop.defuse();

        Ok(())
    }
}

impl<'d, T: Instance> I2sInput<'d, T> {
    /// Receives into the given `buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    #[allow(unused_mut)]
    pub async fn rx<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let ptr = buffer.bytes_ptr();
        let len = buffer.bytes_len();

        if ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }
        let maxcnt = ((len + core::mem::size_of::<u32>() - 1) / core::mem::size_of::<u32>()) as u32;
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }

        let r = T::regs();
        let _s = T::state();

        // TODO we can not progress until the last buffer written in RXD.PTR
        // has started the transmission.
        // We can use some sync primitive from `embassy-sync`.

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });

        Ok(())
    }
}

pub trait Buffer: Sized {
    fn bytes_ptr(&self) -> *const u8;
    fn bytes_len(&self) -> usize;
}

impl Buffer for &[u8] {
    #[inline]
    fn bytes_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    #[inline]
    fn bytes_len(&self) -> usize {
        self.len()
    }
}

impl Buffer for &[i16] {
    #[inline]
    fn bytes_ptr(&self) -> *const u8 {
        self.as_ptr() as *const u8
    }

    #[inline]
    fn bytes_len(&self) -> usize {
        self.len() * core::mem::size_of::<i16>()
    }
}

impl Buffer for &[i32] {
    #[inline]
    fn bytes_ptr(&self) -> *const u8 {
        self.as_ptr() as *const u8
    }

    #[inline]
    fn bytes_len(&self) -> usize {
        self.len() * core::mem::size_of::<i16>()
    }
}

pub(crate) mod sealed {
    use core::sync::atomic::AtomicI32;

    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub rx_waker: AtomicWaker,
        pub tx_waker: AtomicWaker,
        pub overruns: AtomicI32,
        pub seq: AtomicI32,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                rx_waker: AtomicWaker::new(),
                tx_waker: AtomicWaker::new(),
                overruns: AtomicI32::new(0),
                seq: AtomicI32::new(0),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static crate::pac::i2s::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    type Interrupt: Interrupt;
}

// TODO: Unsure why this macro is flagged as unused by CI when in fact it's used elsewhere?
#[allow(unused_macros)]
macro_rules! impl_i2s {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::i2s::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::i2s::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::i2s::sealed::State {
                static STATE: crate::i2s::sealed::State = crate::i2s::sealed::State::new();
                &STATE
            }
        }
        impl crate::i2s::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
