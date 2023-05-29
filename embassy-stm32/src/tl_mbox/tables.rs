use core::mem::MaybeUninit;

use bit_field::BitField;

use super::cmd::{AclDataPacket, CommandPacket};
use super::consts::{POOL_SIZE, TL_CS_EVT_SIZE, TL_EVT_HEADER_SIZE, TL_PACKET_HEADER_SIZE};
use super::unsafe_linked_list::LinkedListNode;

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
    pub version: u32,
    pub memory_size: u32,
    pub info_stack: u32,
    pub reserved: u32,
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
pub struct BleTable {
    pub pcmd_buffer: *mut CommandPacket,
    pub pcs_buffer: *const u8,
    pub pevt_queue: *const u8,
    pub phci_acl_data_buffer: *mut AclDataPacket,
}

#[repr(C, packed)]
pub struct ThreadTable {
    pub no_stack_buffer: *const u8,
    pub cli_cmd_rsp_buffer: *const u8,
    pub ot_cmd_rsp_buffer: *const u8,
}

#[repr(C, packed)]
pub struct SysTable {
    pub pcmd_buffer: *mut CommandPacket,
    pub sys_queue: *const LinkedListNode,
}

// Not used currently but reserved
#[repr(C, packed)]
pub struct LldTestTable {
    pub cli_cmd_rsp_buffer: *const u8,
    pub m0_cmd_buffer: *const u8,
}

// Not used currently but reserved
#[repr(C, packed)]
pub struct BleLldTable {
    pub cmd_rsp_buffer: *const u8,
    pub m0_cmd_buffer: *const u8,
}

// Not used currently but reserved
#[repr(C, packed)]
pub struct ZigbeeTable {
    pub notif_m0_to_m4_buffer: *const u8,
    pub appli_cmd_m4_to_m0_buffer: *const u8,
    pub request_m0_to_m4_buffer: *const u8,
}

#[repr(C, packed)]
pub struct MemManagerTable {
    pub spare_ble_buffer: *const u8,
    pub spare_sys_buffer: *const u8,

    pub ble_pool: *const u8,
    pub ble_pool_size: u32,

    pub pevt_free_buffer_queue: *mut LinkedListNode,

    pub traces_evt_pool: *const u8,
    pub traces_pool_size: u32,
}

#[repr(C, packed)]
pub struct TracesTable {
    pub traces_queue: *const u8,
}

#[repr(C, packed)]
pub struct Mac802_15_4Table {
    pub pcmd_rsp_buffer: *mut u8,
    pub pnotack_buffer: *mut u8,
    pub evt_queue: *mut u8,
}

/// reference table. Contains pointers to all other tables
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct RefTable {
    pub device_info_table: *const DeviceInfoTable,
    pub ble_table: *const BleTable,
    pub thread_table: *const ThreadTable,
    pub sys_table: *const SysTable,
    pub mem_manager_table: *const MemManagerTable,
    pub traces_table: *const TracesTable,
    pub mac_802_15_4_table: *const Mac802_15_4Table,
    pub zigbee_table: *const ZigbeeTable,
    pub lld_tests_table: *const LldTestTable,
    pub ble_lld_table: *const BleLldTable,
}

// --------------------- ref table ---------------------
#[link_section = "TL_REF_TABLE"]
pub static mut TL_REF_TABLE: MaybeUninit<RefTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_DEVICE_INFO_TABLE: MaybeUninit<DeviceInfoTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_BLE_TABLE: MaybeUninit<BleTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_THREAD_TABLE: MaybeUninit<ThreadTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_LLD_TESTS_TABLE: MaybeUninit<LldTestTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_BLE_LLD_TABLE: MaybeUninit<BleLldTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_SYS_TABLE: MaybeUninit<SysTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_MEM_MANAGER_TABLE: MaybeUninit<MemManagerTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_TRACES_TABLE: MaybeUninit<TracesTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_MAC_802_15_4_TABLE: MaybeUninit<Mac802_15_4Table> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_ZIGBEE_TABLE: MaybeUninit<ZigbeeTable> = MaybeUninit::uninit();

// --------------------- tables ---------------------
#[link_section = "MB_MEM1"]
pub static mut FREE_BUFF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut _TRACES_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut CS_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut SYSTEM_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

// --------------------- app tables ---------------------
#[link_section = "MB_MEM2"]
pub static mut EVT_POOL: MaybeUninit<[u8; POOL_SIZE]> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut SYS_CMD_BUF: MaybeUninit<CommandPacket> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut SYS_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut BLE_CMD_BUFFER: MaybeUninit<CommandPacket> = MaybeUninit::uninit();

// not in shared RAM
pub static mut LOCAL_FREE_BUF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut BLE_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
//                                            "magic" numbers from ST -------v---v
pub static mut HCI_ACL_DATA_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + 5 + 251]> = MaybeUninit::uninit();
