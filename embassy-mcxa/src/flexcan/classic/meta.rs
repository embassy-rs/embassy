//! This module is intended for any metaprogramming, macros, or compile-time config stuff relavent to the flexcan::classic module.

/// These macros/consts allow users to define custom sizes for the software RX queue
/// this module uses, via an environment variable. The size defaults to `RX_QUEUE_SIZE_DEFAULT` 
/// if the env var is left unspecified by the user.
pub(in crate::flexcan::classic) mod rx_queue_size {
    // Name of the env var.
    macro_rules! env_var_name { () => { "EMBASSY_MCXA_FLEXCAN_CLASSIC_RX_QUEUE_SIZE" } }
    pub(in crate::flexcan::classic) use env_var_name;

    // Default size of the queue, in messages/Frames. 
    // This is a macro instead of a `const`` so we can use it in docs.
    macro_rules! rx_queue_size_default { () => { 64 } }
    pub(in crate::flexcan::classic) use rx_queue_size_default;

    /// The size of the software queue flexcan::classic uses for RX messages, in Frames.
    /// This value can be configured by the user via an environment variable. Otherwise, it defaults to a default value.
    pub (in crate::flexcan::classic) const RX_QUEUE_SIZE: usize = match option_env!(env_var_name!()) {
        Some(val) => {
            let bytes = val.as_bytes();
            let mut num = 0usize;
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                assert!(b.is_ascii_digit(), concat!(env_var_name!(), " must be a decimal integer"));
                assert!(num < usize::MAX / 10, concat!(env_var_name!(), " is too large"));
                num = num * 10 + (b - b'0') as usize;
                i += 1;
            }
            assert!(num > 0, concat!(env_var_name!(), " must be greater than 0"));
            num
        }
        None => rx_queue_size_default!(),
    };
}

/// Shared rustdoc for the public TX/RX methods. These methods are exposed (with
/// identical documentation) on `FlexCan`, `FlexCanTx`, and `FlexCanRx`, plus the
/// `functions` module, so their doc comments are defined once here and applied
/// via `#[doc = ...]`.
pub(in crate::flexcan::classic) mod docs {
    macro_rules! doc_send {
        () => { concat!(
            "Sends a CAN message.\n",
            "\n",
            "If there's no space left in the TX buffers, this\n",
            "call asynchronously waits for space to free up, and then tries again.\n",
            "\n",
            "Note: During a BusOff event, this function will asynchronously wait until\n",
            "the bus recovers. This is due to the behavior mentioned above: The TX mailbox\n",
            "doesn't drain during BusOff (and will eventually fill up), causing this\n",
            "function to wait until after recovery when buffers start becoming available again.\n",
            "\n",
            "Unless explicitly disabled, FlexCAN will recover from BusOff automatically. However,\n",
            "if you need to be notified immediately when a BusOff event occurs, see the `try_send()`\n",
            "and `error_mode()` functions.",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_send;

    macro_rules! doc_try_send {
        () => { concat!(
            "Attempts to send a CAN message.\n",
            "\n",
            "This function returns immediately upon being called, either with `Ok(())` or\n",
            "a `SendError`. For this function's async counterpart, see `send()`.",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_try_send;

    macro_rules! doc_receive {
        () => { concat!(
            "Receives a CAN message.\n",
            "\n",
            "If there are no new messages, this call asynchronously\n",
            "waits for new messages to arrive.\n",
            "\n",
            "Note: The size of the FlexCan classic-mode RX queue can be configured via the ",
            $crate::flexcan::classic::meta::rx_queue_size::env_var_name!(),
            " environment variable. For example, in your .cargo/config.toml, you could add\n",
            "```toml\n",
            "[env]\n",
            $crate::flexcan::classic::meta::rx_queue_size::env_var_name!(), " = \"32\"\n",
            "```\n",
            "if you wanted the queue to store 32 frames.\n",
            "\n",
            "If you don't specify anything, the queue will default to a size of ",
            $crate::flexcan::classic::meta::rx_queue_size::rx_queue_size_default!(),
            " frames.",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_receive;

    macro_rules! doc_try_receive {
        () => { "Like `receive()`, but returns immediately if there are no new messages (rather than waiting for more to arrive)." };
    }
    pub(in crate::flexcan::classic) use doc_try_receive;

    macro_rules! doc_error_mode {
        () => { concat!(
            "Returns the error mode the FlexCAN is currently in.\n",
            "See `BusErrorMode`.",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_error_mode;
}