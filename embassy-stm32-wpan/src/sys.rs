use core::mem::MaybeUninit;
use core::{mem, ptr};

use crate::cmd::{CmdPacket, CmdSerialStub};
use crate::consts::TlPacketType;
use crate::evt::{CcEvt, EvtBox, EvtPacket, EvtSerial};
use crate::shci::{ShciBleInitCmdPacket, ShciBleInitCmdParam, ShciHeader, SCHI_OPCODE_BLE_INIT};
use crate::tables::SysTable;
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, mm, Ipcc, EVT_CHANNEL, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_SYS_TABLE};

pub struct Sys;

impl Sys {
    /// TL_Sys_Init
    pub fn enable() {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE.as_mut_ptr().write_volatile(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }
    }

    //    pub async fn shci_c2_ble_init(&mut self, param: ShciBleInitCmdParam) -> SchiCommandStatus {
    //        let command_event = self
    //            .write_and_get_response(TlPacketType::SysCmd, ShciOpcode::BleInit as u16, param.payload())
    //            .await;
    //
    //        let payload = command_event.payload[0];
    //        // info!("payload: {:x}", payload);
    //
    //        payload.try_into().unwrap()
    //    }

    pub fn write(opcode: u16, payload: &[u8]) {
        unsafe {
            CmdPacket::write_into(SYS_CMD_BUF.as_mut_ptr(), TlPacketType::SysCmd, opcode, payload);
        }
    }

    pub async fn shci_c2_ble_init(param: ShciBleInitCmdParam) {
        debug!("sending SHCI");

        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || {
            Self::write(SCHI_OPCODE_BLE_INIT, param.payload());
        })
        .await;

        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;
    }

    /// `HW_IPCC_SYS_EvtNot`
    pub async fn read() -> EvtBox {
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
