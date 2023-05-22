//! Adapters between embedded-hal traits.

mod blocking_async;
mod yielding_async;

pub use blocking_async::BlockingAsync;
pub use yielding_async::YieldingAsync;
