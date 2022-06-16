/// A type that can retrieved unsafely from anywhere.
pub trait Steal {
    /// Retrieve and instance of this type.
    ///
    /// # Safety
    ///
    /// It is the responsibility of the application to ensure that the
    /// usage of the returned instance is not in conflict with other uses
    /// of this instance.
    ///
    /// The implementation may panic if the instance is already in use.
    unsafe fn steal() -> Self;
}
