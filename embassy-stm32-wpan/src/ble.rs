use embassy_stm32::ipcc::Ipcc;

use crate::cmd::{CmdPacket, CmdSerial};
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
            while !LinkedListNode::is_empty(EVT_QUEUE.as_mut_ptr()) {
                let node_ptr = LinkedListNode::remove_head(EVT_QUEUE.as_mut_ptr());

                let event = node_ptr.cast();
                let event = EvtBox::new(event);

                EVT_CHANNEL.try_send(event).unwrap();
            }
        }

        Ipcc::c1_clear_flag_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL);
    }

    pub(super) fn acl_data_handler() {
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL, false);

        // TODO: ACL data ack to the user
    }

    pub fn ble_send_cmd(buf: &[u8]) {
        debug!("writing ble cmd");

        unsafe {
            let pcmd_buffer: *mut CmdPacket = (*TL_REF_TABLE.assume_init().ble_table).pcmd_buffer;
            let pcmd_serial: *mut CmdSerial = &mut (*pcmd_buffer).cmdserial;
            let pcmd_serial_buf: *mut u8 = pcmd_serial.cast();

            core::ptr::copy(buf.as_ptr(), pcmd_serial_buf, buf.len());

            let cmd_packet = &mut *(*TL_REF_TABLE.assume_init().ble_table).pcmd_buffer;
            cmd_packet.cmdserial.ty = TlPacketType::BleCmd as u8;
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
