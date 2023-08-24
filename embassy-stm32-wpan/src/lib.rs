#![no_std]
#![cfg_attr(any(feature = "ble", feature = "mac"), feature(async_fn_in_trait))]
#![cfg_attr(feature = "mac", feature(type_alias_impl_trait, concat_bytes))]

// This must go FIRST so that all the other modules see its macros.
mod fmt;

use core::mem::MaybeUninit;
use core::sync::atomic::{compiler_fence, Ordering};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_stm32::interrupt;
use embassy_stm32::ipcc::{Config, Ipcc, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::peripherals::IPCC;
use sub::mm::MemoryManager;
use sub::sys::Sys;
use tables::*;
use unsafe_linked_list::LinkedListNode;

pub mod channels;
pub mod cmd;
pub mod consts;
pub mod evt;
pub mod lhci;
pub mod shci;
pub mod sub;
pub mod tables;
pub mod unsafe_linked_list;

#[cfg(feature = "mac")]
pub mod mac;

#[cfg(feature = "ble")]
pub use crate::sub::ble::hci;

type PacketHeader = LinkedListNode;

pub struct TlMbox<'d> {
    _ipcc: PeripheralRef<'d, IPCC>,

    pub sys_subsystem: Sys,
    pub mm_subsystem: MemoryManager,
    #[cfg(feature = "ble")]
    pub ble_subsystem: sub::ble::Ble,
    #[cfg(feature = "mac")]
    pub mac_subsystem: sub::mac::Mac,
}

impl<'d> TlMbox<'d> {
    pub fn init(
        ipcc: impl Peripheral<P = IPCC> + 'd,
        _irqs: impl interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_RX, ReceiveInterruptHandler>
            + interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_TX, TransmitInterruptHandler>,
        config: Config,
    ) -> Self {
        into_ref!(ipcc);

        unsafe {
            TL_REF_TABLE.as_mut_ptr().write_volatile(RefTable {
                device_info_table: TL_DEVICE_INFO_TABLE.as_ptr(),
                ble_table: TL_BLE_TABLE.as_ptr(),
                thread_table: TL_THREAD_TABLE.as_ptr(),
                sys_table: TL_SYS_TABLE.as_ptr(),
                mem_manager_table: TL_MEM_MANAGER_TABLE.as_ptr(),
                traces_table: TL_TRACES_TABLE.as_ptr(),
                mac_802_15_4_table: TL_MAC_802_15_4_TABLE.as_ptr(),
                zigbee_table: TL_ZIGBEE_TABLE.as_ptr(),
                lld_tests_table: TL_LLD_TESTS_TABLE.as_ptr(),
                ble_lld_table: TL_BLE_LLD_TABLE.as_ptr(),
            });

            TL_SYS_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_DEVICE_INFO_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_BLE_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_THREAD_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_MEM_MANAGER_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());

            TL_TRACES_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_MAC_802_15_4_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_ZIGBEE_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_LLD_TESTS_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            TL_BLE_LLD_TABLE
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());

            EVT_POOL
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            SYS_SPARE_EVT_BUF
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());
            CS_BUFFER
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());

            #[cfg(feature = "ble")]
            {
                BLE_SPARE_EVT_BUF
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());

                BLE_CMD_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
                HCI_ACL_DATA_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
            }

            #[cfg(feature = "mac")]
            {
                MAC_802_15_4_CMD_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
                MAC_802_15_4_NOTIF_RSP_EVT_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
            }
        }

        compiler_fence(Ordering::SeqCst);

        Ipcc::enable(config);

        Self {
            _ipcc: ipcc,
            sys_subsystem: sub::sys::Sys::new(),
            #[cfg(feature = "ble")]
            ble_subsystem: sub::ble::Ble::new(),
            #[cfg(feature = "mac")]
            mac_subsystem: sub::mac::Mac::new(),
            mm_subsystem: sub::mm::MemoryManager::new(),
        }
    }
}
