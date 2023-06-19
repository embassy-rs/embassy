use core::marker::PhantomData;

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::EvtBox;
use crate::shci::{ShciBleInitCmdParam, SCHI_OPCODE_BLE_INIT};
use crate::tables::{SysTable, WirelessFwInfoTable};
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, Ipcc, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_DEVICE_INFO_TABLE, TL_SYS_TABLE};

pub struct Sys {
    phantom: PhantomData<Sys>,
}

impl Sys {
    /// TL_Sys_Init
    pub(crate) fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE.as_mut_ptr().write_volatile(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }

        Self { phantom: PhantomData }
    }

    /// Returns CPU2 wireless firmware information (if present).
    pub fn wireless_fw_info(&self) -> Option<WirelessFwInfoTable> {
        let info = unsafe { TL_DEVICE_INFO_TABLE.as_mut_ptr().read_volatile().wireless_fw_info_table };

        // Zero version indicates that CPU2 wasn't active and didn't fill the information table
        if info.version != 0 {
            Some(info)
        } else {
            None
        }
    }

    pub fn write(&self, opcode: u16, payload: &[u8]) {
        unsafe {
            CmdPacket::write_into(SYS_CMD_BUF.as_mut_ptr(), TlPacketType::SysCmd, opcode, payload);
        }
    }

    pub async fn shci_c2_ble_init(&self, param: ShciBleInitCmdParam) {
        debug!("sending SHCI");

        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || {
            self.write(SCHI_OPCODE_BLE_INIT, param.payload());
        })
        .await;

        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;
    }

    /// `HW_IPCC_SYS_EvtNot`
    pub async fn read(&self) -> EvtBox {
        Ipcc::receive(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, || unsafe {
            if let Some(node_ptr) = LinkedListNode::remove_head(SYSTEM_EVT_QUEUE.as_mut_ptr()) {
                Some(EvtBox::new(node_ptr.cast()))
            } else {
                None
            }
        })
        .await
    }
}
