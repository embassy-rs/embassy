//! USB Audio Class 2.0 implementations.
//!
//! Contains:
//! - The `io_box` class with line out (host to device) and line in (device to host) in one
//!   audio function, with explicit feedback, for high-speed devices

pub mod io_box;

use super::Channel;

/// Returns the `bmChannelConfig` for `channels`. The spatial locations up to `Top` use the same
/// bits as UAC1's `wChannelConfig`.
fn channel_config(channels: &[Channel]) -> u32 {
    channels
        .iter()
        .fold(0, |cfg, ch| cfg | u16::from(ch.get_channel_config()) as u32)
}
