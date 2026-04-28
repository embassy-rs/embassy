pub mod context;
pub mod controller;
pub mod host_if;
pub mod linklayer_plat;
pub mod ll_sys;
pub mod ll_sys_if;
pub mod mac_sys_if;
pub mod power_table;
pub mod runner;
pub mod util_seq;

// Re-export main types
pub use controller::{ChannelPacket, Controller, ControllerState, HighInterruptHandler, LowInterruptHandler};
pub use linklayer_plat::set_nvm_base_address;
pub use runner::ble_runner;

mod bindings {
    pub use stm32_bindings::bindings::{mac, wba_ble_stack as ble, wba_link_layer as link_layer};
}
