use core::future::poll_fn;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

#[cfg(any(feature = "wb-ble", feature = "wb-mac"))]
use embassy_futures::select::{Either, select};
use embassy_hal_internal::Peri;
use embassy_stm32::interrupt;
use embassy_stm32::ipcc::{Config, Ipcc, IpccRxChannel, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::peripherals::IPCC;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::{Duration, with_timeout};
use sub::mm::MemoryManager;
use sub::sys::Sys;
use tables::*;
use unsafe_linked_list::LinkedListNode;

pub mod channels;
pub mod cmd;
pub mod consts;
pub mod evt;
pub mod fus;
pub mod lhci;
pub mod shci;
pub mod sub;
pub mod tables;
pub mod unsafe_linked_list;

#[cfg(feature = "wb-mac")]
pub mod mac;

use crate::shci::SchiSysEventReady;
#[cfg(feature = "wb-ble")]
use crate::shci::ShciBleInitCmdParam;
#[cfg(feature = "wb-ble")]
use crate::sub::ble::Ble;
#[cfg(feature = "wb-ble")]
pub use crate::sub::ble::hci;
#[cfg(feature = "wb-mac")]
use crate::sub::mac::Mac;

type PacketHeader = LinkedListNode;

#[allow(unused)]
struct Flag {
    state: AtomicBool,
    waker: AtomicWaker,
}

#[allow(unused)]
impl Flag {
    pub const fn new(state: bool) -> Self {
        Self {
            state: AtomicBool::new(state),
            waker: AtomicWaker::new(),
        }
    }

    pub fn set_high(&self) {
        if !self.state.swap(true, Ordering::AcqRel) {
            self.waker.wake();
        }
    }

    pub fn set_low(&self) {
        if self.state.swap(false, Ordering::AcqRel) {
            self.waker.wake();
        }
    }

    pub async fn wait_for_high(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            if !self.state.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    pub async fn wait_for_low(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            if self.state.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }
}

/// Transport Layer for the Mailbox interface
pub struct TlMbox<'d> {
    pub sys_subsystem: Sys<'d>,
    pub mm_subsystem: MemoryManager<'d>,
    #[cfg(feature = "wb-ble")]
    pub ble_subsystem: sub::ble::Ble<'d>,
    #[cfg(feature = "wb-mac")]
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

            #[cfg(feature = "wb-ble")]
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

            #[cfg(feature = "wb-mac")]
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
            (_ipcc_hci_acl_tx_data_channel, _ipcc_hci_acl_rx_data_channel),
        ] = Ipcc::new(ipcc, _irqs, config).split();

        let mm = sub::mm::MemoryManager::new(ipcc_mm_release_buffer_channel);
        let sys = sub::sys::Sys::new(ipcc_system_cmd_rsp_channel, ipcc_system_event_channel);

        Self {
            sys_subsystem: sys,
            #[cfg(feature = "wb-ble")]
            ble_subsystem: sub::ble::Ble::new(
                _hw_ipcc_ble_cmd_channel,
                _ipcc_ble_event_channel,
                _ipcc_hci_acl_tx_data_channel,
                _ipcc_hci_acl_rx_data_channel,
            ),
            #[cfg(feature = "wb-mac")]
            mac_subsystem: sub::mac::Mac::new(
                _ipcc_mac_802_15_4_cmd_rsp_channel,
                _ipcc_mac_802_15_4_notification_ack_channel,
            ),
            mm_subsystem: mm,
            traces: _ipcc_traces_channel,
        }
    }

    /// Initialise the Transport Layer, and waits for the ready event with a timeout
    pub async fn wait_ready(
        ipcc: Peri<'d, IPCC>,
        irqs: impl interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_RX, ReceiveInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_TX, TransmitInterruptHandler>,
        config: Config,
    ) -> Result<Self, ()> {
        let mut this = Self::init(ipcc, irqs, config);

        let sys_event = with_timeout(Duration::from_millis(500), this.sys_subsystem.read_ready())
            .await
            .map_err(|_| ())??;

        match sys_event {
            SchiSysEventReady::WirelessFwRunning => Ok(this),
            _ => Err(()),
        }
    }

    #[cfg(feature = "wb-ble")]
    /// Initialise the BLE subsystem
    pub async fn init_ble(mut self, param: ShciBleInitCmdParam) -> Result<(Ble<'d>, MemoryManager<'d>), ()> {
        match select(
            self.mm_subsystem.run_queue(),
            self.sys_subsystem.shci_c2_ble_init(param),
        )
        .await
        {
            Either::Second(res) => res,
            _ => unreachable!(),
        }?;

        Ok((self.ble_subsystem, self.mm_subsystem))
    }

    #[cfg(feature = "wb-mac")]
    /// Initialise the BLE subsystem
    pub async fn init_mac(mut self) -> Result<(Mac<'d>, MemoryManager<'d>), ()> {
        match select(
            self.mm_subsystem.run_queue(),
            self.sys_subsystem.shci_c2_mac_802_15_4_init(),
        )
        .await
        {
            Either::Second(res) => res,
            _ => unreachable!(),
        }?;

        Ok((self.mac_subsystem, self.mm_subsystem))
    }
}
