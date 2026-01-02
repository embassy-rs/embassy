//! Inter-IC Sound (I2S) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::i2s::vals;
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac, EASY_DMA_SIZE};

/// Type alias for `MultiBuffering` with 2 buffers.
pub type DoubleBuffering<S, const NS: usize> = MultiBuffering<S, 2, NS>;

/// I2S transfer error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The buffer is too long.
    BufferTooLong,
    /// The buffer is empty.
    BufferZeroLength,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
    /// The buffer address is not aligned.
    BufferMisaligned,
    /// The buffer length is not a multiple of the alignment.
    BufferLengthMisaligned,
}

/// I2S configuration.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    /// Sample width
    pub sample_width: SampleWidth,
    /// Alignment
    pub align: Align,
    /// Sample format
    pub format: Format,
    /// Channel configuration.
    pub channels: Channels,
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

/// I2S clock configuration.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct MasterClock {
    freq: MckFreq,
    ratio: Ratio,
}

impl MasterClock {
    /// Create a new `MasterClock`.
    pub fn new(freq: MckFreq, ratio: Ratio) -> Self {
        Self { freq, ratio }
    }
}

impl MasterClock {
    /// Get the sample rate for this clock configuration.
    pub fn sample_rate(&self) -> u32 {
        self.freq.to_frequency() / self.ratio.to_divisor()
    }
}

/// Master clock generator frequency.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MckFreq {
    /// 32 Mhz / 8 = 4000.00 kHz
    _32MDiv8,
    /// 32 Mhz / 10 = 3200.00 kHz
    _32MDiv10,
    /// 32 Mhz / 11 = 2909.09 kHz
    _32MDiv11,
    /// 32 Mhz / 15 = 2133.33 kHz
    _32MDiv15,
    /// 32 Mhz / 16 = 2000.00 kHz
    _32MDiv16,
    /// 32 Mhz / 21 = 1523.81 kHz
    _32MDiv21,
    /// 32 Mhz / 23 = 1391.30 kHz
    _32MDiv23,
    /// 32 Mhz / 30 = 1066.67 kHz
    _32MDiv30,
    /// 32 Mhz / 31 = 1032.26 kHz
    _32MDiv31,
    /// 32 Mhz / 32 = 1000.00 kHz
    _32MDiv32,
    /// 32 Mhz / 42 = 761.90 kHz
    _32MDiv42,
    /// 32 Mhz / 63 = 507.94 kHz
    _32MDiv63,
    /// 32 Mhz / 125 = 256.00 kHz
    _32MDiv125,
}

impl MckFreq {
    const REGISTER_VALUES: &'static [vals::Mckfreq] = &[
        vals::Mckfreq::_32MDIV8,
        vals::Mckfreq::_32MDIV10,
        vals::Mckfreq::_32MDIV11,
        vals::Mckfreq::_32MDIV15,
        vals::Mckfreq::_32MDIV16,
        vals::Mckfreq::_32MDIV21,
        vals::Mckfreq::_32MDIV23,
        vals::Mckfreq::_32MDIV30,
        vals::Mckfreq::_32MDIV31,
        vals::Mckfreq::_32MDIV32,
        vals::Mckfreq::_32MDIV42,
        vals::Mckfreq::_32MDIV63,
        vals::Mckfreq::_32MDIV125,
    ];

    const FREQUENCIES: &'static [u32] = &[
        4000000, 3200000, 2909090, 2133333, 2000000, 1523809, 1391304, 1066666, 1032258, 1000000, 761904, 507936,
        256000,
    ];

    /// Return the value that needs to be written to the register.
    pub fn to_register_value(&self) -> vals::Mckfreq {
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
    /// Divide by 32
    _32x,
    /// Divide by 48
    _48x,
    /// Divide by 64
    _64x,
    /// Divide by 96
    _96x,
    /// Divide by 128
    _128x,
    /// Divide by 192
    _192x,
    /// Divide by 256
    _256x,
    /// Divide by 384
    _384x,
    /// Divide by 512
    _512x,
}

impl Ratio {
    const RATIOS: &'static [u32] = &[32, 48, 64, 96, 128, 192, 256, 384, 512];

    /// Return the value that needs to be written to the register.
    pub fn to_register_value(&self) -> vals::Ratio {
        vals::Ratio::from_bits(*self as u8)
    }

    /// Return the divisor for this ratio
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
    /// 11025 Hz
    _11025,
    /// 16000 Hz
    _16000,
    /// 22050 Hz
    _22050,
    /// 32000 Hz
    _32000,
    /// 44100 Hz
    _44100,
    /// 48000 Hz
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
    /// Get the sample rate as an integer.
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
    /// 8000 Hz
    _8000,
    /// 10582 Hz
    _10582,
    /// 12500 Hz
    _12500,
    /// 15625 Hz
    _15625,
    /// 15873 Hz
    _15873,
    /// 25000 Hz
    _25000,
    /// 31250 Hz
    _31250,
    /// 50000 Hz
    _50000,
    /// 62500 Hz
    _62500,
    /// 100000 Hz
    _100000,
    /// 125000 Hz
    _125000,
}

impl ExactSampleRate {
    /// Get the sample rate as an integer.
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
    /// 8 bit samples.
    _8bit,
    /// 16 bit samples.
    _16bit,
    /// 24 bit samples.
    _24bit,
}

impl From<SampleWidth> for vals::Swidth {
    fn from(variant: SampleWidth) -> Self {
        vals::Swidth::from_bits(variant as u8)
    }
}

/// Channel used for the most significant sample value in a frame.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Align {
    /// Left-align samples.
    Left,
    /// Right-align samples.
    Right,
}

impl From<Align> for vals::Align {
    fn from(variant: Align) -> Self {
        match variant {
            Align::Left => vals::Align::LEFT,
            Align::Right => vals::Align::RIGHT,
        }
    }
}

/// Frame format.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Format {
    /// I2S frame format
    I2S,
    /// Aligned frame format
    Aligned,
}

impl From<Format> for vals::Format {
    fn from(variant: Format) -> Self {
        match variant {
            Format::I2S => vals::Format::I2S,
            Format::Aligned => vals::Format::ALIGNED,
        }
    }
}

/// Channels
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Channels {
    /// Stereo (2 channels).
    Stereo,
    /// Mono, left channel only.
    MonoLeft,
    /// Mono, right channel only.
    MonoRight,
}

impl From<Channels> for vals::Channels {
    fn from(variant: Channels) -> Self {
        vals::Channels::from_bits(variant as u8)
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
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
}

/// I2S driver.
pub struct I2S<'d, T: Instance> {
    i2s: Peri<'d, T>,
    mck: Option<Peri<'d, AnyPin>>,
    sck: Peri<'d, AnyPin>,
    lrck: Peri<'d, AnyPin>,
    sdin: Option<Peri<'d, AnyPin>>,
    sdout: Option<Peri<'d, AnyPin>>,
    master_clock: Option<MasterClock>,
    config: Config,
}

impl<'d, T: Instance> I2S<'d, T> {
    /// Create a new I2S in master mode
    pub fn new_master(
        i2s: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        mck: Peri<'d, impl GpioPin>,
        sck: Peri<'d, impl GpioPin>,
        lrck: Peri<'d, impl GpioPin>,
        master_clock: MasterClock,
        config: Config,
    ) -> Self {
        Self {
            i2s,
            mck: Some(mck.into()),
            sck: sck.into(),
            lrck: lrck.into(),
            sdin: None,
            sdout: None,
            master_clock: Some(master_clock),
            config,
        }
    }

    /// Create a new I2S in slave mode
    pub fn new_slave(
        i2s: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: Peri<'d, impl GpioPin>,
        lrck: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self {
            i2s,
            mck: None,
            sck: sck.into(),
            lrck: lrck.into(),
            sdin: None,
            sdout: None,
            master_clock: None,
            config,
        }
    }

    /// I2S output only
    pub fn output<S: Sample, const NB: usize, const NS: usize>(
        mut self,
        sdout: Peri<'d, impl GpioPin>,
        buffers: MultiBuffering<S, NB, NS>,
    ) -> OutputStream<'d, T, S, NB, NS> {
        self.sdout = Some(sdout.into());
        OutputStream {
            _p: self.build(),
            buffers,
        }
    }

    /// I2S input only
    pub fn input<S: Sample, const NB: usize, const NS: usize>(
        mut self,
        sdin: Peri<'d, impl GpioPin>,
        buffers: MultiBuffering<S, NB, NS>,
    ) -> InputStream<'d, T, S, NB, NS> {
        self.sdin = Some(sdin.into());
        InputStream {
            _p: self.build(),
            buffers,
        }
    }

    /// I2S full duplex (input and output)
    pub fn full_duplex<S: Sample, const NB: usize, const NS: usize>(
        mut self,
        sdin: Peri<'d, impl GpioPin>,
        sdout: Peri<'d, impl GpioPin>,
        buffers_out: MultiBuffering<S, NB, NS>,
        buffers_in: MultiBuffering<S, NB, NS>,
    ) -> FullDuplexStream<'d, T, S, NB, NS> {
        self.sdout = Some(sdout.into());
        self.sdin = Some(sdin.into());

        FullDuplexStream {
            _p: self.build(),
            buffers_out,
            buffers_in,
        }
    }

    fn build(self) -> Peri<'d, T> {
        self.apply_config();
        self.select_pins();
        self.setup_interrupt();

        let device = Device::<T>::new();
        device.enable();

        self.i2s
    }

    fn apply_config(&self) {
        let c = T::regs().config();
        match &self.master_clock {
            Some(MasterClock { freq, ratio }) => {
                c.mode().write(|w| w.set_mode(vals::Mode::MASTER));
                c.mcken().write(|w| w.set_mcken(true));
                c.mckfreq().write(|w| w.set_mckfreq(freq.to_register_value()));
                c.ratio().write(|w| w.set_ratio(ratio.to_register_value()));
            }
            None => {
                c.mode().write(|w| w.set_mode(vals::Mode::SLAVE));
            }
        };

        c.swidth().write(|w| w.set_swidth(self.config.sample_width.into()));
        c.align().write(|w| w.set_align(self.config.align.into()));
        c.format().write(|w| w.set_format(self.config.format.into()));
        c.channels().write(|w| w.set_channels(self.config.channels.into()));
    }

    fn select_pins(&self) {
        let psel = T::regs().psel();
        psel.mck().write_value(self.mck.psel_bits());
        psel.sck().write_value(self.sck.psel_bits());
        psel.lrck().write_value(self.lrck.psel_bits());
        psel.sdin().write_value(self.sdin.psel_bits());
        psel.sdout().write_value(self.sdout.psel_bits());
    }

    fn setup_interrupt(&self) {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

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

        slice_in_ram_or(buffer_ptr, Error::BufferNotInRAM)?;

        compiler_fence(Ordering::SeqCst);

        let device = Device::<T>::new();

        device.update_tx(buffer_ptr)?;

        Self::wait_tx_ptr_update().await;

        compiler_fence(Ordering::SeqCst);

        Ok(())
    }

    async fn wait_tx_ptr_update() {
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

            let device = Device::<T>::new();
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

        drop.defuse();
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

        device.update_rx(buffer_ptr)?;

        Self::wait_rx_ptr_update().await;

        compiler_fence(Ordering::SeqCst);

        Ok(())
    }

    async fn wait_rx_ptr_update() {
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

            let device = Device::<T>::new();
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

        drop.defuse();
    }
}

/// I2S output
pub struct OutputStream<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> {
    _p: Peri<'d, T>,
    buffers: MultiBuffering<S, NB, NS>,
}

impl<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> OutputStream<'d, T, S, NB, NS> {
    /// Get a mutable reference to the current buffer.
    pub fn buffer(&mut self) -> &mut [S] {
        self.buffers.get_mut()
    }

    /// Prepare the initial buffer and start the I2S transfer.
    pub async fn start(&mut self) -> Result<(), Error>
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

        device.update_tx(self.buffers.switch())?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        I2S::<T>::wait_tx_ptr_update().await;

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sends the current buffer for transmission in the DMA.
    /// Switches to use the next available buffer.
    pub async fn send(&mut self) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::send_from_ram(self.buffers.switch()).await
    }
}

/// I2S input
pub struct InputStream<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> {
    _p: Peri<'d, T>,
    buffers: MultiBuffering<S, NB, NS>,
}

impl<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> InputStream<'d, T, S, NB, NS> {
    /// Get a mutable reference to the current buffer.
    pub fn buffer(&mut self) -> &mut [S] {
        self.buffers.get_mut()
    }

    /// Prepare the initial buffer and start the I2S transfer.
    pub async fn start(&mut self) -> Result<(), Error>
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

        device.update_rx(self.buffers.switch())?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        I2S::<T>::wait_rx_ptr_update().await;

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sets the current buffer for reception from the DMA.
    /// Switches to use the next available buffer.
    #[allow(unused_mut)]
    pub async fn receive(&mut self) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::receive_from_ram(self.buffers.switch_mut()).await
    }
}

/// I2S full duplex stream (input & output)
pub struct FullDuplexStream<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> {
    _p: Peri<'d, T>,
    buffers_out: MultiBuffering<S, NB, NS>,
    buffers_in: MultiBuffering<S, NB, NS>,
}

impl<'d, T: Instance, S: Sample, const NB: usize, const NS: usize> FullDuplexStream<'d, T, S, NB, NS> {
    /// Get the current output and input buffers.
    pub fn buffers(&mut self) -> (&mut [S], &[S]) {
        (self.buffers_out.get_mut(), self.buffers_in.get())
    }

    /// Prepare the initial buffers and start the I2S transfer.
    pub async fn start(&mut self) -> Result<(), Error>
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

        device.update_tx(self.buffers_out.switch())?;
        device.update_rx(self.buffers_in.switch_mut())?;

        s.started.store(true, Ordering::Relaxed);

        device.start();

        I2S::<T>::wait_tx_ptr_update().await;
        I2S::<T>::wait_rx_ptr_update().await;

        Ok(())
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub async fn stop(&self) {
        I2S::<T>::stop().await
    }

    /// Sets the current buffers for output and input for transmission/reception from the DMA.
    /// Switch to use the next available buffers for output/input.
    pub async fn send_and_receive(&mut self) -> Result<(), Error>
    where
        S: Sample,
    {
        I2S::<T>::send_from_ram(self.buffers_out.switch()).await?;
        I2S::<T>::receive_from_ram(self.buffers_in.switch_mut()).await?;
        Ok(())
    }
}

/// Helper encapsulating common I2S device operations.
struct Device<T>(pac::i2s::I2s, PhantomData<T>);

impl<T: Instance> Device<T> {
    fn new() -> Self {
        Self(T::regs(), PhantomData)
    }

    #[inline(always)]
    pub fn enable(&self) {
        trace!("ENABLED");
        self.0.enable().write(|w| w.set_enable(true));
    }

    #[inline(always)]
    pub fn disable(&self) {
        trace!("DISABLED");
        self.0.enable().write(|w| w.set_enable(false));
    }

    #[inline(always)]
    fn enable_tx(&self) {
        trace!("TX ENABLED");
        self.0.config().txen().write(|w| w.set_txen(true));
    }

    #[inline(always)]
    fn disable_tx(&self) {
        trace!("TX DISABLED");
        self.0.config().txen().write(|w| w.set_txen(false));
    }

    #[inline(always)]
    fn enable_rx(&self) {
        trace!("RX ENABLED");
        self.0.config().rxen().write(|w| w.set_rxen(true));
    }

    #[inline(always)]
    fn disable_rx(&self) {
        trace!("RX DISABLED");
        self.0.config().rxen().write(|w| w.set_rxen(false));
    }

    #[inline(always)]
    fn start(&self) {
        trace!("START");
        self.0.tasks_start().write_value(1);
    }

    #[inline(always)]
    fn stop(&self) {
        self.0.tasks_stop().write_value(1);
    }

    #[inline(always)]
    fn is_stopped(&self) -> bool {
        self.0.events_stopped().read() != 0
    }

    #[inline(always)]
    fn reset_stopped_event(&self) {
        trace!("STOPPED EVENT: Reset");
        self.0.events_stopped().write_value(0);
    }

    #[inline(always)]
    fn disable_stopped_interrupt(&self) {
        trace!("STOPPED INTERRUPT: Disabled");
        self.0.intenclr().write(|w| w.set_stopped(true));
    }

    #[inline(always)]
    fn enable_stopped_interrupt(&self) {
        trace!("STOPPED INTERRUPT: Enabled");
        self.0.intenset().write(|w| w.set_stopped(true));
    }

    #[inline(always)]
    fn reset_tx_ptr_event(&self) {
        trace!("TX PTR EVENT: Reset");
        self.0.events_txptrupd().write_value(0);
    }

    #[inline(always)]
    fn reset_rx_ptr_event(&self) {
        trace!("RX PTR EVENT: Reset");
        self.0.events_rxptrupd().write_value(0);
    }

    #[inline(always)]
    fn disable_tx_ptr_interrupt(&self) {
        trace!("TX PTR INTERRUPT: Disabled");
        self.0.intenclr().write(|w| w.set_txptrupd(true));
    }

    #[inline(always)]
    fn disable_rx_ptr_interrupt(&self) {
        trace!("RX PTR INTERRUPT: Disabled");
        self.0.intenclr().write(|w| w.set_rxptrupd(true));
    }

    #[inline(always)]
    fn enable_tx_ptr_interrupt(&self) {
        trace!("TX PTR INTERRUPT: Enabled");
        self.0.intenset().write(|w| w.set_txptrupd(true));
    }

    #[inline(always)]
    fn enable_rx_ptr_interrupt(&self) {
        trace!("RX PTR INTERRUPT: Enabled");
        self.0.intenset().write(|w| w.set_rxptrupd(true));
    }

    #[inline(always)]
    fn is_tx_ptr_updated(&self) -> bool {
        self.0.events_txptrupd().read() != 0
    }

    #[inline(always)]
    fn is_rx_ptr_updated(&self) -> bool {
        self.0.events_rxptrupd().read() != 0
    }

    #[inline]
    fn update_tx<S>(&self, buffer_ptr: *const [S]) -> Result<(), Error> {
        let (ptr, maxcnt) = Self::validated_dma_parts(buffer_ptr)?;
        self.0.rxtxd().maxcnt().write(|w| w.0 = maxcnt);
        self.0.txd().ptr().write_value(ptr);
        Ok(())
    }

    #[inline]
    fn update_rx<S>(&self, buffer_ptr: *const [S]) -> Result<(), Error> {
        let (ptr, maxcnt) = Self::validated_dma_parts(buffer_ptr)?;
        self.0.rxtxd().maxcnt().write(|w| w.0 = maxcnt);
        self.0.rxd().ptr().write_value(ptr);
        Ok(())
    }

    fn validated_dma_parts<S>(buffer_ptr: *const [S]) -> Result<(u32, u32), Error> {
        let ptr = buffer_ptr as *const S as u32;
        let bytes_len = buffer_ptr.len() * size_of::<S>();
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
    /// Width of this sample type.
    const WIDTH: usize;

    /// Scale of this sample.
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

/// A 4-bytes aligned buffer. Needed for DMA access.
#[derive(Clone, Copy)]
#[repr(align(4))]
pub struct AlignedBuffer<T: Sample, const N: usize>([T; N]);

impl<T: Sample, const N: usize> AlignedBuffer<T, N> {
    /// Create a new `AlignedBuffer`.
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

/// Set of multiple buffers, for multi-buffering transfers.
pub struct MultiBuffering<S: Sample, const NB: usize, const NS: usize> {
    buffers: [AlignedBuffer<S, NS>; NB],
    index: usize,
}

impl<S: Sample, const NB: usize, const NS: usize> MultiBuffering<S, NB, NS> {
    /// Create a new `MultiBuffering`.
    pub fn new() -> Self {
        assert!(NB > 1);
        Self {
            buffers: [AlignedBuffer::<S, NS>::default(); NB],
            index: 0,
        }
    }

    fn get(&self) -> &[S] {
        &self.buffers[self.index]
    }

    fn get_mut(&mut self) -> &mut [S] {
        &mut self.buffers[self.index]
    }

    /// Advance to use the next buffer and return a non mutable pointer to the previous one.
    fn switch(&mut self) -> *const [S] {
        let prev_index = self.index;
        self.index = (self.index + 1) % NB;
        self.buffers[prev_index].deref() as *const [S]
    }

    /// Advance to use the next buffer and return a mutable pointer to the previous one.
    fn switch_mut(&mut self) -> *mut [S] {
        let prev_index = self.index;
        self.index = (self.index + 1) % NB;
        self.buffers[prev_index].deref_mut() as *mut [S]
    }
}

/// Peripheral static state
pub(crate) struct State {
    started: AtomicBool,
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
    stop_waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            stop_waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::i2s::I2s;
    fn state() -> &'static State;
}

/// I2S peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_i2s {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::i2s::SealedInstance for peripherals::$type {
            fn regs() -> pac::i2s::I2s {
                pac::$pac_type
            }
            fn state() -> &'static crate::i2s::State {
                static STATE: crate::i2s::State = crate::i2s::State::new();
                &STATE
            }
        }
        impl crate::i2s::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
