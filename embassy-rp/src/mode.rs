//! Operating modes for peripherals.

pub(crate) trait SealedMode {
    /// `T` in blocking mode, `()` in async mode.
    #[allow(dead_code)]
    type IfBlocking<T>;
    /// `T` in async mode, `()` in blocking mode.
    #[allow(dead_code)]
    type IfAsync<T>;
}

/// Operating mode for a peripheral.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// Blocking mode.
///
/// No interrupt is bound, completion is handled by polling or by an
/// interrupt handler set up by the user.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Blocking;

/// Async mode.
///
/// The built-in interrupt handler is bound, completion is handled by waking
/// the task waiting on it.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Async;

impl SealedMode for Blocking {
    type IfBlocking<T> = T;
    type IfAsync<T> = ();
}
impl Mode for Blocking {}
impl SealedMode for Async {
    type IfBlocking<T> = ();
    type IfAsync<T> = T;
}
impl Mode for Async {}
