//! This is a stub module for CAN FD support.
//!
//! The structure of this module probably shouldn't look all that different to `classic/`. Despite the protocol-level differences between Classic CAN and CAN FD, the high
//! level design of this submodule (and its interfaces) should be able to take a lot of cues from `classic/`, since they share the same FlexCAN hardware at the end of the
//! day.
//!
//! Misc (and very non-exhaustive) notes about an eventual CAN FD implementation:
//! - The RX side of the CAN FD mailbox can probably be nearly identical to `classic/mailbox.rs`'s RX subsystem, since CAN FD should be able to use the same
//! Enhanced RX FIFO configuration as Classic CAN. The FIFO element sizes are fixed at 64 bytes (even for Classic CAN), so they don't need to be resized for CAN FD.
//! The main thing that would need to be different for RX is the ID_HIT handling, unless I'm forgetting something.
//! - `filter.rs` is somewhat configured around how the Enhanced RX FIFO handles RX filters, so if an eventual CAN FD implementation can't use the Enhanced RX FIFO for whatever
//! reason, `filter.rs` may need to be modified (and possibly split into a `classic/filter.rs` and `fd/filter.rs`). This is probably a non-issue since there is no reason
//! for the CAN FD implementation to not use the Enhanced RX FIFO (since it seems optimized specifically for CAN FD), but just putting this note here in case something comes up.
