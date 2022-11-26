#![macro_use]

//! Support for I2S audio

use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::interrupt::InterruptExt;
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::Interrupt;
use crate::pac::i2s::RegisterBlock;
use crate::util::{slice_in_ram_or, slice_ptr_parts};
use crate::{Peripheral, EASY_DMA_SIZE};

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

/// I2S configuration.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub sample_width: SampleWidth,
    pub align: Align,
    pub format: Format,
    pub channels: Channels,
}

impl Config {
    pub fn sample_width(mut self, sample_width: SampleWidth) -> Self {
        self.sample_width = sample_width;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn channels(mut self, channels: Channels) -> Self {
        self.channels = channels;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_width: SampleWidth::_16bit,
            align: Align::Left,
            format: Format::I2S,
            channels: Channels::Stereo,
        }
    }
}

/// I2S Mode
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct MasterClock {
    freq: MckFreq,
    ratio: Ratio,
}

impl MasterClock {
    pub fn new(freq: MckFreq, ratio: Ratio) -> Self {
        Self { freq, ratio }
    }
}

impl MasterClock {
    pub fn sample_rate(&self) -> u32 {
        self.freq.to_frequency() / self.ratio.to_divisor()
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

    /// Return the value that needs to be written to the register.
    pub fn to_register_value(&self) -> u8 {
        usize::from(*self) as u8
    }

    pub fn to_divisor(&self) -> u32 {
        Self::RATIOS[usize::from(*self)]
    }
}

impl From<Ratio> for usize {
    fn from(variant: Ratio) -> Self {
        variant as _
    }
}

/// Approximate sample rates.
///
/// Those are common sample rates that can not be configured without an small error.
///
/// For custom master clock configuration, please refer to [MasterClock].
#[derive(Clone, Copy)]
pub enum ApproxSampleRate {
    _11025,
    _16000,
    _22050,
    _32000,
    _44100,
    _48000,
}

impl From<ApproxSampleRate> for MasterClock {
    fn from(value: ApproxSampleRate) -> Self {
        match value {
            // error = 86
            ApproxSampleRate::_11025 => MasterClock::new(MckFreq::_32MDiv15, Ratio::_192x),
            // error = 127
            ApproxSampleRate::_16000 => MasterClock::new(MckFreq::_32MDiv21, Ratio::_96x),
            // error = 172
            ApproxSampleRate::_22050 => MasterClock::new(MckFreq::_32MDiv15, Ratio::_96x),
            // error = 254
            ApproxSampleRate::_32000 => MasterClock::new(MckFreq::_32MDiv21, Ratio::_48x),
            // error = 344
            ApproxSampleRate::_44100 => MasterClock::new(MckFreq::_32MDiv15, Ratio::_48x),
            // error = 381
            ApproxSampleRate::_48000 => MasterClock::new(MckFreq::_32MDiv21, Ratio::_32x),
        }
    }
}

impl ApproxSampleRate {
    pub fn sample_rate(&self) -> u32 {
        MasterClock::from(*self).sample_rate()
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
        MasterClock::from(*self).sample_rate()
    }
}

impl From<ExactSampleRate> for MasterClock {
    fn from(value: ExactSampleRate) -> Self {
        match value {
            ExactSampleRate::_8000 => MasterClock::new(MckFreq::_32MDiv125, Ratio::_32x),
            ExactSampleRate::_10582 => MasterClock::new(MckFreq::_32MDiv63, Ratio::_48x),
            ExactSampleRate::_12500 => MasterClock::new(MckFreq::_32MDiv10, Ratio::_256x),
            ExactSampleRate::_15625 => MasterClock::new(MckFreq::_32MDiv32, Ratio::_64x),
            ExactSampleRate::_15873 => MasterClock::new(MckFreq::_32MDiv63, Ratio::_32x),
            ExactSampleRate::_25000 => MasterClock::new(MckFreq::_32MDiv10, Ratio::_128x),
            ExactSampleRate::_31250 => MasterClock::new(MckFreq::_32MDiv32, Ratio::_32x),
            ExactSampleRate::_50000 => MasterClock::new(MckFreq::_32MDiv10, Ratio::_64x),
            ExactSampleRate::_62500 => MasterClock::new(MckFreq::_32MDiv16, Ratio::_32x),
            ExactSampleRate::_100000 => MasterClock::new(MckFreq::_32MDiv10, Ratio::_32x),
            ExactSampleRate::_125000 => MasterClock::new(MckFreq::_32MDiv8, Ratio::_32x),
        }
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
    MonoLeft,
    MonoRight,
}

impl From<Channels> for u8 {
    fn from(variant: Channels) -> Self {
        variant as _
    }
}

/// Interface to the I2S peripheral using EasyDMA to offload the transmission and reception workload.
pub struct I2S<'d, T: Instance> {
    i2s: PeripheralRef<'d, T>,
    irq: PeripheralRef<'d, T::Interrupt>,
    mck: Option<PeripheralRef<'d, AnyPin>>,
    sck: PeripheralRef<'d, AnyPin>,
    lrck: PeripheralRef<'d, AnyPin>,
    sdin: Option<PeripheralRef<'d, AnyPin>>,
    sdout: Option<PeripheralRef<'d, AnyPin>>,
    master_clock: Option<MasterClock>,
    config: Config,
}

impl<'d, T: Instance> I2S<'d, T> {
    /// Create a new I2S in master mode
    pub fn master(
        i2s: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        mck: impl Peripheral<P = impl GpioPin> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        lrck: impl Peripheral<P = impl GpioPin> + 'd,
        master_clock: MasterClock,
        config: Config,
    ) -> Self {
        into_ref!(i2s, irq, mck, sck, lrck);
        Self {
            i2s,
            irq,
            mck: Some(mck.map_into()),
            sck: sck.map_into(),
            lrck: lrck.map_into(),
            sdin: None,
            sdout: None,
            master_clock: Some(master_clock),
            config,
        }
    }

    /// Create a new I2S in slave mode
    pub fn slave(
        i2s: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        lrck: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(i2s, irq, sck, lrck);
        Self {
            i2s,
            irq,
            mck: None,
            sck: sck.map_into(),
            lrck: lrck.map_into(),
            sdin: None,
            sdout: None,
            master_clock: None,
            config,
        }
    }

    /// I2S output only
    pub fn output(mut self, sdout: impl Peripheral<P = impl GpioPin> + 'd) -> OutputStream<'d, T> {
        self.sdout = Some(sdout.into_ref().map_into());
        OutputStream { _p: self.build() }
    }

    /// I2S input only
    pub fn input(mut self, sdin: impl Peripheral<P = impl GpioPin> + 'd) -> InputStream<'d, T> {
        self.sdin = Some(sdin.into_ref().map_into());
        InputStream { _p: self.build() }
    }

    /// I2S full duplex (input and output)
    pub fn full_duplex(
        mut self,
        sdin: impl Peripheral<P = impl GpioPin> + 'd,
        sdout: impl Peripheral<P = impl GpioPin> + 'd,
    ) -> FullDuplexStream<'d, T> {
        self.sdout = Some(sdout.into_ref().map_into());
        self.sdin = Some(sdin.into_ref().map_into());
        FullDuplexStream { _p: self.build() }
    }

    fn build(self) -> PeripheralRef<'d, T> {
        self.apply_config();
        self.select_pins();
        self.setup_interrupt();

        let device = Device::<T>::new();
        device.enable();

        self.i2s
    }

    fn apply_config(&self) {
        let c = &T::regs().config;
        match &self.master_clock {
            Some(MasterClock { freq, ratio }) => {
                c.mode.write(|w| w.mode().master());
                c.mcken.write(|w| w.mcken().enabled());
                c.mckfreq
                    .write(|w| unsafe { w.mckfreq().bits(freq.to_register_value()) });
                c.ratio.write(|w| unsafe { w.ratio().bits(ratio.to_register_value()) });
            }
            None => {
                c.mode.write(|w| w.mode().slave());
            }
        };

        c.swidth
            .write(|w| unsafe { w.swidth().bits(self.config.sample_width.into()) });
        c.align.write(|w| w.align().bit(self.config.align.into()));
        c.format.write(|w| w.format().bit(self.config.format.into()));
        c.channels
            .write(|w| unsafe { w.channels().bits(self.config.channels.into()) });
    }

    fn select_pins(&self) {
        let psel = &T::regs().psel;

        if let Some(mck) = &self.mck {
            psel.mck.write(|w| {
                unsafe { w.bits(mck.psel_bits()) };
                w.connect().connected()
            });
        }

        psel.sck.write(|w| {
            unsafe { w.bits(self.sck.psel_bits()) };
            w.connect().connected()
        });

        psel.lrck.write(|w| {
            unsafe { w.bits(self.lrck.psel_bits()) };
            w.connect().connected()
        });

        if let Some(sdin) = &self.sdin {
            psel.sdin.write(|w| {
                unsafe { w.bits(sdin.psel_bits()) };
                w.connect().connected()
            });
        }

        if let Some(sdout) = &self.sdout {
            psel.sdout.write(|w| {
                unsafe { w.bits(sdout.psel_bits()) };
                w.connect().connected()
            });
        }
    }

    fn setup_interrupt(&self) {
        self.irq.set_handler(Self::on_interrupt);
        self.irq.unpend();
        self.irq.enable();

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

    async fn send_from_ram<S>(buffer_ptr: *const [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        trace!("SEND: {}", buffer_ptr as *const S as u32);

        slice_in_ram_or(buffer_ptr, Error::BufferNotInDataMemory)?;

        compiler_fence(Ordering::SeqCst);

        let device = Device::<T>::new();

        let drop = OnDrop::new(move || {
            trace!("TX DROP: Stopping");

            let device = Device::<T>::new();
            device.disable_tx_ptr_interrupt();
            device.reset_tx_ptr_event();
            device.disable_tx();

            // TX is stopped almost instantly, spinning is fine.
            while !device.is_tx_ptr_updated() {}

            trace!("TX DROP: Stopped");
        });

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

        device.update_tx(buffer_ptr)?;

        compiler_fence(Ordering::SeqCst);
        drop.defuse();

        Ok(())
    }

    async fn receive_from_ram<S>(buffer_ptr: *mut [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        trace!("RECEIVE: {}", buffer_ptr as *const S as u32);

        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        compiler_fence(Ordering::SeqCst);

        let device = Device::<T>::new();

        let drop = OnDrop::new(move || {
            trace!("RX DROP: Stopping");

            let device = Device::<T>::new();
            device.disable_rx_ptr_interrupt();
            device.reset_rx_ptr_event();
            device.disable_rx();

            // TX is stopped almost instantly, spinning is fine.
            while !device.is_rx_ptr_updated() {}

            trace!("RX DROP: Stopped");
        });

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

        device.update_rx(buffer_ptr)?;

        compiler_fence(Ordering::SeqCst);

        drop.defuse();

        Ok(())
    }
}

/// I2S output
pub struct OutputStream<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> OutputStream<'d, T> {
    /// Prepare the initial buffer and start the I2S transfer.
    #[allow(unused_mut)]
    pub async fn start<S>(&mut self, buffer: &[S]) -> Result<(), Error>
    where
        S: Sample,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_tx();

        device.update_tx(buffer as *const [S])?;

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
    pub async fn send_from_ram<S>(&mut self, buffer: &[S]) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::send_from_ram(buffer as *const [S]).await
    }
}

/// I2S input
pub struct InputStream<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> InputStream<'d, T> {
    /// Prepare the initial buffer and start the I2S transfer.
    #[allow(unused_mut)]
    pub async fn start<S>(&mut self, buffer: &mut [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_rx();

        device.update_rx(buffer as *mut [S])?;

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
    pub async fn receive_from_ram<S>(&mut self, buffer: &mut [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::receive_from_ram(buffer as *mut [S]).await
    }
}

/// I2S full duplex stream (input & output)
pub struct FullDuplexStream<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> FullDuplexStream<'d, T> {
    /// Prepare the initial buffers and start the I2S transfer.
    #[allow(unused_mut)]
    pub async fn start<S>(&mut self, buffer_out: &[S], buffer_in: &mut [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        let device = Device::<T>::new();

        let s = T::state();
        if s.started.load(Ordering::Relaxed) {
            self.stop().await;
        }

        device.enable();
        device.enable_tx();
        device.enable_rx();

        device.update_tx(buffer_out as *const [S])?;
        device.update_rx(buffer_in as *mut [S])?;

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
    pub async fn send_and_receive_from_ram<S>(&mut self, buffer_out: &[S], buffer_in: &mut [S]) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::send_from_ram(buffer_out as *const [S]).await?;
        I2S::<T>::receive_from_ram(buffer_in as *mut [S]).await?;
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

    #[inline(always)]
    fn is_tx_ptr_updated(&self) -> bool {
        self.0.events_txptrupd.read().bits() != 0
    }

    #[inline(always)]
    fn is_rx_ptr_updated(&self) -> bool {
        self.0.events_rxptrupd.read().bits() != 0
    }

    #[inline]
    fn update_tx<S>(&self, buffer_ptr: *const [S]) -> Result<(), Error> {
        let (ptr, maxcnt) = Self::validated_dma_parts(buffer_ptr)?;
        self.0.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });
        self.0.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr) });
        Ok(())
    }

    #[inline]
    fn update_rx<S>(&self, buffer_ptr: *const [S]) -> Result<(), Error> {
        let (ptr, maxcnt) = Self::validated_dma_parts(buffer_ptr)?;
        self.0.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });
        self.0.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr) });
        Ok(())
    }

    fn validated_dma_parts<S>(buffer_ptr: *const [S]) -> Result<(u32, u32), Error> {
        let (ptr, len) = slice_ptr_parts(buffer_ptr);
        let ptr = ptr as u32;
        let bytes_len = len * size_of::<S>();
        let maxcnt = (bytes_len / size_of::<u32>()) as u32;

        trace!("PTR={}, MAXCNT={}", ptr, maxcnt);

        if ptr % 4 != 0 {
            Err(Error::BufferMisaligned)
        } else if bytes_len % 4 != 0 {
            Err(Error::BufferLengthMisaligned)
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

impl<T: Sample, const N: usize> Deref for AlignedBuffer<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<T: Sample, const N: usize> DerefMut for AlignedBuffer<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
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
