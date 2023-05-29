use core::mem::MaybeUninit;
use core::{mem, slice};

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

use self::ble::BleSubsystem;
use self::cmd::CommandPacket;
pub use self::ipcc::{ReceiveInterruptHandler, TransmitInterruptHandler};
use self::mac::MacSubsystem;
use self::mm::MemoryManager;
use self::shci::{ShciBleInitCmdParam, ShciBleInitCommandPacket, ShciHeader, SCHI_OPCODE_BLE_INIT};
use self::sys::SysSubsystem;
use self::tables::WirelessFwInfoTable;
use self::thread::ThreadSubsystem;
use self::unsafe_linked_list::LinkedListNode;
use crate::interrupt;
use crate::peripherals::IPCC;
pub use crate::tl_mbox::ipcc::Config;
use crate::tl_mbox::ipcc::Ipcc;
use crate::tl_mbox::tables::{
    RefTable, BLE_CMD_BUFFER, BLE_SPARE_EVT_BUF, CS_BUFFER, EVT_POOL, HCI_ACL_DATA_BUFFER, SYS_SPARE_EVT_BUF,
    TL_BLE_LLD_TABLE, TL_BLE_TABLE, TL_DEVICE_INFO_TABLE, TL_LLD_TESTS_TABLE, TL_MAC_802_15_4_TABLE,
    TL_MEM_MANAGER_TABLE, TL_REF_TABLE, TL_SYS_TABLE, TL_THREAD_TABLE, TL_TRACES_TABLE, TL_ZIGBEE_TABLE,
};

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
mod tables;
mod thread;
mod unsafe_linked_list;

pub type PacketHeader = LinkedListNode;

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
