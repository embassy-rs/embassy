use bit_field::BitField;

use crate::cmd::{AclDataPacket, CmdPacket};
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
#[repr(C, align(4))]
pub struct DeviceInfoTable {
    pub safe_boot_info_table: SafeBootInfoTable,
    pub rss_info_table: RssInfoTable,
    pub wireless_fw_info_table: WirelessFwInfoTable,
}

#[derive(Debug)]
#[repr(C, align(4))]
pub struct BleTable {
    pub pcmd_buffer: *mut CmdPacket,
    pub pcs_buffer: *const u8,
    pub pevt_queue: *const u8,
    pub phci_acl_data_buffer: *mut AclDataPacket,
}

#[derive(Debug)]
#[repr(C, align(4))]
pub struct ThreadTable {
    pub nostack_buffer: *const u8,
    pub clicmdrsp_buffer: *const u8,
    pub otcmdrsp_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct LldTestsTable {
    pub clicmdrsp_buffer: *const u8,
    pub m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct BleLldTable {
    pub cmdrsp_buffer: *const u8,
    pub m0cmd_buffer: *const u8,
}

// TODO: use later
#[derive(Debug)]
#[repr(C, align(4))]
pub struct ZigbeeTable {
    pub notif_m0_to_m4_buffer: *const u8,
    pub appli_cmd_m4_to_m0_bufer: *const u8,
    pub request_m0_to_m4_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
pub struct SysTable {
    pub pcmd_buffer: *mut CmdPacket,
    pub sys_queue: *const LinkedListNode,
}

#[derive(Debug)]
#[repr(C, align(4))]
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
#[repr(C, align(4))]
pub struct TracesTable {
    pub traces_queue: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
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
}
