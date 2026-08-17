//! The pender: the callback executors use to notify they have work to do.

unitrait::unitrait! {
    /// The pender: the callback executors use to notify they have work to do.
    ///
    /// There is one global pender for the whole program, registered with the
    /// [`pender_impl!`](crate::pender_impl) macro. It is provided by the
    /// platform/architecture implementation, either the built-in ones enabled by the
    /// `platform-*` Cargo features, or a custom one.
    pub trait Pender {
        /// Called when an executor has work to do.
        ///
        /// The implementation must arrange for [`crate::raw::Executor::poll`] to be called on the
        /// executor identified by `context` as soon as possible.
        ///
        /// `context` is the arbitrary data passed to [`crate::raw::Executor::new`] by whoever
        /// created the executor. It can be used to differentiate between executors, or to pass a
        /// pointer to a callback that should be called.
        ///
        /// This function can be called from *any* context: any thread, any interrupt priority
        /// level, etc. It may be called synchronously from any executor method call as well.
        /// The implementation must deal with this correctly.
        ///
        /// In particular, the implementation must NOT call `poll` directly from within this
        /// function, as this violates the requirement for `poll` to not be called reentrantly.
        #[symbol = "__pender"]
        pub(crate) fn pend(context: *mut ());
    }

    /// Register a type as the global [`Pender`] implementation.
    ///
    /// This must be done exactly once in the crate tree, by the platform/architecture
    /// implementation. Enabling a `platform-*` Cargo feature on `embassy-executor` registers
    /// the corresponding built-in pender.
    macro pender_impl(path = $crate::pender);
}
