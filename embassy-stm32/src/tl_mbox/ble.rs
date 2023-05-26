use core::mem::MaybeUninit;

use embassy_futures::block_on;

use super::cmd::CmdSerial;
use super::consts::TlPacketType;
use super::evt::EvtBox;
use super::unsafe_linked_list::LinkedListNode;
use super::{
    channels, BleTable, BLE_CMD_BUFFER, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE, TL_CHANNEL,
    TL_REF_TABLE,
};
use crate::ipcc::Ipcc;
use crate::tl_mbox::cmd::CmdPacket;

pub struct Ble;

impl Ble {
    pub(crate) fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE = MaybeUninit::new(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_mut_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Ipcc::c1_set_rx_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, true);

        Ble
    }

    pub(crate) fn evt_handler() {
        unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let node_ptr_ptr: *mut _ = &mut node_ptr;

            while !LinkedListNode::is_empty(EVT_QUEUE.as_mut_ptr()) {
                LinkedListNode::remove_head(EVT_QUEUE.as_mut_ptr(), node_ptr_ptr);

                let event = node_ptr.cast();
                let event = EvtBox::new(event);

                block_on(TL_CHANNEL.send(event));
            }
        }

        Ipcc::c1_clear_flag_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL);
    }

    pub(crate) fn send_cmd(buf: &[u8]) {
        unsafe {
            let pcmd_buffer: *mut CmdPacket = (*TL_REF_TABLE.assume_init().ble_table).pcmd_buffer;
            let pcmd_serial: *mut CmdSerial = &mut (*pcmd_buffer).cmd_serial;
            let pcmd_serial_buf: *mut u8 = pcmd_serial.cast();

            core::ptr::copy(buf.as_ptr(), pcmd_serial_buf, buf.len());

            let cmd_packet = &mut *(*TL_REF_TABLE.assume_init().ble_table).pcmd_buffer;
            cmd_packet.cmd_serial.ty = TlPacketType::BleCmd as u8;
        }

        Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_BLE_CMD_CHANNEL);
    }

    pub(crate) fn send_acl_data() {
        unsafe {
            (*(*TL_REF_TABLE.assume_init().ble_table).phci_acl_data_buffer)
                .acl_data_serial
                .ty = TlPacketType::AclData as u8;
        }

        Ipcc::c1_set_flag_channel(channels::Cpu1Channel::HciAclData.into());
        Ipcc::c1_set_tx_channel(channels::Cpu1Channel::HciAclData.into(), true);
    }
}
