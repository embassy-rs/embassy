use core::future::Future;
use core::pin::Pin;

/// Wait for a pin to become high.
pub trait WaitForHigh {
    type Future<'a>: Future<Output = ()> + 'a;

    /// Wait for a pin to become high.
    ///
    /// If the pin is already high, the future completes immediately.
    /// Otherwise, it completes when it becomes high.
    fn wait_for_high<'a>(&'a mut self) -> Self::Future<'a>;
}

/// Wait for a pin to become low.
pub trait WaitForLow {
    type Future<'a>: Future<Output = ()> + 'a;

    /// Wait for a pin to become low.
    ///
    /// If the pin is already low, the future completes immediately.
    /// Otherwise, it completes when it becomes low.
    fn wait_for_low<'a>(&'a mut self) -> Self::Future<'a>;
}

/// Wait for a rising edge (transition from low to high)
pub trait WaitForRisingEdge {
    type Future<'a>: Future<Output = ()> + 'a;

    /// Wait for a rising edge (transition from low to high)
    fn wait_for_rising_edge<'a>(&'a mut self) -> Self::Future<'a>;
}

/// Wait for a falling edge (transition from high to low)
pub trait WaitForFallingEdge {
    type Future<'a>: Future<Output = ()> + 'a;

    /// Wait for a falling edge (transition from high to low)
    fn wait_for_falling_edge<'a>(&'a mut self) -> Self::Future<'a>;
}

/// Wait for any edge (any transition, high to low or low to high)
pub trait WaitForAnyEdge {
    type Future<'a>: Future<Output = ()> + 'a;

    /// Wait for any edge (any transition, high to low or low to high)
    fn wait_for_any_edge<'a>(&'a mut self) -> Self::Future<'a>;
}
