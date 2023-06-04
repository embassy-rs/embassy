use core::mem::MaybeUninit;
use core::{mem, slice};

use atomic_polyfill::{compiler_fence, Ordering};
use bit_field::BitField;
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

use self::ble::BleSubsystem;
use self::cmd::{AclDataPacket, CommandPacket};
use self::evt::CommandStatusEvent;
pub use self::ipcc::{ReceiveInterruptHandler, TransmitInterruptHandler};
use self::mac::MacSubsystem;
use self::mm::MemoryManager;
use self::shci::{ShciBleInitCmdParam, ShciBleInitCommandPacket, ShciHeader, SCHI_OPCODE_BLE_INIT};
use self::sys::SysSubsystem;
use self::thread::ThreadSubsystem;
use self::unsafe_linked_list::LinkedListNode;
use crate::interrupt;
use crate::peripherals::IPCC;
pub use crate::tl_mbox::ipcc::Config;
use crate::tl_mbox::ipcc::Ipcc;

mod ble;
mod channels;
mod cmd;
mod consts;
mod evt;
mod ipcc;
mod mac;
mod mm;
mod shci;
mod sys;
mod thread;
mod unsafe_linked_list;

pub type PacketHeader = LinkedListNode;

const TL_PACKET_HEADER_SIZE: usize = core::mem::size_of::<PacketHeader>();
const TL_EVT_HEADER_SIZE: usize = 3;
const TL_CS_EVT_SIZE: usize = core::mem::size_of::<CommandStatusEvent>();

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
    pcmd_buffer: *mut CommandPacket,
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
    pcmd_buffer: *mut CommandPacket,
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

#[link_section = "MB_MEM2"]
static mut CS_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYSTEM_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYS_CMD_BUF: MaybeUninit<CommandPacket> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut EVT_POOL: MaybeUninit<[u8; POOL_SIZE]> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYS_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut BLE_SPARE_EVT_BUF: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + 255]> =
    MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut BLE_CMD_BUFFER: MaybeUninit<CommandPacket> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
//                                            "magic" numbers from ST ---v---v
static mut HCI_ACL_DATA_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + 5 + 251]> = MaybeUninit::uninit();

pub struct TlMbox<'d> {
    _ipcc: PeripheralRef<'d, IPCC>,
    pub ble_subsystem: BleSubsystem,
    pub sys_subsystem: SysSubsystem,
    pub thread_subsystem: ThreadSubsystem,
    pub mac_subsystem: MacSubsystem,
    // LldTests,
    // Mac802_15_4,
    // Zigbee,
    // Traces,
}

impl<'d> TlMbox<'d> {
    /// initializes low-level transport between CPU1 and BLE stack on CPU2
    pub fn new(
        ipcc: impl Peripheral<P = IPCC> + 'd,
        _irqs: impl interrupt::Binding<interrupt::IPCC_C1_RX, ReceiveInterruptHandler>
            + interrupt::Binding<interrupt::IPCC_C1_TX, TransmitInterruptHandler>,
        config: Config,
    ) -> Self {
        into_ref!(ipcc);

        unsafe {
            compiler_fence(Ordering::AcqRel);

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

            // info!("TL_REF_TABLE addr: {:x}", TL_REF_TABLE.as_ptr() as usize);

            compiler_fence(Ordering::AcqRel);

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

            compiler_fence(Ordering::AcqRel);
        }

        Ipcc::enable(config);

        // MemoryManager is special
        MemoryManager::enable();

        Self {
            _ipcc: ipcc,
            ble_subsystem: BleSubsystem::new(),
            sys_subsystem: SysSubsystem::new(),
            thread_subsystem: ThreadSubsystem::new(),
            mac_subsystem: MacSubsystem::new(),
        }
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

    pub async fn shci_ble_init(&mut self, param: ShciBleInitCmdParam) -> Result<usize, ()> {
        let mut packet = ShciBleInitCommandPacket {
            header: ShciHeader::default(),
            param,
        };

        let packet_ptr: *mut ShciBleInitCommandPacket = &mut packet;

        let buf = unsafe {
            let cmd_ptr: *mut CommandPacket = packet_ptr.cast();

            (*cmd_ptr).cmd_serial.cmd.cmd_code = SCHI_OPCODE_BLE_INIT;
            (*cmd_ptr).cmd_serial.cmd.payload_len = mem::size_of::<ShciBleInitCmdParam>() as u8;

            slice::from_raw_parts(cmd_ptr as *const u8, mem::size_of::<ShciBleInitCommandPacket>())
        };

        self.sys_subsystem.write(buf).await
    }
}
