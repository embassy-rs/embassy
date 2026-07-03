//! This module is intended for any metaprogramming, macros, or compile-time config stuff relavent to the flexcan::classic module.

/// Shared rustdoc for the public TX/RX methods. These methods are exposed (with
/// identical documentation) on `FlexCan`, `FlexCanTx`, and `FlexCanRx`, plus the
/// `functions` module, so their doc comments are defined once here and applied
/// via `#[doc = ...]`.
pub(in crate::flexcan::classic) mod docs {
    macro_rules! doc_send {
        () => {
            concat!(
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
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_send;

    macro_rules! doc_try_send {
        () => {
            concat!(
                "Attempts to send a CAN message.\n",
                "\n",
                "This function returns immediately upon being called, either with `Ok(())` or\n",
                "a `SendError`. For this function's async counterpart, see `send()`.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_try_send;

    macro_rules! doc_receive {
        () => {
            concat!(
                "Receives a CAN message.\n",
                "\n",
                "If there are no new messages, this call asynchronously\n",
                "waits for new messages to arrive.\n",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_receive;

    macro_rules! doc_try_receive {
        () => { concat!(
            "Like `receive()`, but returns immediately if there are no new messages (rather than waiting for more to arrive).",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_try_receive;

    macro_rules! doc_error_mode {
        () => {
            concat!(
                "Returns the error mode the FlexCAN is currently in.\n",
                "See `BusErrorMode`.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_error_mode;

    macro_rules! doc_rx_dropped_count {
        () => { concat!(
            "Indicates the number of RX frames dropped so far due to the RX queue being full.",
            "If you're seeing this number increase, you are receiving messages faster than the RX queue can handle.",
            "This can be mitigated by increasing the size of the RX queue.\n",
            "\nNote: This function tracks frames dropped specifically due to the RX queue being full. It doesn't track other
            sources of dropped frames that may have occured at a lower level.\n",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_rx_dropped_count;

    macro_rules! doc_tx_mailbox_full_count {
        () => { concat!(
            "Indicates the number of times new transmissions have been blocked or deferred due to the TX mailbox being full so far.\n\n",
            "Note: See the docs for `send()` and `try_send()` for each function's behavior when it encounters a full mailbox.",
        )};
    }
    pub(in crate::flexcan::classic) use doc_tx_mailbox_full_count;
}
