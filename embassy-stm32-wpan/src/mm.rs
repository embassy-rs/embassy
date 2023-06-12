//! Memory manager routines

use core::mem::MaybeUninit;

use embassy_stm32::ipcc::Ipcc;

use crate::evt::EvtPacket;
use crate::tables::MemManagerTable;
use crate::unsafe_linked_list::LinkedListNode;
use crate::{
    channels, BLE_SPARE_EVT_BUF, EVT_POOL, FREE_BUF_QUEUE, LOCAL_FREE_BUF_QUEUE, POOL_SIZE, SYS_SPARE_EVT_BUF,
    TL_MEM_MANAGER_TABLE, TL_REF_TABLE,
};

pub(super) struct MemoryManager;

impl MemoryManager {
    pub fn enable() {
        unsafe {
            LinkedListNode::init_head(FREE_BUF_QUEUE.as_mut_ptr());
            LinkedListNode::init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE = MaybeUninit::new(MemManagerTable {
                spare_ble_buffer: BLE_SPARE_EVT_BUF.as_ptr().cast(),
                spare_sys_buffer: SYS_SPARE_EVT_BUF.as_ptr().cast(),
                blepool: EVT_POOL.as_ptr().cast(),
                blepoolsize: POOL_SIZE as u32,
                pevt_free_buffer_queue: FREE_BUF_QUEUE.as_mut_ptr(),
                traces_evt_pool: core::ptr::null(),
                tracespoolsize: 0,
            });
        }
    }

    pub fn evt_drop(evt: *mut EvtPacket) {
        unsafe {
            let list_node = evt.cast();

            LinkedListNode::insert_tail(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), list_node);

            let channel_is_busy = Ipcc::c1_is_active_flag(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);

            // postpone event buffer freeing to IPCC interrupt handler
            if channel_is_busy {
                Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, true);
            } else {
                Self::send_free_buf();
                Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);
            }
        }
    }

    /// gives free event buffers back to CPU2 from local buffer queue
    pub fn send_free_buf() {
        unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let node_ptr_ptr: *mut _ = &mut node_ptr;

            while !LinkedListNode::is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
                LinkedListNode::remove_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), node_ptr_ptr);
                LinkedListNode::insert_tail(
                    (*(*TL_REF_TABLE.as_ptr()).mem_manager_table).pevt_free_buffer_queue,
                    node_ptr,
                );
            }
        }
    }

    /// free buffer channel interrupt handler
    pub fn free_buf_handler() {
        Ipcc::c1_set_tx_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, false);
        Self::send_free_buf();
        Ipcc::c1_set_flag_channel(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL);
    }
}
