use core::mem::MaybeUninit;

use super::evt::EvtPacket;
use super::unsafe_linked_list::{LST_init_head, LST_insert_tail, LST_is_empty, LST_remove_head};
use super::{
    channels, MemManagerTable, BLE_SPARE_EVT_BUF, EVT_POOL, FREE_BUFF_QUEUE, LOCAL_FREE_BUF_QUEUE, POOL_SIZE,
    SYS_SPARE_EVT_BUF, TL_MEM_MANAGER_TABLE, TL_REF_TABLE,
};
use crate::ipcc::Ipcc;

pub struct MemoryManager;

impl MemoryManager {
    pub fn new() -> Self {
        unsafe {
            LST_init_head(FREE_BUFF_QUEUE.as_mut_ptr());
            LST_init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE = MaybeUninit::new(MemManagerTable {
                spare_ble_buffer: BLE_SPARE_EVT_BUF.as_ptr().cast(),
                spare_sys_buffer: SYS_SPARE_EVT_BUF.as_ptr().cast(),
                ble_pool: EVT_POOL.as_ptr().cast(),
                ble_pool_size: POOL_SIZE as u32,
                pevt_free_buffer_queue: FREE_BUFF_QUEUE.as_mut_ptr(),
                traces_evt_pool: core::ptr::null(),
                traces_pool_size: 0,
            });
        }

        MemoryManager
    }

    pub fn evt_handler(ipcc: &mut Ipcc) {
        ipcc.c1_set_tx_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, false);
        Self::send_free_buf();
        ipcc.c1_set_flag_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);
    }

    pub fn evt_drop(evt: *mut EvtPacket, ipcc: &mut Ipcc) {
        unsafe {
            let list_node = evt.cast();

            LST_insert_tail(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), list_node);
        }

        let channel_is_busy = ipcc.c1_is_active_flag(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);

        // postpone event buffer freeing to IPCC interrupt handler
        if channel_is_busy {
            ipcc.c1_set_tx_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, true);
        } else {
            Self::send_free_buf();
            ipcc.c1_set_flag_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);
        }
    }

    fn send_free_buf() {
        unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let node_ptr_ptr: *mut _ = &mut node_ptr;

            while !LST_is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
                LST_remove_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), node_ptr_ptr);
                LST_insert_tail(
                    (*(*TL_REF_TABLE.as_ptr()).mem_manager_table).pevt_free_buffer_queue,
                    node_ptr,
                );
            }
        }
    }
}
