//! Git branch + revision baked in at build time (`build.rs`).

pub const FIRMWARE_ID: &str = env!("FIRMWARE_ID");
