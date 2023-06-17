use core::mem::MaybeUninit;

use embassy_stm32::ipcc::Ipcc;

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::EvtBox;
use crate::tables::BleTable;
use crate::unsafe_linked_list::LinkedListNode;
use crate::{
    channels, BLE_CMD_BUFFER, CS_BUFFER, EVT_CHANNEL, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE, TL_REF_TABLE,
};

pub struct Ble;

impl Ble {
    pub(super) fn enable() {
        unsafe {
            // Ensure reproducible behavior
            BLE_CMD_BUFFER
                .as_mut_ptr()
                .write_volatile(MaybeUninit::zeroed().assume_init());

            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE.as_mut_ptr().write_volatile(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Ipcc::c1_set_rx_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, true);
    }

    pub(super) fn evt_handler() {
        unsafe {
            while let Some(node_ptr) = LinkedListNode::remove_head(EVT_QUEUE.as_mut_ptr()) {
                let event = EvtBox::new(node_ptr.cast());

                EVT_CHANNEL.try_send(event).unwrap();
            }
        }

        Ipcc::c1_clear_flag_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL);
    }

    pub(super) fn acl_data_handler() {
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL, false);

        // TODO: ACL data ack to the user
    }

    pub fn send_cmd(opcode: u16, payload: &[u8]) {
        debug!("writing ble cmd");

        unsafe {
            CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
        }

        Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_BLE_CMD_CHANNEL);
    }

    #[allow(dead_code)] // Not used currently but reserved
    pub(super) fn ble_send_acl_data() {
        let cmd_packet = unsafe { &mut *(*TL_REF_TABLE.assume_init().ble_table).phci_acl_data_buffer };

        cmd_packet.acl_data_serial.ty = TlPacketType::AclData as u8;

        Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL);
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL, true);
    }
}
