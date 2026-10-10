//! Operating modes for peripherals.

trait SealedMode {}

/// Operating mode for a peripheral.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// Blocking mode. No interrupt is bound; methods busy-wait.
pub struct Blocking;
/// Async mode. Completes work in an interrupt.
pub struct Async;

impl SealedMode for Blocking {}
impl Mode for Blocking {}
impl SealedMode for Async {}
impl Mode for Async {}
