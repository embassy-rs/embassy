#[cfg(feature = "nightly")]
mod asynch;
mod blocking;

#[cfg(feature = "nightly")]
pub(crate) use asynch::AsyncTestFlash;
pub(crate) use blocking::BlockingTestFlash;
