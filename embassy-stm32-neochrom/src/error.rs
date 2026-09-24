//! NeoChrom driver errors.

/// NemaGFX / NeoChrom runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Waiting on a command list failed.
    CommandListWait,
    /// GPU2D hardware reported a system error.
    SystemError,
    /// NemaGFX reported an error via `nema_get_error()`.
    NemaGfx {
        /// Raw error code from NemaGFX.
        code: u32,
    },
    /// A frame was already open when [`crate::NeoChrom::begin_frame`] was called.
    FrameAlreadyActive,
    /// An operation requiring an open frame was called without [`crate::NeoChrom::begin_frame`].
    NoActiveFrame,
}

/// Initialization failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InitError {
    /// GPU2D HAL initialization failed.
    Gpu2d,
    /// `nema_init()` returned an error.
    NemaGfx,
}
