use core::mem::MaybeUninit;

use super::unsafe_linked_list::LST_init_head;
use super::{channels, SysTable, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_SYS_TABLE};
use crate::ipcc::Ipcc;

pub struct Sys;

impl Sys {
    pub fn new(ipcc: &mut Ipcc) -> Self {
        unsafe {
            LST_init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE = MaybeUninit::new(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }

        ipcc.c1_set_rx_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, true);

        Sys
    }
}
