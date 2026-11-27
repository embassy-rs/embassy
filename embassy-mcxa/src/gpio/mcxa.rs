//! MCXA pin control and asynchronous GPIO capabilities.

use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_nxp_drivers::gpio::Trigger;

use super::{
    DriveStrength, Flex, Info, Input, Inverter, Mode, NoIrqBound, OpenDrain, Pull, SealedInstance, SlewRate, State,
    sealed,
};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac::common::{RW, Reg};
use crate::pac::port::{Mux, Pcr};

pub struct Async {}
impl sealed::Sealed for Async {}
impl Mode for Async {}

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: Interrupt;
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

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // SAFETY: this is the handler of the port the info and state belong to.
        let served = unsafe { embassy_nxp_drivers::gpio::on_interrupt(T::info(), T::state()) };
        for _ in 0..served.count_ones() {
            T::PERF_INT_INCR();
        }
    }
}

impl From<OpenDrain> for bool {
    fn from(open_drain: OpenDrain) -> bool {
        match open_drain {
            OpenDrain::No => false,
            OpenDrain::Yes => true,
        }
    }
}

impl From<Pull> for (bool, bool) {
    fn from(pull: Pull) -> Self {
        match pull {
            Pull::Disabled => (false, false),
            Pull::Up => (true, true),
            Pull::Down => (true, false),
        }
    }
}

impl From<SlewRate> for bool {
    fn from(slew_rate: SlewRate) -> bool {
        match slew_rate {
            SlewRate::Fast => false,
            SlewRate::Slow => true,
        }
    }
}

impl From<DriveStrength> for bool {
    fn from(strength: DriveStrength) -> bool {
        match strength {
            DriveStrength::Normal => false,
            DriveStrength::Double => true,
        }
    }
}

impl From<Inverter> for bool {
    fn from(strength: Inverter) -> bool {
        match strength {
            Inverter::Disabled => false,
            Inverter::Enabled => true,
        }
    }
}
/// Type-erased representation of a GPIO pin.
pub struct AnyPin {
    port: u8,
    pin: u8,
    info: &'static Info,
    state: &'static State,

    port_reg: crate::pac::port::Port,

    pcr_reg: Reg<Pcr, RW>,

    irq_bound: bool,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    pub(crate) fn new(
        port: u8,
        pin: u8,
        info: &'static Info,
        state: &'static State,
        port_reg: crate::pac::port::Port,
        pcr_reg: Reg<Pcr, RW>,
        irq_bound: bool,
    ) -> Self {
        Self {
            port,
            pin,
            info,
            state,

            port_reg,

            pcr_reg,

            irq_bound,
        }
    }

    #[inline(always)]
    pub(super) fn gpio_info(&self) -> &'static Info {
        self.info
    }

    #[inline(always)]
    pub(super) fn gpio_state(&self) -> &'static State {
        self.state
    }

    #[inline(always)]
    pub fn port_index(&self) -> u8 {
        self.port
    }

    #[inline(always)]
    pub fn pin_index(&self) -> u8 {
        self.pin
    }

    #[inline(always)]

    fn port_reg(&self) -> crate::pac::port::Port {
        self.port_reg
    }

    #[inline(always)]

    fn pcr_reg(&self) -> Reg<Pcr, RW> {
        self.pcr_reg
    }

    #[inline(always)]

    fn irq_bound(&self) -> bool {
        self.irq_bound
    }
}

embassy_hal_internal::impl_peripheral!(AnyPin);

pub(crate) trait SealedPin {
    fn port(&self) -> u8;

    fn pin(&self) -> u8;

    fn info(&self) -> &'static Info;

    fn state(&self) -> &'static State;

    #[inline(always)]
    fn gpio(&self) -> crate::pac::gpio::Gpio {
        self.info().regs
    }

    fn port_reg(&self) -> crate::pac::port::Port;

    fn pcr_reg(&self) -> Reg<Pcr, RW>;

    fn set_function(&self, function: Mux);

    fn set_pull(&self, pull: Pull);

    fn set_drive_strength(&self, strength: bool);

    fn set_slew_rate(&self, slew_rate: bool);

    fn set_enable_input_buffer(&self, buffer_enabled: bool);

    fn set_as_disabled(&self);

    fn set_input_enabled(&self, input_enabled: bool);
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait GpioPin: SealedPin + Sized + PeripheralType + Into<AnyPin> + 'static {
    /// Type-erase the pin.
    fn degrade(self) -> AnyPin {
        // SAFETY: This is only called within the GpioPin trait, which is only
        // implemented within this module on valid pin peripherals and thus
        // has been verified to be correct.
        AnyPin::new(
            self.port(),
            self.pin(),
            self.info(),
            self.state(),
            self.port_reg(),
            self.pcr_reg(),
            false,
        )
    }
}

impl SealedPin for AnyPin {
    #[inline(always)]
    fn pin(&self) -> u8 {
        self.pin_index()
    }

    #[inline(always)]
    fn port(&self) -> u8 {
        self.port_index()
    }

    #[inline(always)]
    fn info(&self) -> &'static Info {
        self.info
    }

    #[inline(always)]
    fn state(&self) -> &'static State {
        self.state
    }

    #[inline(always)]

    fn port_reg(&self) -> crate::pac::port::Port {
        self.port_reg()
    }

    #[inline(always)]

    fn pcr_reg(&self) -> Reg<Pcr, RW> {
        self.pcr_reg()
    }

    #[inline(always)]

    fn set_function(&self, function: Mux) {
        self.pcr_reg().modify(|w| w.set_mux(function));
    }

    #[inline(always)]

    fn set_pull(&self, pull: Pull) {
        let (pull_enable, pull_select) = pull.into();
        self.pcr_reg().modify(|w| {
            w.set_pe(pull_enable);
            w.set_ps(pull_select)
        });
    }

    #[inline(always)]
    fn set_drive_strength(&self, strength: bool) {
        self.pcr_reg().modify(|w| w.set_dse(strength));
    }

    #[inline(always)]
    fn set_slew_rate(&self, slew_rate: bool) {
        self.pcr_reg().modify(|w| w.set_sre(slew_rate));
    }

    #[inline(always)]

    fn set_enable_input_buffer(&self, buffer_enabled: bool) {
        self.pcr_reg().modify(|w| w.set_ibe(buffer_enabled));
    }

    #[inline(always)]

    fn set_as_disabled(&self) {
        // Set GPIO direction as input
        self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, false));
        // Set input buffer as disabled
        self.set_enable_input_buffer(false);
        // Set mode as GPIO (vs other potential functions)
        self.set_function(Mux::Mux0);
        // Set pin as disabled
        self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, true));
    }

    #[inline(always)]

    fn set_input_enabled(&self, input_enabled: bool) {
        // if `input_enabled` is true then we want Pid0 since this means "Configured for general-purpose input."
        // if `input_enabled` is false then we want Pid1 since this means "Disabled for general-purpose input".
        self.gpio()
            .pidr()
            .modify(|w| w.set_pid(self.pin() as usize, !input_enabled))
    }
}

impl GpioPin for AnyPin {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_gpio_pin {
    ($peri:ident, $port:expr, $pin:expr, $block:ident) => {
        ::pastey::paste! {

            impl $crate::gpio::SealedPin for $crate::peripherals::$peri {
                #[inline(always)]
                fn port(&self) -> u8 {
                    $port
                }

                #[inline(always)]
                fn pin(&self) -> u8 {
                    $pin
                }

                #[inline(always)]
                fn info(&self) -> &'static $crate::gpio::Info {
                    <$crate::peripherals::$block as $crate::gpio::SealedInstance>::info()
                }

                #[inline(always)]
                fn state(&self) -> &'static $crate::gpio::State {
                    <$crate::peripherals::$block as $crate::gpio::SealedInstance>::state()
                }

                #[inline(always)]
                fn port_reg(&self) -> $crate::pac::port::Port {
                    $crate::pac::[<PORT $port>]
                }

                #[inline(always)]
                fn pcr_reg(&self) -> $crate::pac::common::Reg<$crate::pac::port::Pcr, $crate::pac::common::RW> {
                    self.port_reg().pcr($pin)
                }

                #[inline(always)]
                fn set_function(&self, function: $crate::pac::port::Mux) {
                    self.pcr_reg().modify(|w| w.set_mux(function));
                }

                #[inline(always)]
                fn set_pull(&self, pull: $crate::gpio::Pull) {
                    let (pull_enable, pull_select) = pull.into();
                    self.pcr_reg().modify(|w| {
                        w.set_pe(pull_enable);
                        w.set_ps(pull_select);
                    });
                }

                #[inline(always)]
                fn set_drive_strength(&self, strength: bool) {
                    self.pcr_reg().modify(|w| w.set_dse(strength));
                }

                #[inline(always)]
                fn set_slew_rate(&self, slew_rate: bool) {
                    self.pcr_reg().modify(|w| w.set_sre(slew_rate));
                }

                #[inline(always)]
                fn set_enable_input_buffer(&self, buffer_enabled: bool) {
                    self.pcr_reg().modify(|w| w.set_ibe(buffer_enabled));
                }

                #[inline(always)]
                fn set_as_disabled(&self) {
                    // Set GPIO direction as input
                    self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, false));
                    // Set input buffer as disabled
                    self.set_enable_input_buffer(false);
                    // Set mode as GPIO (vs other potential functions)
                    self.set_function($crate::pac::port::Mux::Mux0);
                    // Set pin as disabled
                    self.set_input_enabled(false);
                }

                #[inline(always)]
                fn set_input_enabled(&self, input_enabled: bool) {
                    // if `input_enabled` is true then we want Pid0 since this means "Configured for general-purpose input."
                    // if `input_enabled` is false then we want Pid1 since this means "Disabled for general-purpose input".
                    self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, !input_enabled))
                }
            }

            impl $crate::gpio::GpioPin for $crate::peripherals::$peri {}

            impl From<$crate::peripherals::$peri> for $crate::gpio::AnyPin {
                fn from(value: $crate::peripherals::$peri) -> Self {
                    value.degrade()
                }
            }

            impl $crate::peripherals::$peri {
                /// Convenience helper to obtain a type-erased handle to this pin.
                pub fn degrade(&self) -> $crate::gpio::AnyPin {
                    use $crate::gpio::SealedPin;

                    $crate::gpio::AnyPin::new(
                        self.port(),
                        self.pin(),
                        self.info(),
                        self.state(),

                        self.port_reg(),

                        self.pcr_reg(),

                        false,
                    )
                }
            }

            impl $crate::gpio::HasGpioInstance for $crate::peripherals::$peri {
                type Instance = $crate::peripherals::$block;
                fn degrade_async<'p>(
                    this: embassy_hal_internal::Peri<'p, Self>,
                    _irq: impl $crate::interrupt::typelevel::Binding<
                        <Self::Instance as $crate::gpio::Instance>::Interrupt,
                        $crate::gpio::InterruptHandler<Self::Instance>,
                    >,
                ) -> embassy_hal_internal::Peri<'p, $crate::gpio::AnyPin> {
                    use $crate::interrupt::typelevel::Interrupt;
                    use $crate::gpio::SealedPin;
                    unsafe {
                        <<Self as $crate::gpio::HasGpioInstance>::Instance as $crate::gpio::Instance>::Interrupt::enable();
                    }
                    unsafe {
                        embassy_hal_internal::Peri::new_unchecked($crate::gpio::AnyPin::new(
                            this.port(),
                            this.pin(),
                            this.info(),
                            this.state(),
                            this.port_reg(),
                            this.pcr_reg(),
                            true,
                        ))
                    }
                }
            }
        }
    };
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains unmodified. The initial output level is unspecified, but
    /// can be changed before the pin is put into output mode.
    pub fn new(pin: Peri<'d, impl GpioPin>) -> Self {
        pin.set_function(Mux::Mux0);
        Self::from_pin(pin.into())
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        self.set_enable_input_buffer(true);
        // Clears PIDR to enable pin input. set_as_disabled() sets PID=1
        // which permanently disables input reads (always returns 0).
        self.line.set_direction(false);
    }

    /// Put the pin into output mode.
    pub fn set_as_output(&mut self) {
        self.set_pull(Pull::Disabled);

        // Clears PIDR in case this pin was previously disabled.
        self.line.set_direction(true);
    }

    /// Configure open-drain output.
    #[inline]
    pub fn set_open_drain(&mut self, open_drain: OpenDrain) {
        self.pin.pcr_reg().modify(|w| w.set_ode(open_drain.into()));
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_input_inversion(&mut self, invert: Inverter) {
        self.pin.pcr_reg().modify(|w| w.set_inv(invert.into()));
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
        pin.set_function(Mux::Mux0);
        unsafe {
            <P::Instance as Instance>::Interrupt::enable();
        }
        Self::from_pin(pin.into())
    }

    /// Wrap an [`AnyPin`] in Flex with Async support.
    ///
    /// This enables the use of async functions like: [`Input::wait_for_high`] and [`Input::wait_for_falling_edge`].
    /// In order to use an [`AnyPin`] with this function it needs to be constructed by
    /// calling [`PeriGpioExt::degrade_async`] on the pin to bind the Irq.
    /// If an [`AnyPin`] is provided that was not constructed with [`PeriGpioExt::degrade_async`],
    /// it will return the error: [`NoIrqBound`].
    pub fn async_from_anypin(pin: Peri<'d, AnyPin>) -> Result<Self, NoIrqBound> {
        pin.set_function(Mux::Mux0);
        if pin.irq_bound() {
            Ok(Self::from_pin(pin))
        } else {
            Err(NoIrqBound)
        }
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.line.wait_for(Trigger::High)
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.line.wait_for(Trigger::Low)
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.line.wait_for(Trigger::RisingEdge)
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.line.wait_for(Trigger::FallingEdge)
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.line.wait_for(Trigger::AnyEdge)
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
