use core::mem;

use atomic_polyfill::{compiler_fence, Ordering};

use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::EventPacket;
use super::tables::{BleTable, BLE_CMD_BUFFER, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE, TL_REF_TABLE};
use super::unsafe_linked_list::LinkedListNode;
use super::{channels, mm};
use crate::tl_mbox::ipcc::Ipcc;

pub struct BleSubsystem;

impl BleSubsystem {
    /// TL_BLE_Init
    pub fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE.as_mut_ptr().write_volatile(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_mut_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Self {}
    }

    /// `HW_IPCC_BLE_EvtNot`
    pub async fn read<R>(&mut self, mut r: impl FnMut(&EventPacket) -> R) -> R {
        Ipcc::receive(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, || unsafe {
            let node = EVT_QUEUE.as_mut_ptr() as *mut LinkedListNode;
            let mut node_ptr = core::ptr::null_mut();
            if LinkedListNode::is_empty(node) {
                // `TL_MAC_802_15_4_SendAck` resets the buffer before requesting more
                let command_packet = EVT_QUEUE.as_mut_ptr() as *mut CommandPacket;
                let typ = &command_packet.read().cmd_serial.typ as *const _ as *mut u8;
                typ.write_volatile(TlPacketType::OtAck as u8);

                None
            } else {
                LinkedListNode::remove_head(node, &mut node_ptr);

                let node_ptr: *const EventPacket = node_ptr.cast();
                let ret = r(mem::transmute(node_ptr));

                compiler_fence(Ordering::SeqCst);
                mm::MemoryManager::drop_packet(node_ptr as *mut _);

                Some(ret)
            }
        })
        .await
    }

    /// `HW_IPCC_BLE_CmdEvtNot`
    pub async fn write_and_get_response<R>(
        &mut self,
        f: impl FnOnce(*mut CommandPacket),
        r: impl FnOnce(&EventPacket) -> R,
    ) -> R {
        self.write(f).await;
        Ipcc::flush(channels::cpu1::IPCC_BLE_CMD_CHANNEL).await;

        let event_packet = unsafe { BLE_CMD_BUFFER.as_ptr() as *const EventPacket };
        let event_packet = unsafe { mem::transmute(event_packet) };

        r(event_packet)
    }

    /// `TL_BLE_SendCmd`
    pub async fn write(&mut self, f: impl FnOnce(*mut CommandPacket)) {
        Ipcc::send(channels::cpu1::IPCC_BLE_CMD_CHANNEL, || unsafe {
            f(BLE_CMD_BUFFER.as_mut_ptr());
        })
        .await;
    }
    //    pub async fn read(&mut self) -> Result<EventBox, ()> {
    //        Ipcc::receive(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, || unsafe {
    //            EventBox::from_node((*TL_REF_TABLE.as_ptr().read_volatile().ble_table).pevt_queue as *mut _)
    //        })
    //        .await
    //    }
    //
    //    /// TL_BLE_SendCmd
    //    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
    //        Ipcc::send(channels::cpu1::IPCC_BLE_CMD_CHANNEL, || unsafe {
    //            // ((TL_CommandPacket_t*)(TL_RefTable.p_ble_table->pcmd_buffer))->cmdserial.type = TL_BLECMD_PKT_TYPE;
    //            CommandPacket::copy_into_packet_from_slice(
    //                (*TL_REF_TABLE.as_ptr().read_volatile().ble_table).pcmd_buffer,
    //                buf,
    //                TlPacketType::BleCmd,
    //            );
    //        })
    //        .await;
    //
    //        Ok(buf.len())
    //    }
}
