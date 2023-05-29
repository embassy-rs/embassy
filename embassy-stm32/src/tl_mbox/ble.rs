use super::channels;
use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::EventBox;
use super::tables::{BleTable, BLE_CMD_BUFFER, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE, TL_REF_TABLE};
use super::unsafe_linked_list::LinkedListNode;
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

    pub async fn read(&mut self) -> Result<EventBox, ()> {
        Ipcc::receive(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, || unsafe {
            EventBox::from_node((*TL_REF_TABLE.as_ptr().read_volatile().ble_table).pevt_queue as *mut _)
        })
        .await
    }

    /// TL_BLE_SendCmd
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        Ipcc::send(channels::cpu1::IPCC_BLE_CMD_CHANNEL, || unsafe {
            // ((TL_CommandPacket_t*)(TL_RefTable.p_ble_table->pcmd_buffer))->cmdserial.type = TL_BLECMD_PKT_TYPE;
            CommandPacket::copy_into_packet_from_slice(
                (*TL_REF_TABLE.as_ptr().read_volatile().ble_table).pcmd_buffer,
                buf,
                TlPacketType::BleCmd,
            );
        })
        .await;

        Ok(buf.len())
    }
}
