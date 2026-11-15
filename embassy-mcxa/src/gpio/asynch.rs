//! Interrupt-driven GPIO, shared by MCXA and MCXN.
//!
//! Both families have one `GPIOn` block per port with per-pin `ICR` (IRQC/ISF)
//! and `ISFR` flags. Each port's pin events are delivered through one NVIC line:
//! `GPIOn` on MCXA, and `GPIOn0` on MCXN, where `ICR.IRQS` routes each pin to
//! `GPIOn0`/`ISFR0` or `GPIOn1`/`ISFR1` and this driver always selects 0.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::pin;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitMap;

use super::{AnyPin, Flex, GpioPin, Input, Mode, NoIrqBound, PinMux, Pull, SealedPin, sealed};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac::gpio::Irqc;

pub struct Async {}
impl sealed::Sealed for Async {}
impl Mode for Async {}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b as usize)
            }
        }
    }
}

#[cfg(feature = "_mcxa2xx")]
const PORT_COUNT: usize = 5;
#[cfg(any(feature = "_mcxa5xx", feature = "_mcxn"))]
const PORT_COUNT: usize = 6;

static PORT_WAIT_MAPS: [WaitMap<usize, ()>; PORT_COUNT] = [const { WaitMap::new() }; PORT_COUNT];

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: Interrupt;
}

pub(crate) struct Info {
    pub port_index: usize,
    pub gpio: crate::pac::gpio::Gpio,
}

pub trait PeriGpioExt<'d, T: HasGpioInstance> {
    /// Type erase the pin while also binding and Irq.
    ///
    /// This means the [`AnyPin`] can be used to construct an async [`Input`] with [`Flex::async_from_anypin`]
    /// and an async [`Flex`] with [`Flex::async_from_anypin`].
    fn degrade_async(
        self,
        _irq: impl crate::interrupt::typelevel::Binding<<T::Instance as Instance>::Interrupt, InterruptHandler<T::Instance>>,
    ) -> Peri<'d, AnyPin>;
}

impl<'d, T: HasGpioInstance> PeriGpioExt<'d, T> for Peri<'d, T> {
    /// Type erase the pin while also binding and Irq.
    ///
    /// This means the [`AnyPin`] can be used to construct an async [`Input`] with [`Flex::async_from_anypin`]
    /// and an async [`Flex`] with [`Flex::async_from_anypin`].
    fn degrade_async(
        self,
        _irq: impl crate::interrupt::typelevel::Binding<<T::Instance as Instance>::Interrupt, InterruptHandler<T::Instance>>,
    ) -> Peri<'d, AnyPin> {
        HasGpioInstance::degrade_async(self, _irq)
    }
}

pub trait HasGpioInstance: GpioPin {
    type Instance: Instance;

    /// Type erase the pin while also binding and Irq.
    ///
    /// This means the [`AnyPin`] can be used to construct an async [`Input`] with [`Flex::async_from_anypin`]
    /// and an async [`Flex`] with [`Flex::async_from_anypin`].
    fn degrade_async<'p>(
        this: Peri<'p, Self>,
        _irq: impl crate::interrupt::typelevel::Binding<
            <Self::Instance as Instance>::Interrupt,
            InterruptHandler<Self::Instance>,
        >,
    ) -> Peri<'p, AnyPin>;
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    const PERF_INT_INCR: fn();
}

/// Implement [`Instance`] for `GPIOn`, bound to the port's pin-event interrupt.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_gpio_instance {
    ($n:expr) => {
        paste::paste! {
            #[cfg(not(feature = "_mcxn"))]
            $crate::impl_gpio_instance!($n, [<GPIO $n>]);
            #[cfg(feature = "_mcxn")]
            $crate::impl_gpio_instance!($n, [<GPIO $n 0>]);
        }
    };
    ($n:expr, $irq:tt) => {
        paste::paste! {
            impl $crate::gpio::SealedInstance for $crate::peripherals::[<GPIO $n>] {
                fn info() -> &'static $crate::gpio::Info {
                    static INFO: $crate::gpio::Info =  $crate::gpio::Info {
                        gpio: $crate::pac::[<GPIO $n>],
                        port_index: $n,
                    };
                    &INFO
                }
                const PERF_INT_INCR: fn() = $crate::perf_counters::[<incr_interrupt_gpio $n _wake>];
            }

            impl $crate::gpio::Instance for $crate::peripherals::[<GPIO $n>] {
                type Interrupt = $crate::interrupt::typelevel::$irq;
            }
        }
    };
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let isfr = info.gpio.isfr(0);

        for pin in BitIter(isfr.read().0) {
            // Clear all pending interrupts
            isfr.write(|w| w.0 = 1 << pin);
            info.gpio.icr(pin).modify(|w| w.set_irqc(Irqc::Irqc0)); // Disable interrupt

            // Wake the corresponding port waker
            if let Some(w) = PORT_WAIT_MAPS.get(info.port_index) {
                T::PERF_INT_INCR();
                w.wake(&pin, ());
            }
        }
    }
}

/// Async methods
impl<'d> Flex<'d, Async> {
    /// Wrap the pin in Flex with Async support.
    ///
    /// This enables the use of async functions like: [`Flex::wait_for_high`] and [`Flex::wait_for_falling_edge`].
    pub fn new_async<P>(
        pin: Peri<'d, P>,
        _irq: impl crate::interrupt::typelevel::Binding<<P::Instance as Instance>::Interrupt, InterruptHandler<P::Instance>>,
    ) -> Self
    where
        P: GpioPin + HasGpioInstance,
    {
        pin.set_function(PinMux::Mux0);
        unsafe {
            <P::Instance as Instance>::Interrupt::enable();
        }
        Self {
            pin: pin.into(),
            _phantom: PhantomData,
        }
    }

    /// Wrap an [`AnyPin`] in Flex with Async support.
    ///
    /// This enables the use of async functions like: [`Input::wait_for_high`] and [`Input::wait_for_falling_edge`].
    /// In order to use an [`AnyPin`] with this function it needs to be constructed by
    /// calling [`PeriGpioExt::degrade_async`] on the pin to bind the Irq.
    /// If an [`AnyPin`] is provided that was not constructed with [`PeriGpioExt::degrade_async`],
    /// it will return the error: [`NoIrqBound`].
    pub fn async_from_anypin(pin: Peri<'d, AnyPin>) -> Result<Self, NoIrqBound> {
        pin.set_function(PinMux::Mux0);
        if pin.irq_bound() {
            Ok(Self {
                pin: pin.into(),
                _phantom: PhantomData,
            })
        } else {
            Err(NoIrqBound)
        }
    }

    /// Helper function that waits for a given interrupt trigger
    async fn wait_for_inner(&mut self, level: crate::pac::gpio::Irqc) {
        let pin_index = usize::from(self.pin.pin_index());

        // First, ensure that we have a waker that is ready for this port+pin
        let w = PORT_WAIT_MAPS[usize::from(self.pin.port_index())].wait(pin_index);
        let mut w = pin!(w);
        // Wait for the subscription to occur, which requires polling at least once
        //
        // This function returns a result, but can only be an Err if:
        //
        // * We call `.close()` on a WaitMap, which we never do
        // * We have a duplicate key, which can't happen because `wait_for_*` methods
        //   take an &mut ref of their unique port+pin combo
        //
        // So we wait for it to complete, but ignore the result.
        _ = w.as_mut().subscribe().await;

        // Now that our waker is in the map, we can enable the appropriate interrupt
        //
        // Clear any existing pending interrupt on this pin
        let gpio = self.pin.gpio();
        gpio.isfr(0).write(|w| w.0 = 1 << pin_index);
        gpio.icr(pin_index).write(|w| w.set_isf(true));

        // Pin interrupt configuration
        gpio.icr(pin_index).modify(|w| {
            // Route the pin to the GPIOn0 line and ISFR0 that the handler services.
            #[cfg(feature = "_mcxn")]
            w.set_irqs(false);
            w.set_irqc(level);
        });

        // Finally, we can await the matching call to `.wake()` from the interrupt.
        //
        // Again, technically, this could return a result, but for the same reasons
        // as above, this can't be an error in our case, so just wait for it to complete
        _ = w.await;
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::Irqc12)
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::Irqc8)
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::Irqc9)
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::Irqc10)
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::Irqc11)
    }
}

/// Async methods
impl<'d> Input<'d, Async> {
    /// Create a GPIO input driver for a [GpioPin] with async support.
    ///
    /// This enables the use of async functions like: [`Input::wait_for_high`] and [`Input::wait_for_falling_edge`].
    pub fn new_async<P>(
        pin: Peri<'d, P>,
        irq: impl crate::interrupt::typelevel::Binding<<P::Instance as Instance>::Interrupt, InterruptHandler<P::Instance>>
        + 'd,
        pull_select: Pull,
    ) -> Self
    where
        P: GpioPin + HasGpioInstance,
    {
        let mut flex = Flex::new_async(pin, irq);
        flex.set_as_input();
        flex.set_pull(pull_select);
        Self { flex }
    }

    /// Create a GPIO input driver for a [GpioPin] with async support from an [`AnyPin`].
    ///
    /// This enables the use of async functions like: [`Input::wait_for_high`] and [`Input::wait_for_falling_edge`].
    /// In order to use an [`AnyPin`] with this function it needs to be constructed by
    /// calling [`PeriGpioExt::degrade_async`] on the pin to bind the Irq.
    /// If an [`AnyPin`] is provided that was not constructed with [`PeriGpioExt::degrade_async`],
    /// it will return the error: [`NoIrqBound`].
    pub fn async_from_anypin(pin: Peri<'d, AnyPin>, pull_select: Pull) -> Result<Self, NoIrqBound> {
        let mut flex = Flex::async_from_anypin(pin)?;
        flex.set_as_input();
        flex.set_pull(pull_select);
        Ok(Self { flex })
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_high()
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_low()
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_rising_edge()
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_falling_edge()
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_any_edge()
    }
}

impl embedded_hal_async::digital::Wait for Input<'_, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

impl embedded_hal_async::digital::Wait for Flex<'_, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}
