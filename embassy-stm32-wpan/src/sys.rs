use embassy_stm32::ipcc::Ipcc;

use crate::cmd::{CmdPacket, CmdSerial};
use crate::evt::{CcEvt, EvtBox, EvtSerial};
use crate::tables::SysTable;
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, EVT_CHANNEL, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_SYS_TABLE};

pub struct Sys;

impl Sys {
    pub fn enable() {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE.as_mut_ptr().write_volatile(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            })
        }

        Ipcc::c1_set_rx_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, true);
    }

    pub fn cmd_evt_handler() -> CcEvt {
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, false);

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
            let pcmd: *const CmdPacket = (*TL_SYS_TABLE.as_ptr()).pcmd_buffer;
            let cmd_serial: *const CmdSerial = &(*pcmd).cmdserial;
            let evt_serial: *const EvtSerial = cmd_serial.cast();
            let cc: *const CcEvt = (*evt_serial).evt.payload.as_ptr().cast();
            *cc
        }
    }

    pub fn evt_handler() {
        unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let node_ptr_ptr: *mut _ = &mut node_ptr;

            while !LinkedListNode::is_empty(SYSTEM_EVT_QUEUE.as_mut_ptr()) {
                LinkedListNode::remove_head(SYSTEM_EVT_QUEUE.as_mut_ptr(), node_ptr_ptr);

                let event = node_ptr.cast();
                let event = EvtBox::new(event);

                EVT_CHANNEL.try_send(event).unwrap();
            }
        }

        Ipcc::c1_clear_flag_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL);
    }

    pub fn send_cmd() {
        Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL);
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, true);
    }
}
