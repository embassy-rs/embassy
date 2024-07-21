//! Mutex primitives.
//!
//! This module provides a trait for mutexes that can be used in different contexts.
pub use scoped_mutex::impls::cs::CriticalSectionRawMutex;
pub use scoped_mutex::impls::local::LocalRawMutex as NoopRawMutex;
#[cfg(any(cortex_m, feature = "std"))]
pub use scoped_mutex::impls::thread_mode::ThreadModeRawMutex;
pub use scoped_mutex::ConstScopedRawMutex;
// Semver re-exports
pub use scoped_mutex::ConstScopedRawMutex as RawMutex;
