use core::mem::MaybeUninit;

use bit_field::BitField;

use self::ble::Ble;
use self::cmd::{AclDataPacket, CmdPacket};
use self::evt::CsEvt;
use self::mm::MemoryManager;
use self::sys::Sys;
use self::unsafe_linked_list::LinkedListNode;
use crate::ipcc::Ipcc;

mod ble;
mod channels;
mod cmd;
mod evt;
mod mm;
mod sys;
mod unsafe_linked_list;

pub type PacketHeader = LinkedListNode;

const TL_PACKET_HEADER_SIZE: usize = core::mem::size_of::<PacketHeader>();
const TL_EVT_HEADER_SIZE: usize = 3;
const TL_CS_EVT_SIZE: usize = core::mem::size_of::<CsEvt>();

const CFG_TL_BLE_EVT_QUEUE_LENGTH: usize = 5;
const CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE: usize = 255;
const TL_BLE_EVENT_FRAME_SIZE: usize = TL_EVT_HEADER_SIZE + CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE;

const POOL_SIZE: usize = CFG_TL_BLE_EVT_QUEUE_LENGTH * 4 * divc(TL_PACKET_HEADER_SIZE + TL_BLE_EVENT_FRAME_SIZE, 4);

const fn divc(x: usize, y: usize) -> usize {
    (x + y - 1) / y
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SafeBootInfoTable {
    version: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FusInfoTable {
    version: u32,
    memory_size: u32,
    fus_info: u32,
}

/// # Version
/// - 0 -> 3   = Build - 0: Untracked - 15:Released - x: Tracked version
/// - 4 -> 7   = branch - 0: Mass Market - x: ...
/// - 8 -> 15  = Subversion
/// - 16 -> 23 = Version minor
/// - 24 -> 31 = Version major
/// # Memory Size
/// - 0 -> 7   = Flash ( Number of 4k sector)
/// - 8 -> 15  = Reserved ( Shall be set to 0 - may be used as flash extension )
/// - 16 -> 23 = SRAM2b ( Number of 1k sector)
/// - 24 -> 31 = SRAM2a ( Number of 1k sector)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct WirelessFwInfoTable {
    version: u32,
    memory_size: u32,
    info_stack: u32,
    reserved: u32,
}

impl WirelessFwInfoTable {
    pub fn version_major(&self) -> u8 {
        let version = self.version;
        (version.get_bits(24..31) & 0xff) as u8
    }

    pub fn version_minor(&self) -> u8 {
        let version = self.version;
        (version.get_bits(16..23) & 0xff) as u8
    }

    pub fn subversion(&self) -> u8 {
        let version = self.version;
        (version.get_bits(8..15) & 0xff) as u8
    }

    /// size of FLASH, expressed in number of 4K sectors
    pub fn flash_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.get_bits(0..7) & 0xff) as u8
    }

    /// size for SRAM2a, expressed in number of 1K sectors
    pub fn sram2a_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.get_bits(24..31) & 0xff) as u8
    }

    /// size of SRAM2b, expressed in number of 1K sectors
    pub fn sram2b_size(&self) -> u8 {
        let memory_size = self.memory_size;
        (memory_size.get_bits(16..23) & 0xff) as u8
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DeviceInfoTable {
    pub safe_boot_info_table: SafeBootInfoTable,
    pub fus_info_table: FusInfoTable,
    pub wireless_fw_info_table: WirelessFwInfoTable,
}

#[repr(C, packed)]
struct BleTable {
    pcmd_buffer: *mut CmdPacket,
    pcs_buffer: *const u8,
    pevt_queue: *const u8,
    phci_acl_data_buffer: *mut AclDataPacket,
}

#[repr(C, packed)]
struct ThreadTable {
    no_stack_buffer: *const u8,
    cli_cmd_rsp_buffer: *const u8,
    ot_cmd_rsp_buffer: *const u8,
}

#[repr(C, packed)]
struct SysTable {
    pcmd_buffer: *mut CmdPacket,
    sys_queue: *const LinkedListNode,
}

#[allow(dead_code)] // Not used currently but reserved
#[repr(C, packed)]
struct LldTestTable {
    cli_cmd_rsp_buffer: *const u8,
    m0_cmd_buffer: *const u8,
}

#[allow(dead_code)] // Not used currently but reserved
#[repr(C, packed)]
struct BleLldTable {
    cmd_rsp_buffer: *const u8,
    m0_cmd_buffer: *const u8,
}

#[allow(dead_code)] // Not used currently but reserved
#[repr(C, packed)]
struct ZigbeeTable {
    notif_m0_to_m4_buffer: *const u8,
    appli_cmd_m4_to_m0_buffer: *const u8,
    request_m0_to_m4_buffer: *const u8,
}

#[repr(C, packed)]
struct MemManagerTable {
    spare_ble_buffer: *const u8,
    spare_sys_buffer: *const u8,

    ble_pool: *const u8,
    ble_pool_size: u32,

    pevt_free_buffer_queue: *mut LinkedListNode,

    traces_evt_pool: *const u8,
    traces_pool_size: u32,
}

#[repr(C, packed)]
struct TracesTable {
    traces_queue: *const u8,
}

#[repr(C, packed)]
struct Mac802_15_4Table {
    pcmd_rsp_buffer: *const u8,
    pnotack_buffer: *const u8,
    evt_queue: *const u8,
}

/// reference table. Contains pointers to all other tables
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct RefTable {
    pub device_info_table: *const DeviceInfoTable,
    ble_table: *const BleTable,
    thread_table: *const ThreadTable,
    sys_table: *const SysTable,
    mem_manager_table: *const MemManagerTable,
    traces_table: *const TracesTable,
    mac_802_15_4_table: *const Mac802_15_4Table,
    zigbee_table: *const ZigbeeTable,
    lld_tests_table: *const LldTestTable,
    ble_lld_table: *const BleLldTable,
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
static mut TL_LLD_TESTS_TABLE: MaybeUninit<LldTestTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_BLE_LLD_TABLE: MaybeUninit<BleLldTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_SYS_TABLE: MaybeUninit<SysTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MEM_MANAGER_TABLE: MaybeUninit<MemManagerTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_TRACES_TABLE: MaybeUninit<TracesTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MAC_802_15_4_TABLE: MaybeUninit<Mac802_15_4Table> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_ZIGBEE_TABLE: MaybeUninit<ZigbeeTable> = MaybeUninit::uninit();

#[allow(dead_code)] // Not used currently but reserved
#[link_section = "MB_MEM1"]
static mut FREE_BUFF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

// not in shared RAM
static mut LOCAL_FREE_BUF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[allow(dead_code)] // Not used currently but reserved
#[link_section = "MB_MEM1"]
static mut TRACES_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut CS_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut SYSTEM_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYS_CMD_BUF: MaybeUninit<CmdPacket> = MaybeUninit::uninit();

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
//                                            "magic" numbers from ST ---v---v
static mut HCI_ACL_DATA_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + 5 + 251]> = MaybeUninit::uninit();

pub struct TlMbox {
    _sys: Sys,
    _ble: Ble,
    _mm: MemoryManager,
}

impl TlMbox {
    /// initializes low-level transport between CPU1 and BLE stack on CPU2
    pub fn init(ipcc: &mut Ipcc) -> TlMbox {
        unsafe {
            TL_REF_TABLE = MaybeUninit::new(RefTable {
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

            TL_SYS_TABLE = MaybeUninit::zeroed();
            TL_DEVICE_INFO_TABLE = MaybeUninit::zeroed();
            TL_BLE_TABLE = MaybeUninit::zeroed();
            TL_THREAD_TABLE = MaybeUninit::zeroed();
            TL_MEM_MANAGER_TABLE = MaybeUninit::zeroed();
            TL_TRACES_TABLE = MaybeUninit::zeroed();
            TL_MAC_802_15_4_TABLE = MaybeUninit::zeroed();
            TL_ZIGBEE_TABLE = MaybeUninit::zeroed();
            TL_LLD_TESTS_TABLE = MaybeUninit::zeroed();
            TL_BLE_LLD_TABLE = MaybeUninit::zeroed();

            EVT_POOL = MaybeUninit::zeroed();
            SYS_SPARE_EVT_BUF = MaybeUninit::zeroed();
            BLE_SPARE_EVT_BUF = MaybeUninit::zeroed();

            CS_BUFFER = MaybeUninit::zeroed();
            BLE_CMD_BUFFER = MaybeUninit::zeroed();
            HCI_ACL_DATA_BUFFER = MaybeUninit::zeroed();
        }

        ipcc.init();

        let _sys = Sys::new(ipcc);
        let _ble = Ble::new(ipcc);
        let _mm = MemoryManager::new();

        TlMbox { _sys, _ble, _mm }
    }

    pub fn wireless_fw_info(&self) -> Option<WirelessFwInfoTable> {
        let info = unsafe { &(*(*TL_REF_TABLE.as_ptr()).device_info_table).wireless_fw_info_table };

        // zero version indicates that CPU2 wasn't active and didn't fill the information table
        if info.version != 0 {
            Some(*info)
        } else {
            None
        }
    }
}
