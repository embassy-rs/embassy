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

use core::mem;

// Re-export main types
pub use controller::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler};
pub use linklayer_plat::set_nvm_base_address;
pub use platform::Platform;

mod bindings {
    pub use stm32_bindings::bindings::{mac, wba_ble_stack as ble, wba_link_layer as link_layer};
}

#[allow(private_bounds)]
pub trait Runtime: SealedRuntime + 'static {
    fn to_basic(&mut self) -> &mut BasicRuntime;
}
trait SealedRuntime {}

/// Basic runtime with just RNG
pub struct BasicRuntime {
    _private: (),
}

/// Runtime with RNG, PKA, and AES
pub struct FullRuntime {
    _private: (),
}

impl SealedRuntime for BasicRuntime {}
impl Runtime for BasicRuntime {
    fn to_basic(&mut self) -> &mut BasicRuntime {
        self
    }
}

impl SealedRuntime for FullRuntime {}
impl Runtime for FullRuntime {
    fn to_basic(&mut self) -> &mut BasicRuntime {
        unsafe { mem::transmute(self) }
    }
}
