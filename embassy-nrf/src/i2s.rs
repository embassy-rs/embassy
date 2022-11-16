#![macro_use]

//! I2S

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::interrupt::{InterruptExt, Priority};
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

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
    BufferNotInDataMemory,
    BufferMisaligned,
    BufferLengthMisaligned,
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
    const REGISTER_VALUES: &'static [u32] = &[
        0x20000000, 0x18000000, 0x16000000, 0x11000000, 0x10000000, 0x0C000000, 0x0B000000, 0x08800000, 0x08400000,
        0x08000000, 0x06000000, 0x04100000, 0x020C0000,
    ];

    const FREQUENCIES: &'static [u32] = &[
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
    const RATIOS: &'static [u32] = &[32, 48, 64, 96, 128, 192, 256, 384, 512];

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

/// Interface to the I2S peripheral using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct I2S<'d, T: Instance> {
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

        Self { _p: i2s }
    }

    pub fn output(self) -> Output<'d, T> {
        Output { _p: self._p }
    }

    pub fn input(self) -> Input<'d, T> {
        Input { _p: self._p }
    }

    pub fn full_duplex(self) -> FullDuplex<'d, T> {
        FullDuplex { _p: self._p }
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

        let device = Device::<T>::new();
        device.disable_tx_ptr_interrupt();
        device.disable_rx_ptr_interrupt();

        device.reset_tx_ptr_event();
        device.reset_rx_ptr_event();

        device.enable_tx_ptr_interrupt();
        device.enable_rx_ptr_interrupt();
    }

    fn on_interrupt(_: *mut ()) {
        let device = Device::<T>::new();
        let s = T::state();

        if device.is_tx_ptr_updated() {
            trace!("TX INT");
            s.tx_waker.wake();
            device.disable_tx_ptr_interrupt();
        }

        if device.is_rx_ptr_updated() {
            trace!("RX INT");
            s.rx_waker.wake();
            device.disable_rx_ptr_interrupt();
        }
    }
}

pub struct Output<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Output<'d, T> {
    /// Starts I2S transfer.
    #[inline(always)]
    pub fn start<B>(&self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        // TODO what to do if it is started already?

        let device = Device::<T>::new();
        device.enable();
        device.set_tx_buffer(buffer)?;
        device.enable_tx();
        device.start();

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        todo!()
    }

    /// Transmits the given `buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    #[allow(unused_mut)]
    pub async fn send<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        trace!("SEND: {}", buffer.bytes_ptr() as u32);

        let device = Device::<T>::new();
        let drop = device.on_tx_drop();

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            if device.is_tx_ptr_updated() {
                trace!("TX POLL: Ready");
                device.reset_tx_ptr_event();
                device.enable_tx_ptr_interrupt();
                Poll::Ready(())
            } else {
                trace!("TX POLL: Pending");
                Poll::Pending
            }
        })
        .await;

        device.set_tx_buffer(buffer)?;

        compiler_fence(Ordering::SeqCst);
        drop.defuse();

        Ok(())
    }
}

pub struct Input<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Input<'d, T> {
    // TODO
}

pub struct FullDuplex<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> FullDuplex<'d, T> {
    // TODO
}

struct Device<T>(&'static RegisterBlock, PhantomData<T>);

impl<T: Instance> Device<T> {
    fn new() -> Self {
        Self(T::regs(), PhantomData)
    }

    #[inline(always)]
    pub fn enable(&self) {
        trace!("ENABLED");
        self.0.enable.write(|w| w.enable().enabled());
    }

    #[inline(always)]
    pub fn disable(&self) {
        trace!("DISABLED");
        self.0.enable.write(|w| w.enable().disabled());
    }

    #[inline(always)]
    fn enable_tx(&self) {
        trace!("TX ENABLED");
        self.0.config.txen.write(|w| w.txen().enabled());
    }

    #[inline(always)]
    fn disable_tx(&self) {
        trace!("TX DISABLED");
        self.0.config.txen.write(|w| w.txen().disabled());
    }

    #[inline(always)]
    fn enable_rx(&self) {
        trace!("RX ENABLED");
        self.0.config.rxen.write(|w| w.rxen().enabled());
    }

    #[inline(always)]
    fn disable_rx(&self) {
        trace!("RX DISABLED");
        self.0.config.rxen.write(|w| w.rxen().disabled());
    }

    #[inline(always)]
    fn start(&self) {
        trace!("START");
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    #[inline]
    fn set_tx_buffer<B>(&self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let (ptr, maxcnt) = Self::validate_buffer(buffer)?;
        self.0.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });
        self.0.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr) });
        Ok(())
    }

    #[inline]
    fn set_rx_buffer<B>(&self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let (ptr, maxcnt) = Self::validate_buffer(buffer)?;
        self.0.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });
        self.0.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr) });
        Ok(())
    }

    #[inline(always)]
    fn is_tx_ptr_updated(&self) -> bool {
        self.0.events_txptrupd.read().bits() != 0
    }

    #[inline(always)]
    fn is_rx_ptr_updated(&self) -> bool {
        self.0.events_rxptrupd.read().bits() != 0
    }

    #[inline(always)]
    fn reset_tx_ptr_event(&self) {
        trace!("TX PTR EVENT: Reset");
        self.0.events_txptrupd.reset();
    }

    #[inline(always)]
    fn reset_rx_ptr_event(&self) {
        trace!("RX PTR EVENT: Reset");
        self.0.events_rxptrupd.reset();
    }

    #[inline(always)]
    fn disable_tx_ptr_interrupt(&self) {
        trace!("TX PTR INTERRUPT: Disabled");
        self.0.intenclr.write(|w| w.txptrupd().clear());
    }

    #[inline(always)]
    fn disable_rx_ptr_interrupt(&self) {
        trace!("RX PTR INTERRUPT: Disabled");
        self.0.intenclr.write(|w| w.rxptrupd().clear());
    }

    #[inline(always)]
    fn enable_tx_ptr_interrupt(&self) {
        trace!("TX PTR INTERRUPT: Enabled");
        self.0.intenset.write(|w| w.txptrupd().set());
    }

    #[inline(always)]
    fn enable_rx_ptr_interrupt(&self) {
        trace!("RX PTR INTERRUPT: Enabled");
        self.0.intenclr.write(|w| w.rxptrupd().clear());
    }

    #[inline]
    fn on_tx_drop(&self) -> OnDrop<fn()> {
        OnDrop::new(move || {
            trace!("TX DROP: Stopping");

            let device = Device::<T>::new();
            device.disable_tx_ptr_interrupt();
            device.reset_tx_ptr_event();
            device.disable_tx();

            // TX is stopped almost instantly, spinning is fine.
            while !device.is_tx_ptr_updated() {}

            trace!("TX DROP: Stopped");
        })
    }

    fn validate_buffer<B>(buffer: B) -> Result<(u32, u32), Error>
    where
        B: Buffer,
    {
        let ptr = buffer.bytes_ptr() as u32;
        let len = buffer.bytes_len();
        let maxcnt = ((len + core::mem::size_of::<u32>() - 1) / core::mem::size_of::<u32>()) as u32;

        trace!("PTR={}, MAXCNT={}", ptr, maxcnt);

        // TODO can we avoid repeating all those runtime checks for the same buffer again and again?

        if ptr % 4 != 0 {
            Err(Error::BufferMisaligned)
        } else if len % 4 != 0 {
            Err(Error::BufferLengthMisaligned)
        } else if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            Err(Error::BufferNotInDataMemory)
        } else if maxcnt > MAX_DMA_MAXCNT {
            Err(Error::BufferTooLong)
        } else {
            Ok((ptr, maxcnt))
        }
    }
}

pub trait Buffer: Sized {
    fn bytes_ptr(&self) -> *const u8;
    fn bytes_len(&self) -> usize;
}

impl Buffer for &[i8] {
    #[inline]
    fn bytes_ptr(&self) -> *const u8 {
        self.as_ptr() as *const u8
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
        self.len() * core::mem::size_of::<i32>()
    }
}

pub(crate) mod sealed {
    use core::sync::atomic::AtomicI32;

    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub rx_waker: AtomicWaker,
        pub tx_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                rx_waker: AtomicWaker::new(),
                tx_waker: AtomicWaker::new(),
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
            fn regs() -> &'static crate::pac::i2s::RegisterBlock {
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
