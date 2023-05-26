use core::mem::MaybeUninit;

use embassy_futures::block_on;

use super::cmd::{CmdPacket, CmdSerial};
use super::consts::TlPacketType;
use super::evt::{EvtBox, EvtPacket};
use super::unsafe_linked_list::LinkedListNode;
use super::{
    channels, Mac802_15_4Table, EVT_QUEUE, MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, TL_CHANNEL,
    TL_MAC_802_15_4_TABLE, TL_REF_TABLE,
};
use crate::ipcc::Ipcc;

pub struct Mac802_15_4;

impl Mac802_15_4 {
    pub(crate) fn init(ipcc: &mut Ipcc) -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_MAC_802_15_4_TABLE = MaybeUninit::new(Mac802_15_4Table {
                pcmd_rsp_buffer: MAC_802_15_4_CMD_BUFFER.as_mut_ptr().cast(),
                pnotack_buffer: MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr().cast(),
                evt_queue: EVT_QUEUE.as_ptr().cast(),
            });
        }

        ipcc.c1_set_rx_channel(channels::Cpu2Channel::Mac802_15_4NotifAck.into(), true);

        Self
    }

    pub(crate) fn notif_evt_handler(ipcc: &mut Ipcc) {
        unsafe {
            let notif_buffer: *mut EvtPacket = (*TL_REF_TABLE.assume_init().mac_802_15_4_table).pnotack_buffer.cast();
            let event = EvtBox::new(notif_buffer);

            block_on(TL_CHANNEL.send(event));
        }

        ipcc.c1_set_rx_channel(channels::Cpu2Channel::Mac802_15_4NotifAck.into(), false);
    }

    pub(crate) fn cmd_evt_handler(ipcc: &mut Ipcc) {
        unsafe {
            let _notif_buffer = (*TL_REF_TABLE.assume_init().mac_802_15_4_table).pcmd_rsp_buffer;

            // NOTE: ST's HAL does nothing with this buffer, ??????
        }

        ipcc.c1_set_tx_channel(channels::Cpu1Channel::Mac802_15_4cmdRsp.into(), false);
    }

    pub(crate) fn send_cmd(ipcc: &mut Ipcc, buf: &[u8]) {
        unsafe {
            let pcmd_buffer: *mut CmdPacket = (*TL_REF_TABLE.assume_init().mac_802_15_4_table).pcmd_rsp_buffer.cast();
            let pcmd_serial: *mut CmdSerial = &mut (*pcmd_buffer).cmd_serial;
            let pcmd_serial_buf: *mut u8 = pcmd_serial.cast();

            core::ptr::copy(buf.as_ptr(), pcmd_serial_buf, buf.len());

            let cmd_packet: &mut CmdPacket =
                &mut *(*TL_REF_TABLE.assume_init().mac_802_15_4_table).pcmd_rsp_buffer.cast();
            cmd_packet.cmd_serial.ty = TlPacketType::OtCmd as u8;
        }

        ipcc.c1_set_flag_channel(channels::Cpu1Channel::Mac802_15_4cmdRsp.into());
        ipcc.c1_set_tx_channel(channels::Cpu1Channel::Mac802_15_4cmdRsp.into(), true);
    }

    pub(crate) fn send_ack(ipcc: &mut Ipcc) {
        // TODO
        unsafe {
            let packet: &mut CmdPacket = &mut *(*TL_REF_TABLE.assume_init().mac_802_15_4_table).pnotack_buffer.cast();
            packet.cmd_serial.ty = TlPacketType::OtAck as u8;
        }

        ipcc.c1_clear_flag_channel(channels::Cpu2Channel::Mac802_15_4NotifAck.into());
        ipcc.c1_set_rx_channel(channels::Cpu2Channel::Mac802_15_4NotifAck.into(), true);
    }
}
