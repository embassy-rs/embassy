mod context;
pub mod controller;
mod host_if;
mod linklayer_plat;
mod ll_sys;
mod ll_sys_if;
mod mac_sys_if;
mod power_table;
pub mod runner;
mod util_seq;

// Re-export main types
pub use controller::{ChannelPacket, Controller, ControllerState, HighInterruptHandler, LowInterruptHandler};
pub use linklayer_plat::set_nvm_base_address;
pub use runner::ble_runner;

mod bindings {
    pub use stm32_bindings::bindings::{mac, wba_ble_stack as ble, wba_link_layer as link_layer};
}
