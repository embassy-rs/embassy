//! Timer (TIM) drivers.

#![macro_use]

use embassy_hal_internal::PeripheralType;
use mspm0_metapac::tim::Tim;

use crate::gpio::Pin;
use crate::interrupt;

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: interrupt::typelevel::Interrupt;

    #[inline]
    fn width() -> CounterWidth {
        Self::info().width
    }
}

/// A timer instance with 2 compare and capture channels.
pub trait General2ChannelInstance: Instance {}

/// A timer instance with 4 compare and capture channels.
pub trait General4ChannelInstance: General2ChannelInstance {}

/// A timer instance with a 32-bit counter.
pub trait General32BitInstance: Instance {}

/// An advanced timer instance with complementary channel outputs and fault detection.
pub trait AdvancedInstance: Instance {}

/// Marker trait describing the type used for counting.
#[allow(private_bounds)]
pub trait TimerBits: SealedTimerBits {}
impl TimerBits for u16 {}
impl TimerBits for u32 {}

/// Width of counter in a timer instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CounterWidth {
    /// 16-bit counter width.
    Bits16,

    /// 32-bit counter width.
    Bits32,
}

impl CounterWidth {
    /// Counter width in bits.
    pub const fn bits(&self) -> u8 {
        match self {
            CounterWidth::Bits16 => 16,
            CounterWidth::Bits32 => 32,
        }
    }
}

/// A timer pin.
#[allow(private_bounds)]
pub trait TimerPin<T: Instance, Channel: TimerChannel>: Pin + PeripheralType {
    /// Get the PF number needed to use this pin as a timer pin for the specified [`Channel`].
    fn pf_num(&self) -> u8;
}

/// A timer channel
#[allow(private_bounds)]
pub trait TimerChannel: SealedChannel {}

/// Marker type for channel 0.
pub enum Ch0 {}
impl TimerChannel for Ch0 {}

/// Marker type for channel 1.
pub enum Ch1 {}
impl TimerChannel for Ch1 {}

/// Marker type for channel 2.
pub enum Ch2 {}
impl TimerChannel for Ch2 {}

/// Marker type for channel 3.
pub enum Ch3 {}
impl TimerChannel for Ch3 {}

/// Marker type for channel 0 complementary output.
pub enum CompCh0 {}
impl TimerChannel for CompCh0 {}

/// Marker type for channel 1 complementary output.
pub enum CompCh1 {}
impl TimerChannel for CompCh1 {}

/// Marker type for channel 2 complementary output.
pub enum CompCh2 {}
impl TimerChannel for CompCh2 {}

/// Marker type for channel 3 complementary output.
pub enum CompCh3 {}
impl TimerChannel for CompCh3 {}

/// Clock source for the timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSel {
    /// Use the low frequency clock.
    ///
    /// The LFCLK runs at 32.768 kHz.
    LfClk,

    /// Use the middle frequency clock.
    ///
    /// The MFCLK runs at 4 MHz.
    MfClk,
    // TODO: BusClk
    // The actual clock speed used depends on power domain and system clock config.
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CountingMode {
    /// The timer counts up to the reload value and then resets back at 0.
    #[default]
    EdgeAlignedUp,

    /// The timer counts down to 0 and then resets back to the load value.
    EdgeAlignedDown,

    /// The timer counts up to the load value and then counts back to 0.
    CenterAligned,
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
}

trait SealedTimerBits: Into<u32> {}
impl SealedTimerBits for u16 {}
impl SealedTimerBits for u32 {}

trait SealedChannel {}
impl SealedChannel for Ch0 {}
impl SealedChannel for Ch1 {}
impl SealedChannel for Ch2 {}
impl SealedChannel for Ch3 {}
impl SealedChannel for CompCh0 {}
impl SealedChannel for CompCh1 {}
impl SealedChannel for CompCh2 {}
impl SealedChannel for CompCh3 {}

pub(crate) struct Info {
    pub(crate) regs: Tim,
    #[allow(unused)]
    pub(crate) prescaler: bool,
    pub(crate) width: CounterWidth,
    #[allow(unused)]
    pub(crate) channels: u8,
}

macro_rules! impl_tim_instance {
    (
        $instance: ident,
        prescaler: $prescaler: expr,
        width: $width: ident,
        channels: $channels: expr
    ) => {
        impl crate::tim::SealedInstance for crate::peripherals::$instance {
            #[inline]
            fn info() -> &'static crate::tim::Info {
                const INFO: crate::tim::Info = crate::tim::Info {
                    regs: crate::pac::$instance,
                    prescaler: $prescaler,
                    width: crate::tim::CounterWidth::$width,
                    channels: $channels,
                };

                &INFO
            }
        }

        impl crate::tim::Instance for crate::peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$instance;
        }
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_32bit {
    ($instance: ident) => {
        impl crate::tim::General32BitInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_2ch {
    ($instance: ident) => {
        impl crate::tim::General2ChannelInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_4ch {
    ($instance: ident) => {
        impl crate::tim::General4ChannelInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_advanced {
    ($instance: ident) => {
        impl crate::tim::AdvancedInstance for crate::peripherals::$instance {}
    };
}

macro_rules! impl_tim_pin {
    (
        $instance: ident,
        $pin: ident,
        $pf: expr,
        $channel: ident
    ) => {
        impl crate::tim::TimerPin<crate::peripherals::$instance, crate::tim::$channel> for crate::peripherals::$pin {
            #[inline]
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}
