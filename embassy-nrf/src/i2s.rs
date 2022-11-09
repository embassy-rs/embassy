#![macro_use]

//! I2S

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use pac::i2s::config::mcken;

use crate::{pac, Peripheral};
use crate::interrupt::{Interrupt, InterruptExt};
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::gpio::sealed::Pin as _;

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

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub ratio: Ratio,
    pub sample_width: SampleWidth,
    pub align: Align,
    pub format: Format,
    pub channels: Channels,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ratio: Ratio::_32x,
            sample_width: SampleWidth::_16bit,
            align: Align::Left,
            format: Format::I2S,
            channels: Channels::Stereo,
        }
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

/// I2S Mode
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Controller,
    Peripheral,
}

// /// Master clock generator frequency.
// #[derive(Debug, Eq, PartialEq, Clone, Copy)]
// pub enum MckFreq {
//     _32MDiv8 = 0x20000000,
//     _32MDiv10 = 0x18000000,
//     _32MDiv11 = 0x16000000,
//     _32MDiv15 = 0x11000000,
//     _32MDiv16 = 0x10000000,
//     _32MDiv21 = 0x0C000000,
//     _32MDiv23 = 0x0B000000,
//     _32MDiv30 = 0x08800000,
//     _32MDiv31 = 0x08400000,
//     _32MDiv32 = 0x08000000,
//     _32MDiv42 = 0x06000000,
//     _32MDiv63 = 0x04100000,
//     _32MDiv125 = 0x020C0000,
// }


/// Interface to the UARTE peripheral using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct I2s<'d, T: Instance> {
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

impl<'d, T: Instance> I2s<'d, T> {
    /// Create a new I2S
    pub fn new(
        i2s: impl Peripheral<P = T> + 'd,
        // irq: impl Peripheral<P = T::Interrupt> + 'd,
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
            // irq,
            mck.map_into(), sck.map_into(), lrck.map_into(), sdin.map_into(), sdout.map_into(), config)
    }

    fn new_inner(
        i2s: impl Peripheral<P = T> + 'd,
        // irq: impl Peripheral<P = T::Interrupt> + 'd,
        mck: PeripheralRef<'d, AnyPin>,
        sck: PeripheralRef<'d, AnyPin>,
        lrck: PeripheralRef<'d, AnyPin>,
        sdin: PeripheralRef<'d, AnyPin>,
        sdout: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(
            i2s,
            // irq,
            mck, sck, lrck, sdin, sdout);

        let r = T::regs();

        // TODO get configuration rather than hardcoding ratio, swidth, align, format, channels

        r.config.mcken.write(|w| w.mcken().enabled());
        r.config.mckfreq.write(|w| w.mckfreq()._32mdiv16());
        r.config.ratio.write(|w| w.ratio()._192x());
        r.config.mode.write(|w| w.mode().master());
        r.config.swidth.write(|w| w.swidth()._16bit());
        r.config.align.write(|w| w.align().left());
        r.config.format.write(|w| w.format().i2s());
        r.config.channels.write(|w| w.channels().stereo());

        r.psel.mck.write(|w| {
            unsafe { w.bits(mck.psel_bits()) };
            w.connect().connected()
        });

        r.psel.sck.write(|w| {
            unsafe { w.bits(sck.psel_bits()) };
            w.connect().connected()
        });

        r.psel.lrck.write(|w| {
            unsafe { w.bits(lrck.psel_bits()) };
            w.connect().connected()
        });

        r.psel.sdin.write(|w| {
            unsafe { w.bits(sdin.psel_bits()) };
            w.connect().connected()
        });

        r.psel.sdout.write(|w| {
            unsafe { w.bits(sdout.psel_bits()) };
            w.connect().connected()
        });

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
    pub fn stop(&self) -> &Self {
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

    /// Transmits the given `tx_buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    // TODO Define a better interface for the input buffer
    #[allow(unused_mut)]
    pub async fn tx(&mut self, ptr: *const u8, len: usize) -> Result<(), Error> {
        self.output.tx(ptr, len).await
    }
}

impl<'d, T: Instance> I2sOutput<'d, T> {
    /// Transmits the given `tx_buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    // TODO Define a better interface for the input buffer
    pub async fn tx(&mut self, ptr: *const u8, len: usize) -> Result<(), Error> {
        if ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        let maxcnt = (len / (core::mem::size_of::<u32>() / core::mem::size_of::<u8>())) as u32;
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        let r = T::regs();
        let _s = T::state();

        // TODO we can not progress until the last buffer written in TXD.PTR
        // has started the transmission.
        // We can use some sync primitive from `embassy-sync`.

        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });

        Ok(())
    }
}

pub(crate) mod sealed {
    use core::sync::atomic::AtomicU8;

    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub input_waker: AtomicWaker,
        pub output_waker: AtomicWaker,
        pub buffers_refcount: AtomicU8,
    }
    impl State {
        pub const fn new() -> Self {
            Self {
                input_waker: AtomicWaker::new(),
                output_waker: AtomicWaker::new(),
                buffers_refcount: AtomicU8::new(0),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::i2s::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    type Interrupt: Interrupt;
}

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

