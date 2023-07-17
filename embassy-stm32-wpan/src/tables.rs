use core::mem::MaybeUninit;

use aligned::{Aligned, A4};
use bit_field::BitField;

use crate::cmd::{AclDataPacket, CmdPacket};
#[cfg(feature = "mac")]
use crate::consts::C_SIZE_CMD_STRING;
use crate::consts::{POOL_SIZE, TL_CS_EVT_SIZE, TL_EVT_HEADER_SIZE, TL_PACKET_HEADER_SIZE};
use crate::unsafe_linked_list::LinkedListNode;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct SafeBootInfoTable {
    version: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct RssInfoTable {
    pub version: u32,
    pub memory_size: u32,
    pub rss_info: u32,
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
    pub version: u32,
    pub memory_size: u32,
    pub thread_info: u32,
    pub ble_info: u32,
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
#[repr(C)]
pub struct DeviceInfoTable {
    pub safe_boot_info_table: SafeBootInfoTable,
    pub rss_info_table: RssInfoTable,
    pub wireless_fw_info_table: WirelessFwInfoTable,
}

#[derive(Debug)]
#[repr(C)]
pub struct BleTable {
    pub pcmd_buffer: *mut CmdPacket,
    pub pcs_buffer: *const u8,
    pub pevt_queue: *const u8,
    pub phci_acl_data_buffer: *mut AclDataPacket,
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadTable {
    pub nostack_buffer: *const u8,
    pub clicmdrsp_buffer: *const u8,
    pub otcmdrsp_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct LldTestsTable {
    pub clicmdrsp_buffer: *const u8,
    pub m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C)]
pub struct BleLldTable {
    pub cmdrsp_buffer: *const u8,
    pub m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C)]
pub struct ZigbeeTable {
    pub notif_m0_to_m4_buffer: *const u8,
    pub appli_cmd_m4_to_m0_bufer: *const u8,
    pub request_m0_to_m4_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct SysTable {
    pub pcmd_buffer: *mut CmdPacket,
    pub sys_queue: *const LinkedListNode,
}

#[derive(Debug)]
#[repr(C)]
pub struct MemManagerTable {
    pub spare_ble_buffer: *const u8,
    pub spare_sys_buffer: *const u8,

    pub blepool: *const u8,
    pub blepoolsize: u32,

    pub pevt_free_buffer_queue: *mut LinkedListNode,

    pub traces_evt_pool: *const u8,
    pub tracespoolsize: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct TracesTable {
    pub traces_queue: *const u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Mac802_15_4Table {
    pub p_cmdrsp_buffer: *const u8,
    pub p_notack_buffer: *const u8,
    pub evt_queue: *const u8,
}

/// Reference table. Contains pointers to all other tables.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RefTable {
    pub device_info_table: *const DeviceInfoTable,
    pub ble_table: *const BleTable,
    pub thread_table: *const ThreadTable,
    pub sys_table: *const SysTable,
    pub mem_manager_table: *const MemManagerTable,
    pub traces_table: *const TracesTable,
    pub mac_802_15_4_table: *const Mac802_15_4Table,
    pub zigbee_table: *const ZigbeeTable,
    pub lld_tests_table: *const LldTestsTable,
    pub ble_lld_table: *const BleLldTable,
}

// --------------------- ref table ---------------------
#[link_section = "TL_REF_TABLE"]
pub static mut TL_REF_TABLE: MaybeUninit<RefTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
pub static mut TL_DEVICE_INFO_TABLE: Aligned<A4, MaybeUninit<DeviceInfoTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_BLE_TABLE: Aligned<A4, MaybeUninit<BleTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_THREAD_TABLE: Aligned<A4, MaybeUninit<ThreadTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_LLD_TESTS_TABLE: Aligned<A4, MaybeUninit<LldTestsTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_BLE_LLD_TABLE: Aligned<A4, MaybeUninit<BleLldTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_SYS_TABLE: Aligned<A4, MaybeUninit<SysTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_MEM_MANAGER_TABLE: Aligned<A4, MaybeUninit<MemManagerTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_TRACES_TABLE: Aligned<A4, MaybeUninit<TracesTable>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_MAC_802_15_4_TABLE: Aligned<A4, MaybeUninit<Mac802_15_4Table>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM1"]
pub static mut TL_ZIGBEE_TABLE: Aligned<A4, MaybeUninit<ZigbeeTable>> = Aligned(MaybeUninit::uninit());

// --------------------- tables ---------------------
#[link_section = "MB_MEM1"]
pub static mut FREE_BUF_QUEUE: Aligned<A4, MaybeUninit<LinkedListNode>> = Aligned(MaybeUninit::uninit());

#[allow(dead_code)]
#[link_section = "MB_MEM1"]
pub static mut TRACES_EVT_QUEUE: Aligned<A4, MaybeUninit<LinkedListNode>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM2"]
pub static mut CS_BUFFER: Aligned<A4, MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]>> =
    Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM2"]
pub static mut EVT_QUEUE: Aligned<A4, MaybeUninit<LinkedListNode>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM2"]
pub static mut SYSTEM_EVT_QUEUE: Aligned<A4, MaybeUninit<LinkedListNode>> = Aligned(MaybeUninit::uninit());

// --------------------- app tables ---------------------
#[cfg(feature = "mac")]
#[link_section = "MB_MEM2"]
pub static mut MAC_802_15_4_CMD_BUFFER: Aligned<A4, MaybeUninit<CmdPacket>> = Aligned(MaybeUninit::uninit());

#[cfg(feature = "mac")]
#[link_section = "MB_MEM2"]
pub static mut MAC_802_15_4_NOTIF_RSP_EVT_BUFFER: MaybeUninit<
    Aligned<A4, [u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]>,
> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
pub static mut EVT_POOL: Aligned<A4, MaybeUninit<[u8; POOL_SIZE]>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM2"]
pub static mut SYS_CMD_BUF: Aligned<A4, MaybeUninit<CmdPacket>> = Aligned(MaybeUninit::uninit());

#[link_section = "MB_MEM2"]
pub static mut SYS_SPARE_EVT_BUF: Aligned<A4, MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]>> =
    Aligned(MaybeUninit::uninit());

#[cfg(feature = "mac")]
#[link_section = "MB_MEM2"]
pub static mut MAC_802_15_4_CNFINDNOT: Aligned<A4, MaybeUninit<[u8; C_SIZE_CMD_STRING]>> =
    Aligned(MaybeUninit::uninit());

#[cfg(feature = "ble")]
#[link_section = "MB_MEM1"]
pub static mut BLE_CMD_BUFFER: Aligned<A4, MaybeUninit<CmdPacket>> = Aligned(MaybeUninit::uninit());

#[cfg(feature = "ble")]
#[link_section = "MB_MEM2"]
pub static mut BLE_SPARE_EVT_BUF: Aligned<A4, MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]>> =
    Aligned(MaybeUninit::uninit());

#[cfg(feature = "ble")]
#[link_section = "MB_MEM2"]
//                                 fuck these "magic" numbers from ST ---v---v
pub static mut HCI_ACL_DATA_BUFFER: Aligned<A4, MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + 5 + 251]>> =
    Aligned(MaybeUninit::uninit());
