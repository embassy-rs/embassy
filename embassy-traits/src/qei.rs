use core::future::Future;
use core::pin::Pin;
use embedded_hal::Direction;

// Wait for a specified number of rotations either up or down
pub trait WaitForRotate {
    type RotateFuture<'a>: Future<Output = Direction> + 'a;

    fn wait_for_rotate<'a>(
        self: Pin<&'a mut Self>,
        count_down: u16,
        count_up: u16,
    ) -> Self::RotateFuture<'a>;
}
