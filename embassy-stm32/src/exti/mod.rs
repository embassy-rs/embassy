//! External Interrupts (EXTI)
use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use futures_util::FutureExt;

use crate::gpio::{AnyPin, ExtiPin, Input, Level, Pin as GpioPin, PinNumber, Pull};
use crate::interrupt::Interrupt as InterruptEnum;
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as InterruptType};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::EXTI;
use crate::{Peri, pac};

#[macro_use]
mod low_level;
pub use low_level::{InterruptState, TriggerEdge};

pub mod blocking;

const EXTI_COUNT: usize = 16;
static EXTI_WAKERS: [AtomicWaker; EXTI_COUNT] = [const { AtomicWaker::new() }; EXTI_COUNT];

#[cfg(all(exti_w, feature = "_core-cm0p"))]
fn cpu_regs() -> pac::exti::Cpu {
    EXTI.cpu(1)
}

#[cfg(all(exti_w, not(feature = "_core-cm0p")))]
fn cpu_regs() -> pac::exti::Cpu {
    EXTI.cpu(0)
}

#[cfg(not(exti_w))]
fn cpu_regs() -> pac::exti::Exti {
    EXTI
}

#[cfg(not(any(
    exti_c0, exti_g0, exti_u0, exti_l5, gpio_v1, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6
)))]
fn exticr_regs() -> pac::syscfg::Syscfg {
    pac::SYSCFG
}
#[cfg(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6))]
fn exticr_regs() -> pac::exti::Exti {
    EXTI
}
#[cfg(gpio_v1)]
fn exticr_regs() -> pac::afio::Afio {
    pac::AFIO
}

unsafe fn on_irq() {
    cfg_no_rpr_fpr! {
        let bits = EXTI.pr(0).read().0;
    }
    cfg_has_rpr_fpr! {
        let bits = EXTI.rpr(0).read().0 | EXTI.fpr(0).read().0;
    }

    // We don't handle or change any EXTI lines above 16.
    let bits = bits & 0x0000FFFF;

    // Mask all the channels that fired.
    cpu_regs().imr(0).modify(|w| w.0 &= !bits);

    // Wake the tasks
    for pin in BitIter(bits) {
        EXTI_WAKERS[pin as usize].wake();
    }

    // Clear pending
    low_level::clear_exti_pending_mask(bits);

    #[cfg(feature = "low-power")]
    crate::low_power::Executor::on_wakeup_irq_or_event();
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

/// EXTI input driver.
///
/// This driver augments a GPIO `Input` with EXTI functionality. EXTI is not
/// built into `Input` itself because it needs to take ownership of the corresponding
/// EXTI channel, which is a limited resource.
///
/// Pins PA5, PB5, PC5... all use EXTI channel 5, so you can't use EXTI on, say, PA5 and PC5 at the same time.
pub struct ExtiInput<'d, Mode: PeriMode> {
    pin: Input<'d>,
    _kind: PhantomData<Mode>,
}

impl<'d, Mode: PeriMode> Unpin for ExtiInput<'d, Mode> {}

impl<'d, Mode: PeriMode> ExtiInput<'d, Mode> {
    /// Get whether the pin is high.
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin is low.
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }
}

impl<'d> ExtiInput<'d, Async> {
    /// Create an EXTI input.
    ///
    /// The Binding must bind the Channel's IRQ to [InterruptHandler].
    pub fn new<T: ExtiPin + GpioPin>(
        pin: Peri<'d, T>,
        _ch: Peri<'d, T::ExtiChannel>,
        pull: Pull,
        _irq: impl Binding<
            <<T as ExtiPin>::ExtiChannel as Channel>::IRQ,
            InterruptHandler<<<T as ExtiPin>::ExtiChannel as Channel>::IRQ>,
        >,
    ) -> Self {
        Self {
            pin: Input::new(pin, pull),
            _kind: PhantomData,
        }
    }

    /// Asynchronously wait until the pin is high.
    ///
    /// This returns immediately if the pin is already high.
    pub async fn wait_for_high(&mut self) {
        let fut = ExtiInputFuture::new(&self.pin, TriggerEdge::Rising, true);
        if self.is_high() {
            return;
        }
        fut.await
    }

    /// Asynchronously wait until the pin is low.
    ///
    /// This returns immediately if the pin is already low.
    pub async fn wait_for_low(&mut self) {
        let fut = ExtiInputFuture::new(&self.pin, TriggerEdge::Falling, true);
        if self.is_low() {
            return;
        }
        fut.await
    }

    /// Asynchronously wait until the pin sees a rising edge.
    ///
    /// If the pin is already high, it will wait for it to go low then back high.
    pub async fn wait_for_rising_edge(&mut self) {
        ExtiInputFuture::new(&self.pin, TriggerEdge::Rising, true).await
    }

    /// Asynchronously wait until the pin sees a rising edge.
    ///
    /// If the pin is already high, it will wait for it to go low then back high.
    pub fn poll_for_rising_edge<'a>(&mut self, cx: &mut Context<'a>) {
        let _ = ExtiInputFuture::new(&self.pin, TriggerEdge::Rising, false).poll_unpin(cx);
    }

    /// Asynchronously wait until the pin sees a falling edge.
    ///
    /// If the pin is already low, it will wait for it to go high then back low.
    pub async fn wait_for_falling_edge(&mut self) {
        ExtiInputFuture::new(&self.pin, TriggerEdge::Falling, true).await
    }

    /// Asynchronously wait until the pin sees a falling edge.
    ///
    /// If the pin is already low, it will wait for it to go high then back low.
    pub fn poll_for_falling_edge<'a>(&mut self, cx: &mut Context<'a>) {
        let _ = ExtiInputFuture::new(&self.pin, TriggerEdge::Falling, false).poll_unpin(cx);
    }

    /// Asynchronously wait until the pin sees any edge (either rising or falling).
    pub async fn wait_for_any_edge(&mut self) {
        ExtiInputFuture::new(&self.pin, TriggerEdge::Any, true).await
    }

    /// Asynchronously wait until the pin sees any edge (either rising or falling).
    pub fn poll_for_any_edge<'a>(&mut self, cx: &mut Context<'a>) {
        let _ = ExtiInputFuture::new(&self.pin, TriggerEdge::Any, false).poll_unpin(cx);
    }
}

impl<'d> ExtiInput<'d, Blocking> {
    /// Creates a new EXTI input for use with manual interrupt handling.
    ///
    /// This configures and enables the EXTI interrupt for the pin, but does not
    /// provide async methods for waiting on events. You must provide your own
    /// interrupt handler to service EXTI events and clear pending flags.
    ///
    /// For async/await integration with Embassy's executor, use `ExtiInput<Async>` instead.
    ///
    /// # Arguments
    /// * `pin` - The GPIO pin to use
    /// * `ch` - The EXTI channel corresponding to the pin (consumed for ownership tracking)
    /// * `pull` - The pull configuration for the pin
    /// * `trigger_edge` - The edge triggering mode (falling, rising, or any)
    ///
    /// # Returns
    /// A new `ExtiInput` instance with interrupts enabled
    pub fn new_blocking<T: GpioPin + ExtiPin>(
        pin: Peri<'d, T>,
        _ch: Peri<'d, T::ExtiChannel>, // Consumed for ownership tracking
        pull: Pull,
        trigger_edge: TriggerEdge,
    ) -> Self {
        let pin = Input::new(pin, pull);

        low_level::configure_and_enable_exti(&pin, trigger_edge);

        Self {
            pin,
            _kind: PhantomData,
        }
    }

    /// Reconfigures the edge detection mode for this pin's EXTI line
    ///
    /// This method updates which edges (rising, falling, or any) will trigger
    /// interrupts for this pin.
    /// Note that reconfiguring the edge detection will clear any pending
    /// interrupt flag for this pin.
    pub fn set_edge_detection(&mut self, trigger_edge: TriggerEdge) {
        let pin_num = self.pin.pin.pin.pin();
        let port_num = self.pin.pin.pin.port();
        low_level::configure_exti_pin(pin_num, port_num, trigger_edge);
    }

    /// Enables the EXTI interrupt for this pin
    pub fn enable_interrupt(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::set_exti_interrupt_enabled(pin_num, InterruptState::Enabled);
    }

    /// Disables the EXTI interrupt for this pin
    pub fn disable_interrupt(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::set_exti_interrupt_enabled(pin_num, InterruptState::Disabled);
    }

    /// Clears any pending interrupt for this pin
    ///
    /// This method clears the pending interrupt flag for the EXTI line
    /// associated with this pin. This should typically be called from
    /// the interrupt handler after processing an interrupt.
    pub fn clear_pending(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::clear_exti_pending(pin_num);
    }

    /// Checks if an interrupt is pending for the current pin
    ///
    /// This method checks if there is a pending interrupt on the EXTI line
    /// associated with this pin.
    ///
    /// # Returns
    /// `true` if an interrupt is pending, `false` otherwise
    pub fn is_pending(&self) -> bool {
        let pin_num = self.pin.pin.pin.pin();
        low_level::is_exti_pending(pin_num)
    }

    fn pin_mask(&self) -> u32 {
        1 << self.pin.pin.pin.pin()
    }
}

impl<'d, Mode: PeriMode> embedded_hal_02::digital::v2::InputPin for ExtiInput<'d, Mode> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<'d, Mode: PeriMode> embedded_hal_1::digital::ErrorType for ExtiInput<'d, Mode> {
    type Error = Infallible;
}

impl<'d, Mode: PeriMode> embedded_hal_1::digital::InputPin for ExtiInput<'d, Mode> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for ExtiInput<'d, Async> {
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

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct ExtiInputFuture<'a> {
    pin: PinNumber,
    drop: bool,
    phantom: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    fn new(pin: &Input, trigger_edge: TriggerEdge, drop: bool) -> Self {
        low_level::configure_and_enable_exti(pin, trigger_edge);

        Self {
            pin: pin.pin.pin.pin(),
            drop,
            phantom: PhantomData,
        }
    }
}

impl<'a> Drop for ExtiInputFuture<'a> {
    fn drop(&mut self) {
        if self.drop {
            critical_section::with(|_| {
                let pin = self.pin as _;
                cpu_regs().imr(0).modify(|w| w.set_line(pin, false));
            });
        }
    }
}

impl<'a> Future for ExtiInputFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        EXTI_WAKERS[self.pin as usize].register(cx.waker());

        let imr = cpu_regs().imr(0).read();
        if !imr.line(self.pin as _) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

macro_rules! foreach_exti_irq {
    ($action:ident) => {
        foreach_interrupt!(
            (EXTI0)  => { $action!(EXTI0); };
            (EXTI1)  => { $action!(EXTI1); };
            (EXTI2)  => { $action!(EXTI2); };
            (EXTI3)  => { $action!(EXTI3); };
            (EXTI4)  => { $action!(EXTI4); };
            (EXTI5)  => { $action!(EXTI5); };
            (EXTI6)  => { $action!(EXTI6); };
            (EXTI7)  => { $action!(EXTI7); };
            (EXTI8)  => { $action!(EXTI8); };
            (EXTI9)  => { $action!(EXTI9); };
            (EXTI10) => { $action!(EXTI10); };
            (EXTI11) => { $action!(EXTI11); };
            (EXTI12) => { $action!(EXTI12); };
            (EXTI13) => { $action!(EXTI13); };
            (EXTI14) => { $action!(EXTI14); };
            (EXTI15) => { $action!(EXTI15); };

            // plus the weird ones
            (EXTI0_1)   => { $action!(EXTI0_1); };
            (EXTI15_10) => { $action!(EXTI15_10); };
            (EXTI15_4)  => { $action!(EXTI15_4); };
            (EXTI1_0)   => { $action!(EXTI1_0); };
            (EXTI2_3)   => { $action!(EXTI2_3); };
            (EXTI2_TSC) => { $action!(EXTI2_TSC); };
            (EXTI3_2)   => { $action!(EXTI3_2); };
            (EXTI4_15)  => { $action!(EXTI4_15); };
            (EXTI9_5)   => { $action!(EXTI9_5); };
        );
    };
}

///EXTI interrupt handler. All EXTI interrupt vectors should be bound to this handler.
///
/// It is generic over the [Interrupt](InterruptType) rather
/// than the [Channel] because it should not be bound multiple
/// times to the same vector on chips which multiplex multiple EXTI interrupts into one vector.
//
// It technically doesn't need to be generic at all, except to satisfy the generic argument
// of [Handler]. All EXTI interrupts eventually land in the same on_irq() function.
pub struct InterruptHandler<T: crate::interrupt::typelevel::Interrupt> {
    _phantom: PhantomData<T>,
}

impl<T: InterruptType> Handler<T> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_irq()
    }
}

trait SealedChannel {}

/// EXTI channel trait.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Sized {
    /// EXTI channel number.
    fn number(&self) -> PinNumber;
    /// [Enum-level Interrupt](InterruptEnum), which may be the same for multiple channels.
    fn irq(&self) -> InterruptEnum;
    /// [Type-level Interrupt](InterruptType), which may be the same for multiple channels.
    type IRQ: InterruptType;
}

//Doc isn't hidden in order to surface the explanation to users, even though it's completely inoperable, not just deprecated.
//Entire type along with doc can probably be removed after deprecation has appeared in a release once.
/// Deprecated type-erased EXTI channel.
///
/// Support for AnyChannel was removed in order to support manually bindable EXTI interrupts via bind_interrupts; [ExtiInput::new()]
/// must know the required IRQ at compile time, and therefore cannot support type-erased channels.
#[deprecated = "type-erased EXTI channels are no longer supported, in order to support manually bindable EXTI interrupts (more info: https://github.com/embassy-rs/embassy/pull/4922)"]
pub struct AnyChannel {
    #[allow(unused)]
    number: PinNumber,
}

macro_rules! impl_exti {
    ($type:ident, $number:expr) => {
        impl SealedChannel for crate::peripherals::$type {}
        impl Channel for crate::peripherals::$type {
            fn number(&self) -> PinNumber {
                $number
            }
            fn irq(&self) -> InterruptEnum {
                crate::_generated::peripheral_interrupts::EXTI::$type::IRQ
            }
            type IRQ = crate::_generated::peripheral_interrupts::EXTI::$type;
        }

        //Still here to surface deprecation messages to the user - remove when removing AnyChannel
        #[allow(deprecated)]
        impl From<crate::peripherals::$type> for AnyChannel {
            fn from(_val: crate::peripherals::$type) -> Self {
                Self { number: $number }
            }
        }
    };
}

impl_exti!(EXTI0, 0);
impl_exti!(EXTI1, 1);
impl_exti!(EXTI2, 2);
impl_exti!(EXTI3, 3);
impl_exti!(EXTI4, 4);
impl_exti!(EXTI5, 5);
impl_exti!(EXTI6, 6);
impl_exti!(EXTI7, 7);
impl_exti!(EXTI8, 8);
impl_exti!(EXTI9, 9);
impl_exti!(EXTI10, 10);
impl_exti!(EXTI11, 11);
impl_exti!(EXTI12, 12);
impl_exti!(EXTI13, 13);
impl_exti!(EXTI14, 14);
impl_exti!(EXTI15, 15);

macro_rules! enable_irq {
    ($e:ident) => {
        crate::interrupt::typelevel::$e::enable();
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(_cs: critical_section::CriticalSection) {
    use crate::interrupt::typelevel::Interrupt;

    foreach_exti_irq!(enable_irq);
}
