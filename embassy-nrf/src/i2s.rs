#![macro_use]

//! Support for I2S audio

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::interrupt::InterruptExt;
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::Interrupt;
use crate::pac::i2s::RegisterBlock;
use crate::{Peripheral, EASY_DMA_SIZE};

// TODO: Define those in lib.rs somewhere else

/// Limits for Easy DMA - it can only read from data ram
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

/// Approximate sample rates.
///
/// Those are common sample rates that can not be configured without an small error.
///
/// For custom master clock configuration, please refer to [Mode].
#[derive(Clone, Copy)]
pub enum ApproxSampleRate {
    _11025,
    _16000,
    _22050,
    _32000,
    _44100,
    _48000,
}

impl From<ApproxSampleRate> for Mode {
    fn from(value: ApproxSampleRate) -> Self {
        match value {
            // error = 86
            ApproxSampleRate::_11025 => Mode::Master {
                freq: MckFreq::_32MDiv15,
                ratio: Ratio::_192x,
            },
            // error = 127
            ApproxSampleRate::_16000 => Mode::Master {
                freq: MckFreq::_32MDiv21,
                ratio: Ratio::_96x,
            },
            // error = 172
            ApproxSampleRate::_22050 => Mode::Master {
                freq: MckFreq::_32MDiv15,
                ratio: Ratio::_96x,
            },
            // error = 254
            ApproxSampleRate::_32000 => Mode::Master {
                freq: MckFreq::_32MDiv21,
                ratio: Ratio::_48x,
            },
            // error = 344
            ApproxSampleRate::_44100 => Mode::Master {
                freq: MckFreq::_32MDiv15,
                ratio: Ratio::_48x,
            },
            // error = 381
            ApproxSampleRate::_48000 => Mode::Master {
                freq: MckFreq::_32MDiv21,
                ratio: Ratio::_32x,
            },
        }
    }
}

impl ApproxSampleRate {
    pub fn sample_rate(&self) -> u32 {
        // This will always provide a Master mode, so it is safe to unwrap.
        Mode::from(*self).sample_rate().unwrap()
    }
}

/// Exact sample rates.
///
/// Those are non standard sample rates that can be configured without error.
///
/// For custom master clock configuration, please refer to [Mode].
#[derive(Clone, Copy)]
pub enum ExactSampleRate {
    _8000,
    _10582,
    _12500,
    _15625,
    _15873,
    _25000,
    _31250,
    _50000,
    _62500,
    _100000,
    _125000,
}

impl ExactSampleRate {
    pub fn sample_rate(&self) -> u32 {
        // This will always provide a Master mode, so it is safe to unwrap.
        Mode::from(*self).sample_rate().unwrap()
    }
}

impl From<ExactSampleRate> for Mode {
    fn from(value: ExactSampleRate) -> Self {
        match value {
            ExactSampleRate::_8000 => Mode::Master {
                freq: MckFreq::_32MDiv125,
                ratio: Ratio::_32x,
            },
            ExactSampleRate::_10582 => Mode::Master {
                freq: MckFreq::_32MDiv63,
                ratio: Ratio::_48x,
            },
            ExactSampleRate::_12500 => Mode::Master {
                freq: MckFreq::_32MDiv10,
                ratio: Ratio::_256x,
            },
            ExactSampleRate::_15625 => Mode::Master {
                freq: MckFreq::_32MDiv32,
                ratio: Ratio::_64x,
            },
            ExactSampleRate::_15873 => Mode::Master {
                freq: MckFreq::_32MDiv63,
                ratio: Ratio::_32x,
            },
            ExactSampleRate::_25000 => Mode::Master {
                freq: MckFreq::_32MDiv10,
                ratio: Ratio::_128x,
            },
            ExactSampleRate::_31250 => Mode::Master {
                freq: MckFreq::_32MDiv32,
                ratio: Ratio::_32x,
            },
            ExactSampleRate::_50000 => Mode::Master {
                freq: MckFreq::_32MDiv10,
                ratio: Ratio::_64x,
            },
            ExactSampleRate::_62500 => Mode::Master {
                freq: MckFreq::_32MDiv16,
                ratio: Ratio::_32x,
            },
            ExactSampleRate::_100000 => Mode::Master {
                freq: MckFreq::_32MDiv10,
                ratio: Ratio::_32x,
            },
            ExactSampleRate::_125000 => Mode::Master {
                freq: MckFreq::_32MDiv8,
                ratio: Ratio::_32x,
            },
        }
    }
}

/// I2S configuration.
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
            mode: ExactSampleRate::_31250.into(),
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

    /// Return the value that needs to be written to the register.
    pub fn to_register_value(&self) -> u32 {
        Self::REGISTER_VALUES[usize::from(*self)]
    }

    /// Return the master clock frequency.
    pub fn to_frequency(&self) -> u32 {
        Self::FREQUENCIES[usize::from(*self)]
    }
}

impl From<MckFreq> for usize {
    fn from(variant: MckFreq) -> Self {
        variant as _
    }
}

/// Master clock frequency ratio
///
/// Sample Rate = LRCK = MCK / Ratio
///
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

/// Sample width.
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

/// Channel used for the most significant sample value in a frame.
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

/// Channels
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Channels {
    Stereo,
    /// Mono left
    Left,
    /// Mono right
    Right,
}

impl From<Channels> for u8 {
    fn from(variant: Channels) -> Self {
        variant as _
    }
}

/// Interface to the I2S peripheral using EasyDMA to offload the transmission and reception workload.
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

        Self::apply_config(&config);
        Self::select_pins(mck, sck, lrck, sdin, sdout);
        Self::setup_interrupt(irq);

        T::regs().enable.write(|w| w.enable().enabled());

        Self { _p: i2s }
    }

    /// I2S output only
    pub fn output(self) -> Output<'d, T> {
        Output { _p: self._p }
    }

    /// I2S input only
    pub fn input(self) -> Input<'d, T> {
        Input { _p: self._p }
    }

    /// I2S full duplex (input and output)
    pub fn full_duplex(self) -> FullDuplex<'d, T> {
        FullDuplex { _p: self._p }
    }

    fn apply_config(config: &Config) {
        let c = &T::regs().config;
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
        mck: PeripheralRef<'d, AnyPin>,
        sck: PeripheralRef<'d, AnyPin>,
        lrck: PeripheralRef<'d, AnyPin>,
        sdin: PeripheralRef<'d, AnyPin>,
        sdout: PeripheralRef<'d, AnyPin>,
    ) {
        let psel = &T::regs().psel;

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

    fn setup_interrupt(irq: PeripheralRef<'d, T::Interrupt>) {
        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        let device = Device::<T>::new();
        device.disable_tx_ptr_interrupt();
        device.disable_rx_ptr_interrupt();
        device.disable_stopped_interrupt();

        device.reset_tx_ptr_event();
        device.reset_rx_ptr_event();
        device.reset_stopped_event();

        device.enable_tx_ptr_interrupt();
        device.enable_rx_ptr_interrupt();
        device.enable_stopped_interrupt();
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

        if device.is_stopped() {
            trace!("STOPPED INT");
            s.stop_waker.wake();
            device.disable_stopped_interrupt();
        }
    }

    async fn stop() {
        compiler_fence(Ordering::SeqCst);

        let device = Device::<T>::new();
        device.stop();

        T::state().started.store(false, Ordering::Relaxed);

        poll_fn(|cx| {
            T::state().stop_waker.register(cx.waker());

            if device.is_stopped() {
                trace!("STOP: Ready");
                device.reset_stopped_event();
                Poll::Ready(())
            } else {
                trace!("STOP: Pending");
                Poll::Pending
            }
        })
        .await;

        device.disable();
    }

    async fn send<B>(buffer: B) -> Result<(), Error>
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

    async fn receive<B>(buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        trace!("RECEIVE: {}", buffer.bytes_ptr() as u32);

        let device = Device::<T>::new();
        let drop = device.on_rx_drop();

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            T::state().rx_waker.register(cx.waker());

            if device.is_rx_ptr_updated() {
                trace!("RX POLL: Ready");
                device.reset_rx_ptr_event();
                device.enable_rx_ptr_interrupt();
                Poll::Ready(())
            } else {
                trace!("RX POLL: Pending");
                Poll::Pending
            }
        })
        .await;

        device.set_rx_buffer(buffer)?;

        compiler_fence(Ordering::SeqCst);
        drop.defuse();

        Ok(())
    }
}

/// I2S output
pub struct Output<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Output<'d, T> {
    /// Prepare the initial buffer and start the I2S transfer.
    pub async fn start<B>(&self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_tx();
        device.set_tx_buffer(buffer)?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sets the given `buffer` for transmission in the DMA.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// The buffer must not be written while being used by the DMA,
    /// which takes two other `send`s being awaited.
    #[allow(unused_mut)]
    pub async fn send<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        I2S::<T>::send(buffer).await
    }
}

/// I2S input
pub struct Input<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Input<'d, T> {
    /// Prepare the initial buffer and start the I2S transfer.
    pub async fn start<B>(&self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_rx();
        device.set_rx_buffer(buffer)?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sets the given `buffer` for reception from the DMA.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// The buffer must not be read while being used by the DMA,
    /// which takes two other `receive`s being awaited.
    #[allow(unused_mut)]
    pub async fn receive<B>(&mut self, buffer: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        I2S::<T>::receive(buffer).await
    }
}

/// I2S ful duplex (input & output)
pub struct FullDuplex<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> FullDuplex<'d, T> {
    /// Prepare the initial buffers and start the I2S transfer.
    pub async fn start<B>(&self, buffer_out: B, buffer_in: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_tx();
        device.enable_rx();
        device.set_tx_buffer(buffer_out)?;
        device.set_rx_buffer(buffer_in)?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sets the given `buffer_out` and `buffer_in` for transmission/reception from the DMA.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// The buffers must not be written/read while being used by the DMA,
    /// which takes two other `send_and_receive` operations being awaited.
    #[allow(unused_mut)]
    pub async fn send_and_receive<B>(&mut self, buffer_out: B, buffer_in: B) -> Result<(), Error>
    where
        B: Buffer,
    {
        I2S::<T>::send(buffer_out).await?;
        I2S::<T>::receive(buffer_in).await?;
        Ok(())
    }
}

/// Helper encapsulating common I2S device operations.
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

    #[inline(always)]
    fn stop(&self) {
        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    #[inline(always)]
    fn is_stopped(&self) -> bool {
        self.0.events_stopped.read().bits() != 0
    }

    #[inline(always)]
    fn reset_stopped_event(&self) {
        trace!("STOPPED EVENT: Reset");
        self.0.events_stopped.reset();
    }

    #[inline(always)]
    fn disable_stopped_interrupt(&self) {
        trace!("STOPPED INTERRUPT: Disabled");
        self.0.intenclr.write(|w| w.stopped().clear());
    }

    #[inline(always)]
    fn enable_stopped_interrupt(&self) {
        trace!("STOPPED INTERRUPT: Enabled");
        self.0.intenset.write(|w| w.stopped().set());
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

    #[inline]
    fn on_rx_drop(&self) -> OnDrop<fn()> {
        OnDrop::new(move || {
            trace!("RX DROP: Stopping");

            let device = Device::<T>::new();
            device.disable_rx_ptr_interrupt();
            device.reset_rx_ptr_event();
            device.disable_rx();

            // TX is stopped almost instantly, spinning is fine.
            while !device.is_rx_ptr_updated() {}

            trace!("RX DROP: Stopped");
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
        } else if maxcnt as usize > EASY_DMA_SIZE {
            Err(Error::BufferTooLong)
        } else {
            Ok((ptr, maxcnt))
        }
    }
}

/// Sample details
pub trait Sample: Sized + Copy + Default {
    const WIDTH: usize;
    const SCALE: Self;
}

impl Sample for i8 {
    const WIDTH: usize = 8;
    const SCALE: Self = 1 << (Self::WIDTH - 1);
}

impl Sample for i16 {
    const WIDTH: usize = 16;
    const SCALE: Self = 1 << (Self::WIDTH - 1);
}

impl Sample for i32 {
    const WIDTH: usize = 24;
    const SCALE: Self = 1 << (Self::WIDTH - 1);
}

/// A 4-bytes aligned [Buffer].
#[repr(align(4))]
pub struct AlignedBuffer<T: Sample, const N: usize>([T; N]);

impl<T: Sample, const N: usize> AlignedBuffer<T, N> {
    pub fn new(array: [T; N]) -> Self {
        Self(array)
    }
}

impl<T: Sample, const N: usize> Default for AlignedBuffer<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T: Sample, const N: usize> AsRef<[T]> for AlignedBuffer<T, N> {
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<T: Sample, const N: usize> AsMut<[T]> for AlignedBuffer<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

/// Common operations required for a buffer to be used by the DMA
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
    use core::sync::atomic::AtomicBool;

    use embassy_sync::waitqueue::AtomicWaker;

    /// Peripheral static state
    pub struct State {
        pub started: AtomicBool,
        pub rx_waker: AtomicWaker,
        pub tx_waker: AtomicWaker,
        pub stop_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                started: AtomicBool::new(false),
                rx_waker: AtomicWaker::new(),
                tx_waker: AtomicWaker::new(),
                stop_waker: AtomicWaker::new(),
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
