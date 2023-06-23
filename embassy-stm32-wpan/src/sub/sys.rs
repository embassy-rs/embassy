use core::marker::PhantomData;
use core::ptr;

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::{CcEvt, EvtBox, EvtPacket};
#[allow(unused_imports)]
use crate::shci::{SchiCommandStatus, ShciBleInitCmdParam, ShciOpcode};
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

    pub async fn write(&self, opcode: ShciOpcode, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || unsafe {
            CmdPacket::write_into(SYS_CMD_BUF.as_mut_ptr(), TlPacketType::SysCmd, opcode as u16, payload);
        })
        .await;
    }

    /// `HW_IPCC_SYS_CmdEvtNot`
    pub async fn write_and_get_response(&self, opcode: ShciOpcode, payload: &[u8]) -> SchiCommandStatus {
        self.write(opcode, payload).await;
        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;

        unsafe {
            let p_event_packet = SYS_CMD_BUF.as_ptr() as *const EvtPacket;
            let p_command_event = &((*p_event_packet).evt_serial.evt.payload) as *const _ as *const CcEvt;
            let p_payload = &((*p_command_event).payload) as *const u8;

            ptr::read_volatile(p_payload).try_into().unwrap()
        }
    }

    #[cfg(feature = "mac")]
    pub async fn shci_c2_mac_802_15_4_init(&self) -> SchiCommandStatus {
        self.write_and_get_response(ShciOpcode::Mac802_15_4Init, &[]).await
    }

    #[cfg(feature = "ble")]
    pub async fn shci_c2_ble_init(&self, param: ShciBleInitCmdParam) -> SchiCommandStatus {
        self.write_and_get_response(ShciOpcode::BleInit, param.payload()).await
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
