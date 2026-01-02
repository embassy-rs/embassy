//! The embassy-stm32-wpan crate aims to provide safe use of the commands necessary to interface
//! with the Cortex C0 CPU2 coprocessor of STM32WB microcontrollers. It implements safe wrappers
//! around the Transport Layer, and in particular the system, memory, BLE and Mac channels.
//!
//! # Design
//!
//! This crate loosely follows the Application Note 5289 "How to build wireless applications with
//! STM32WB MCUs"; several of the startup procedures laid out in Annex 14.1 are implemented using
//! inline copies of the code contained within the `stm32wb_copro` C library.
//!
//! BLE commands are implemented via use of the [stm32wb_hci] crate, for which the
//! [stm32wb_hci::Controller] trait has been implemented.

#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]
#![allow(static_mut_refs)] // TODO: Fix

// This must go FIRST so that all the other modules see its macros.
mod fmt;

use core::mem::MaybeUninit;
use core::sync::atomic::{compiler_fence, Ordering};

use embassy_hal_internal::Peri;
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

/// Transport Layer for the Mailbox interface
pub struct TlMbox<'d> {
    _ipcc: Peri<'d, IPCC>,

    pub sys_subsystem: Sys,
    pub mm_subsystem: MemoryManager,
    #[cfg(feature = "ble")]
    pub ble_subsystem: sub::ble::Ble,
    #[cfg(feature = "mac")]
    pub mac_subsystem: sub::mac::Mac,
}

impl<'d> TlMbox<'d> {
    /// Initialise the Transport Layer, and creates and returns a wrapper around it.
    ///
    /// This method performs the initialisation laid out in AN5289 annex 14.1. However, it differs
    /// from the implementation documented in Figure 64, to avoid needing to reference any C
    /// function pointers.
    ///
    /// Annex 14.1 lays out the following methods that should be called:
    ///     1. tl_mbox.c/TL_Init, which initialises the reference table that is shared between CPU1
    ///        and CPU2.
    ///     2. shci_tl.c/shci_init(), which initialises the system transport layer, and in turn
    ///        calls tl_mbox.c/TL_SYS_Init, which initialises SYSTEM_EVT_QUEUE channel.
    ///     3. tl_mbox.c/TL_MM_Init(), which initialises the channel used for sending memory
    ///        manager commands.
    ///     4. tl_mbox.c/TL_Enable(), which enables the IPCC, and starts CPU2.
    /// This implementation initialises all of the shared refernce tables and all IPCC channel that
    /// would be initialised by this process. The developer should therefore treat this method as
    /// completing all steps in Figure 64.
    ///
    /// Once this method has been called, no system commands may be sent until the CPU2 ready
    /// signal is received, via [sys_subsystem.read]; this completes the procedure laid out in
    /// Figure 65.
    ///
    /// If the `ble` feature is enabled, at this point, the user should call
    /// [sys_subsystem.shci_c2_ble_init], before any commands are written to the
    /// [TlMbox.ble_subsystem] ([sub::ble::Ble::new()] completes the process that would otherwise
    /// be handled by `TL_BLE_Init`; see Figure 66). This completes the procedure laid out in
    /// Figure 66.
    // TODO: document what the user should do after calling init to use the mac_802_15_4 subsystem
    pub fn init(
        ipcc: Peri<'d, IPCC>,
        _irqs: impl interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_RX, ReceiveInterruptHandler>
            + interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_TX, TransmitInterruptHandler>,
        config: Config,
    ) -> Self {
        // this is an inlined version of TL_Init from the STM32WB firmware as requested by AN5289.
        // HW_IPCC_Init is not called, and its purpose is (presumably?) covered by this
        // implementation
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

        // this is equivalent to `HW_IPCC_Enable`, which is called by `TL_Enable`
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
