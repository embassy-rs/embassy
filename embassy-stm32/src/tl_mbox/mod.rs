use core::mem::MaybeUninit;

use bit_field::BitField;
use embassy_cortex_m::interrupt::Interrupt;
use embassy_futures::block_on;
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;

use self::cmd::{AclDataPacket, CmdPacket};
use self::evt::{CcEvt, EvtBox};
use self::ipcc::{Config, Ipcc};
use self::unsafe_linked_list::LinkedListNode;
use crate::interrupt;
use crate::peripherals::IPCC;

pub mod ble;
pub mod channels;
pub mod cmd;
pub mod consts;
pub mod evt;
pub mod hci;
pub mod ipcc;
pub mod mm;
pub mod shci;
pub mod sys;
pub mod unsafe_linked_list;

/// Interrupt handler.
pub struct ReceiveInterruptHandler {}

impl interrupt::Handler<interrupt::IPCC_C1_RX> for ReceiveInterruptHandler {
    unsafe fn on_interrupt() {
        debug!("ipcc rx interrupt");

        if Ipcc::is_rx_pending(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL) {
            debug!("sys evt");
            sys::Sys::evt_handler();
        } else if Ipcc::is_rx_pending(channels::cpu2::IPCC_BLE_EVENT_CHANNEL) {
            debug!("ble evt");
            ble::Ble::evt_handler();
        } else {
            todo!()
        }

        STATE.signal(());
    }
}

pub struct TransmitInterruptHandler {}

impl interrupt::Handler<interrupt::IPCC_C1_TX> for TransmitInterruptHandler {
    unsafe fn on_interrupt() {
        debug!("ipcc tx interrupt");

        if Ipcc::is_tx_pending(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL) {
            debug!("sys cmd rsp");
            // TODO: handle this case
            let cc = sys::Sys::cmd_evt_handler();
            let a = unsafe { core::slice::from_raw_parts(&cc as *const _ as *const u8, core::mem::size_of::<CcEvt>()) };
            debug!("{:#04x}", a);

            LAST_CC_EVT.signal(cc);
        } else if Ipcc::is_tx_pending(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL) {
            debug!("mm");
            mm::MemoryManager::free_buf_handler();
        } else {
            todo!()
        }

        STATE.signal(());
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct SafeBootInfoTable {
    version: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct RssInfoTable {
    version: u32,
    memory_size: u32,
    rss_info: u32,
}

/**
 * Version
 * [0:3]   = Build - 0: Untracked - 15:Released - x: Tracked version
 * [4:7]   = branch - 0: Mass Market - x: ...
 * [8:15]  = Subversion
 * [16:23] = Version minor
 * [24:31] = Version major
 *
 * Memory Size
 * [0:7]   = Flash ( Number of 4k sector)
 * [8:15]  = Reserved ( Shall be set to 0 - may be used as flash extension )
 * [16:23] = SRAM2b ( Number of 1k sector)
 * [24:31] = SRAM2a ( Number of 1k sector)
 */
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct WirelessFwInfoTable {
    version: u32,
    memory_size: u32,
    thread_info: u32,
    ble_info: u32,
}

impl WirelessFwInfoTable {
    pub fn version_major(&self) -> u8 {
        let version = self.version;
        (version.get_bits(24..31) & 0xff) as u8
    }

    pub fn version_minor(&self) -> u8 {
        let version = self.version;
        (version.clone().get_bits(16..23) & 0xff) as u8
    }

    pub fn subversion(&self) -> u8 {
        let version = self.version;
        (version.clone().get_bits(8..15) & 0xff) as u8
    }

    /// Size of FLASH, expressed in number of 4K sectors.
    pub fn flash_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.clone().get_bits(0..7) & 0xff) as u8
    }

    /// Size of SRAM2a, expressed in number of 1K sectors.
    pub fn sram2a_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.clone().get_bits(24..31) & 0xff) as u8
    }

    /// Size of SRAM2b, expressed in number of 1K sectors.
    pub fn sram2b_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.clone().get_bits(16..23) & 0xff) as u8
    }
}

#[derive(Debug, Clone)]
#[repr(C, align(4))]
pub struct DeviceInfoTable {
    pub safe_boot_info_table: SafeBootInfoTable,
    pub rss_info_table: RssInfoTable,
    pub wireless_fw_info_table: WirelessFwInfoTable,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct BleTable {
    pcmd_buffer: *mut CmdPacket,
    pcs_buffer: *const u8,
    pevt_queue: *const u8,
    phci_acl_data_buffer: *mut AclDataPacket,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct ThreadTable {
    nostack_buffer: *const u8,
    clicmdrsp_buffer: *const u8,
    otcmdrsp_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct LldTestsTable {
    clicmdrsp_buffer: *const u8,
    m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct BleLldTable {
    cmdrsp_buffer: *const u8,
    m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct ZigbeeTable {
    notif_m0_to_m4_buffer: *const u8,
    appli_cmd_m4_to_m0_bufer: *const u8,
    request_m0_to_m4_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct SysTable {
    pcmd_buffer: *mut CmdPacket,
    sys_queue: *const LinkedListNode,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct MemManagerTable {
    spare_ble_buffer: *const u8,
    spare_sys_buffer: *const u8,

    blepool: *const u8,
    blepoolsize: u32,

    pevt_free_buffer_queue: *mut LinkedListNode,

    traces_evt_pool: *const u8,
    tracespoolsize: u32,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct TracesTable {
    traces_queue: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct Mac802_15_4Table {
    p_cmdrsp_buffer: *const u8,
    p_notack_buffer: *const u8,
    evt_queue: *const u8,
}

/// Reference table. Contains pointers to all other tables.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RefTable {
    device_info_table: *const DeviceInfoTable,
    ble_table: *const BleTable,
    thread_table: *const ThreadTable,
    sys_table: *const SysTable,
    mem_manager_table: *const MemManagerTable,
    traces_table: *const TracesTable,
    mac_802_15_4_table: *const Mac802_15_4Table,
}

#[link_section = "TL_REF_TABLE"]
pub static mut TL_REF_TABLE: MaybeUninit<RefTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_DEVICE_INFO_TABLE: MaybeUninit<DeviceInfoTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_BLE_TABLE: MaybeUninit<BleTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_THREAD_TABLE: MaybeUninit<ThreadTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_SYS_TABLE: MaybeUninit<SysTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MEM_MANAGER_TABLE: MaybeUninit<MemManagerTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_TRACES_TABLE: MaybeUninit<TracesTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MAC_802_15_4_TABLE: MaybeUninit<Mac802_15_4Table> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut FREE_BUF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

// Not in shared RAM
static mut LOCAL_FREE_BUF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[allow(dead_code)] // Not used currently but reserved
#[link_section = "MB_MEM2"]
static mut TRACES_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

type PacketHeader = LinkedListNode;

const TL_PACKET_HEADER_SIZE: usize = core::mem::size_of::<PacketHeader>();
const TL_EVT_HEADER_SIZE: usize = 3;
const TL_CS_EVT_SIZE: usize = core::mem::size_of::<evt::CsEvt>();

#[link_section = "MB_MEM2"]
static mut CS_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYSTEM_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut SYS_CMD_BUF: MaybeUninit<CmdPacket> = MaybeUninit::uninit();

/**
 * Queue length of BLE Event
 * This parameter defines the number of asynchronous events that can be stored in the HCI layer before
 * being reported to the application. When a command is sent to the BLE core coprocessor, the HCI layer
 * is waiting for the event with the Num_HCI_Command_Packets set to 1. The receive queue shall be large
 * enough to store all asynchronous events received in between.
 * When CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE is set to 27, this allow to store three 255 bytes long asynchronous events
 * between the HCI command and its event.
 * This parameter depends on the value given to CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE. When the queue size is to small,
 * the system may hang if the queue is full with asynchronous events and the HCI layer is still waiting
 * for a CC/CS event, In that case, the notification TL_BLE_HCI_ToNot() is called to indicate
 * to the application a HCI command did not receive its command event within 30s (Default HCI Timeout).
 */
const CFG_TLBLE_EVT_QUEUE_LENGTH: usize = 5;
const CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE: usize = 255;
const TL_BLE_EVENT_FRAME_SIZE: usize = TL_EVT_HEADER_SIZE + CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE;

const fn divc(x: usize, y: usize) -> usize {
    ((x) + (y) - 1) / (y)
}

const POOL_SIZE: usize = CFG_TLBLE_EVT_QUEUE_LENGTH * 4 * divc(TL_PACKET_HEADER_SIZE + TL_BLE_EVENT_FRAME_SIZE, 4);

#[link_section = "MB_MEM2"]
static mut EVT_POOL: MaybeUninit<[u8; POOL_SIZE]> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYS_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut BLE_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut BLE_CMD_BUFFER: MaybeUninit<CmdPacket> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
//                                 fuck these "magic" numbers from ST ---v---v
static mut HCI_ACL_DATA_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + 5 + 251]> = MaybeUninit::uninit();

static HEAPLESS_EVT_QUEUE: Channel<CriticalSectionRawMutex, EvtBox, 32> = Channel::new();

static STATE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// current event that is produced during IPCC IRQ handler execution
/// on SYS channel
/// last received Command Complete event
static LAST_CC_EVT: Signal<CriticalSectionRawMutex, CcEvt> = Signal::new();

pub struct TlMbox<'d> {
    sys: sys::Sys,
    ble: ble::Ble,
    _mm: mm::MemoryManager,

    _ipcc: PeripheralRef<'d, IPCC>,
}

impl<'d> TlMbox<'d> {
    pub fn new(
        ipcc: impl Peripheral<P = IPCC> + 'd,
        _irqs: impl interrupt::Binding<interrupt::IPCC_C1_RX, ReceiveInterruptHandler>
            + interrupt::Binding<interrupt::IPCC_C1_TX, TransmitInterruptHandler>,
        config: Config,
    ) -> Self {
        into_ref!(ipcc);

        unsafe {
            TL_REF_TABLE.as_mut_ptr().write_volatile(RefTable {
                device_info_table: TL_DEVICE_INFO_TABLE.as_mut_ptr(),
                ble_table: TL_BLE_TABLE.as_ptr(),
                thread_table: TL_THREAD_TABLE.as_ptr(),
                sys_table: TL_SYS_TABLE.as_ptr(),
                mem_manager_table: TL_MEM_MANAGER_TABLE.as_ptr(),
                traces_table: TL_TRACES_TABLE.as_ptr(),
                mac_802_15_4_table: TL_MAC_802_15_4_TABLE.as_ptr(),
            });

            TL_SYS_TABLE = MaybeUninit::zeroed();
            TL_DEVICE_INFO_TABLE = MaybeUninit::zeroed();
            TL_BLE_TABLE = MaybeUninit::zeroed();
            TL_THREAD_TABLE = MaybeUninit::zeroed();
            TL_MEM_MANAGER_TABLE = MaybeUninit::zeroed();
            TL_TRACES_TABLE = MaybeUninit::zeroed();
            TL_MAC_802_15_4_TABLE = MaybeUninit::zeroed();

            EVT_POOL = MaybeUninit::zeroed();
            SYS_SPARE_EVT_BUF = MaybeUninit::zeroed();
            BLE_SPARE_EVT_BUF = MaybeUninit::zeroed();

            CS_BUFFER = MaybeUninit::zeroed();
            BLE_CMD_BUFFER = MaybeUninit::zeroed();
            HCI_ACL_DATA_BUFFER = MaybeUninit::zeroed();
        }

        Ipcc::enable(config);

        let sys = sys::Sys::new();
        let ble = ble::Ble::new();
        let mm = mm::MemoryManager::new();

        // enable interrupts
        crate::interrupt::IPCC_C1_RX::unpend();
        crate::interrupt::IPCC_C1_TX::unpend();

        unsafe { crate::interrupt::IPCC_C1_RX::enable() };
        unsafe { crate::interrupt::IPCC_C1_TX::enable() };

        Self {
            sys,
            ble,
            _mm: mm,
            _ipcc: ipcc,
        }
    }

    /// Returns CPU2 wireless firmware information (if present).
    pub fn wireless_fw_info(&self) -> Option<WirelessFwInfoTable> {
        let info = unsafe { &(*(*TL_REF_TABLE.as_ptr()).device_info_table).wireless_fw_info_table };

        // Zero version indicates that CPU2 wasn't active and didn't fill the information table
        if info.version != 0 {
            Some(*info)
        } else {
            None
        }
    }

    /// picks single [`EvtBox`] from internal event queue.
    ///
    /// Internal event queu is populated in IPCC_RX_IRQ handler
    pub fn dequeue_event(&mut self) -> Option<EvtBox> {
        HEAPLESS_EVT_QUEUE.try_recv().ok()
    }

    /// retrieves last Command Complete event and removes it from mailbox
    pub fn pop_last_cc_evt(&mut self) -> Option<CcEvt> {
        if LAST_CC_EVT.signaled() {
            let cc = Some(block_on(LAST_CC_EVT.wait()));
            LAST_CC_EVT.reset();

            cc
        } else {
            None
        }
    }
}
