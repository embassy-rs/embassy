//! Async buffered UART driver.
//!
//! Note that discarding a future from a read or write operation may lead to losing
//! data. For example, when using `futures_util::future::select` and completion occurs
//! on the "other" future, you should capture the incomplete future and continue to use
//! it for the next read or write. This pattern is a consideration for all IO, and not
//! just serial communications.
//!
//! Please also see [crate::uarte] to understand when [BufferedUarte] should be used.
#[cfg_attr(not(feature = "_nrf54l"), path = "v1.rs")]
#[cfg_attr(feature = "_nrf54l", path = "v2.rs")]
mod _version;

pub use _version::*;
