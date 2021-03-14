use core::future::Future;
use core::pin::Pin;
use embedded_hal::Direction;

// Wait for a specified number of rotations either up or down
pub trait WaitForRotate {
    type RotateFuture<'a>: Future<Output = Direction> + 'a;

    /// Wait for a specified number of rotations, in ticks, either up or down.
    ///
    /// Return Direction::Upcounting if the high bound is reached.
    /// Return Direction::Downcounting if the low bound is reached.
    ///
    /// Number of ticks is encoder dependent. As an example, if we connect
    /// the Bourns PEC11H-4120F-S0020, we have 20 ticks per full rotation.
    /// Other encoders may vary.
    fn wait_for_rotate<'a>(
        self: Pin<&'a mut Self>,
        count_down: u16,
        count_up: u16,
    ) -> Self::RotateFuture<'a>;
}
