use core::mem::MaybeUninit;
#[allow(unused_imports)]
use core::{mem, slice};

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

#[cfg(feature = "ble")]
use self::ble::BleSubsystem;
#[allow(unused_imports)]
use self::cmd::{CommandPacket, CommandSerial};
pub use self::ipcc::{ReceiveInterruptHandler, TransmitInterruptHandler};
#[cfg(feature = "mac")]
use self::mac::MacSubsystem;
use self::mm::MemoryManager;
#[allow(unused_imports)]
use self::shci::{ShciBleInitCmdParam, ShciOpcode};
use self::sys::SysSubsystem;
use self::tables::WirelessFwInfoTable;
use self::thread::ThreadSubsystem;
use self::unsafe_linked_list::LinkedListNode;
use crate::interrupt;
use crate::peripherals::IPCC;
pub use crate::tl_mbox::ipcc::Config;
use crate::tl_mbox::ipcc::Ipcc;
use crate::tl_mbox::tables::{
    RefTable, BLE_SPARE_EVT_BUF, EVT_POOL, SYS_SPARE_EVT_BUF, TL_BLE_LLD_TABLE, TL_BLE_TABLE, TL_DEVICE_INFO_TABLE,
    TL_LLD_TESTS_TABLE, TL_MAC_802_15_4_TABLE, TL_MEM_MANAGER_TABLE, TL_REF_TABLE, TL_SYS_TABLE, TL_THREAD_TABLE,
    TL_TRACES_TABLE, TL_ZIGBEE_TABLE,
};
#[cfg(feature = "ble")]
use crate::tl_mbox::tables::{BLE_CMD_BUFFER, CS_BUFFER, HCI_ACL_DATA_BUFFER};
#[cfg(feature = "mac")]
use crate::tl_mbox::tables::{MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER};

#[cfg(feature = "ble")]
pub mod ble;
mod channels;
pub mod cmd;
pub mod consts;
mod evt;
mod ipcc;
#[cfg(feature = "mac")]
mod mac;
mod mm;
pub mod shci;
pub mod sys;
pub mod tables;
mod thread;
mod unsafe_linked_list;

pub type PacketHeader = LinkedListNode;

pub struct TlMbox<'d> {
    _ipcc: PeripheralRef<'d, IPCC>,
    #[cfg(feature = "ble")]
    pub ble_subsystem: BleSubsystem,
    pub sys_subsystem: SysSubsystem,
    pub thread_subsystem: ThreadSubsystem,
    #[cfg(feature = "mac")]
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
            BLE_SPARE_EVT_BUF
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());

            #[cfg(feature = "ble")]
            {
                BLE_CMD_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
                HCI_ACL_DATA_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
                CS_BUFFER
                    .as_mut_ptr()
                    .write_volatile(MaybeUninit::zeroed().assume_init());
            }

            #[cfg(feature = "mac")]
            {
                MAC_802_15_4_CMD_BUFFER = MaybeUninit::zeroed();
                MAC_802_15_4_NOTIF_RSP_EVT_BUFFER = MaybeUninit::zeroed();
            }
        }

        compiler_fence(Ordering::SeqCst);

        // MemoryManager is special
        MemoryManager::enable();

        #[cfg(feature = "ble")]
        let ble_subsystem = BleSubsystem::new();
        let sys_subsystem = SysSubsystem::new();
        let thread_subsystem = ThreadSubsystem::new();
        #[cfg(feature = "mac")]
        let mac_subsystem = MacSubsystem::new();

        compiler_fence(Ordering::SeqCst);

        Ipcc::enable(config);

        Self {
            _ipcc: ipcc,
            #[cfg(feature = "ble")]
            ble_subsystem: ble_subsystem,
            sys_subsystem: sys_subsystem,
            thread_subsystem: thread_subsystem,
            #[cfg(feature = "mac")]
            mac_subsystem: mac_subsystem,
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

    //    #[cfg(feature = "mac")]
    //    pub async fn mac_802_15_4_init(&mut self) -> Result<usize, ()> {
    //        let mut cmd_serial = CommandSerial::default();
    //        cmd_serial.cmd.cmd_code = ShciOpcode::Mac802_15_4DeInit as u16;
    //        cmd_serial.cmd.payload_len = 0;
    //
    //        let buf = unsafe {
    //            core::slice::from_raw_parts(
    //                (&cmd_serial as *const _) as *const u8,
    //                core::mem::size_of::<CommandSerial>(),
    //            )
    //        };
    //
    //        self.sys_subsystem.write(buf).await
    //    }
    //    #[cfg(feature = "ble")]
    //    pub async fn ble_init(&mut self, param: ShciBleInitCmdParam) -> Result<usize, ()> {
    //        let payload_len = mem::size_of::<ShciBleInitCmdParam>();
    //
    //        let mut cmd_serial = CommandSerial::default();
    //        cmd_serial.cmd.cmd_code = ShciOpcode::BleInit as u16;
    //        cmd_serial.cmd.payload_len = payload_len as u8;
    //
    //        let payload = unsafe { slice::from_raw_parts((&param as *const _) as *const u8, payload_len) };
    //
    //        cmd_serial.cmd.payload[..payload.len()].copy_from_slice(payload);
    //
    //        let buf =
    //            unsafe { slice::from_raw_parts((&cmd_serial as *const _) as *const u8, mem::size_of::<CommandSerial>()) };
    //
    //        self.sys_subsystem.write(buf).await
    //    }

    //    pub async fn shci_ble_init(&mut self, param: ShciBleInitCmdParam) -> Result<usize, ()> {
    //        let mut packet = ShciBleInitCommandPacket {
    //            header: ShciHeader::default(),
    //            param,
    //        };
    //
    //        let packet_ptr: *mut ShciBleInitCommandPacket = &mut packet;
    //
    //        let buf = unsafe {
    //            let cmd_ptr: *mut CommandPacket = packet_ptr.cast();
    //
    //            (*cmd_ptr).cmd_serial.cmd.cmd_code = SCHI_OPCODE_BLE_INIT;
    //            (*cmd_ptr).cmd_serial.cmd.payload_len = mem::size_of::<ShciBleInitCmdParam>() as u8;
    //
    //            slice::from_raw_parts(cmd_ptr as *const u8, mem::size_of::<ShciBleInitCommandPacket>())
    //        };
    //
    //        self.sys_subsystem.write(buf).await
    //    }
}
