use core::mem::MaybeUninit;

use embassy_futures::block_on;

use super::cmd::{CmdPacket, CmdSerial};
use super::consts::TlPacketType;
use super::evt::{CcEvt, EvtBox, EvtSerial};
use super::unsafe_linked_list::LinkedListNode;
use super::{channels, SysTable, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_CHANNEL, TL_REF_TABLE, TL_SYS_TABLE};
use crate::ipcc::Ipcc;

pub struct Sys;

impl Sys {
    pub(crate) fn new(ipcc: &mut Ipcc) -> Self {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE = MaybeUninit::new(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }

        ipcc.c1_set_rx_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, true);

        Sys
    }

    pub(crate) fn evt_handler(ipcc: &mut Ipcc) {
        unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let node_ptr_ptr: *mut _ = &mut node_ptr;

            while !LinkedListNode::is_empty(SYSTEM_EVT_QUEUE.as_mut_ptr()) {
                LinkedListNode::remove_head(SYSTEM_EVT_QUEUE.as_mut_ptr(), node_ptr_ptr);

                let event = node_ptr.cast();
                let event = EvtBox::new(event);

                // TODO: not really happy about this
                block_on(TL_CHANNEL.send(event));
            }
        }

        ipcc.c1_clear_flag_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL);
    }

    pub(crate) fn cmd_evt_handler(ipcc: &mut Ipcc) -> CcEvt {
        ipcc.c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, false);

        // ST's command response data structure is really convoluted.
        //
        // for command response events on SYS channel, the header is missing
        // and one should:
        // 1. interpret the content of CMD_BUFFER as CmdPacket
        // 2. Access CmdPacket's cmdserial field and interpret its content as EvtSerial
        // 3. Access EvtSerial's evt field (as Evt) and interpret its payload as CcEvt
        // 4. CcEvt type is the actual SHCI response
        // 5. profit
        unsafe {
            let cmd: *const CmdPacket = (*TL_SYS_TABLE.as_ptr()).pcmd_buffer;
            let cmd_serial: *const CmdSerial = &(*cmd).cmd_serial;
            let evt_serial: *const EvtSerial = cmd_serial.cast();
            let cc = (*evt_serial).evt.payload.as_ptr().cast();
            *cc
        }
    }

    #[allow(dead_code)]
    pub(crate) fn send_cmd(ipcc: &mut Ipcc, buf: &[u8]) {
        unsafe {
            // TODO: check this
            let cmd_buffer = &mut *(*TL_REF_TABLE.assume_init().sys_table).pcmd_buffer;
            let cmd_serial: *mut CmdSerial = &mut (*cmd_buffer).cmd_serial;
            let cmd_serial_buf = cmd_serial.cast();

            core::ptr::copy(buf.as_ptr(), cmd_serial_buf, buf.len());

            let cmd_packet = &mut *(*TL_REF_TABLE.assume_init().sys_table).pcmd_buffer;
            cmd_packet.cmd_serial.ty = TlPacketType::SysCmd as u8;

            ipcc.c1_set_flag_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL);
            ipcc.c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, true);
        }
    }
}
