#[cfg(not(miri))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    // These test cases should fail to compile since OnceLock and LazyLock should not unconditionally implement sync
    // for all types. These tests are regression tests against the following issues:
    // * https://github.com/embassy-rs/embassy/issues/4307
    // * https://github.com/embassy-rs/embassy/issues/3904
    t.compile_fail("tests/ui/sync_impl/lazy_lock_function.rs");
    t.compile_fail("tests/ui/sync_impl/lazy_lock_type.rs");
    t.compile_fail("tests/ui/sync_impl/once_lock.rs");

    // NoopRawMutex is not Sync, so the `From` impls that produce the `SendDyn*`
    // wrappers must not be available for channels built on it.
    t.compile_fail("tests/ui/pubsub_send/subscriber_noop_mutex.rs");
    t.compile_fail("tests/ui/pubsub_send/publisher_noop_mutex.rs");
    t.compile_fail("tests/ui/pubsub_send/immediate_publisher_noop_mutex.rs");
}
