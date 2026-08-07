//! Operating modes for peripherals.

trait SealedMode {}

/// Operating mode for a peripheral.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// Blocking mode.
///
/// No interrupt is bound, completion is handled by polling or by an
/// interrupt handler set up by the user.
pub struct Blocking;

/// Async mode.
///
/// The built-in interrupt handler is bound, completion is handled by waking
/// the task waiting on it.
pub struct Async;

impl SealedMode for Blocking {}
impl Mode for Blocking {}
impl SealedMode for Async {}
impl Mode for Async {}
