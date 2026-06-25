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
                assert!(b >= b'0' && b <= b'9', concat!(env_var_name!(), " must be a decimal integer"));
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