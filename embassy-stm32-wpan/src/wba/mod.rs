mod context;
pub mod controller;
mod host_if;
mod linklayer_plat;
mod ll_sys;
mod ll_sys_if;
mod mac_sys_if;
pub mod platform;
mod power_table;
mod util_seq;

// Re-export main types
pub use controller::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler};
pub use linklayer_plat::{erase_bond_nvm_flash, set_nvm_base_address};
pub use platform::Platform;

pub mod bindings {
    pub use stm32_bindings::bindings::{mac, wba_ble_stack as ble, wba_link_layer as link_layer};
}

/// Opaque token proving the platform has been initialized.
///
/// Returned by [`Platform::new`] and required by [`Controller::new`] and the
/// [`crate::bluetooth::HCI`] constructors. The borrow ties the controller's
/// lifetime to the platform, preventing a second BLE stack initialization
/// while the first one is still alive.
pub struct Runtime {
    pub(crate) _private: (),
}
