// This must go FIRST so that all the other modules see its macros.
mod fmt;

use core::mem::MaybeUninit;
use core::sync::atomic::{Ordering, compiler_fence};

use embassy_hal_internal::Peri;
use embassy_stm32::interrupt;
use embassy_stm32::ipcc::{Config, Ipcc, IpccRxChannel, ReceiveInterruptHandler, TransmitInterruptHandler};
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

#[cfg(feature = "wb55_mac")]
pub mod mac;

#[cfg(feature = "wb55_ble")]
pub use crate::sub::ble::hci;

type PacketHeader = LinkedListNode;

/// Transport Layer for the Mailbox interface
pub struct TlMbox<'d> {
    pub sys_subsystem: Sys<'d>,
    pub mm_subsystem: MemoryManager<'d>,
    #[cfg(feature = "wb55_ble")]
    pub ble_subsystem: sub::ble::Ble<'d>,
    #[cfg(feature = "wb55_mac")]
    pub mac_subsystem: sub::mac::Mac<'d>,
    pub traces: IpccRxChannel<'d>,
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
    pub async fn init(
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

            #[cfg(feature = "wb55_ble")]
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

            #[cfg(feature = "wb55_mac")]
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
        let [
            (_hw_ipcc_ble_cmd_channel, _ipcc_ble_event_channel),
            (ipcc_system_cmd_rsp_channel, ipcc_system_event_channel),
            (_ipcc_mac_802_15_4_cmd_rsp_channel, _ipcc_mac_802_15_4_notification_ack_channel),
            (ipcc_mm_release_buffer_channel, _ipcc_traces_channel),
            (_ipcc_ble_lld_cmd_channel, _ipcc_ble_lld_rsp_channel),
            (_ipcc_hci_acl_data_channel, _),
        ] = Ipcc::new(ipcc, _irqs, config).split();

        let mm = sub::mm::MemoryManager::new(ipcc_mm_release_buffer_channel);
        let mut sys = sub::sys::Sys::new(ipcc_system_cmd_rsp_channel, ipcc_system_event_channel);

        debug!("sys event: {}", sys.read().await.payload());

        Self {
            sys_subsystem: sys,
            #[cfg(feature = "wb55_ble")]
            ble_subsystem: sub::ble::Ble::new(
                _hw_ipcc_ble_cmd_channel,
                _ipcc_ble_event_channel,
                _ipcc_hci_acl_data_channel,
            ),
            #[cfg(feature = "wb55_mac")]
            mac_subsystem: sub::mac::Mac::new(
                _ipcc_mac_802_15_4_cmd_rsp_channel,
                _ipcc_mac_802_15_4_notification_ack_channel,
            ),
            mm_subsystem: mm,
            traces: _ipcc_traces_channel,
        }
    }
}
