use embassy_futures::poll_once;

use super::channels;
use super::consts::POOL_SIZE;
use super::evt::EventPacket;
use super::ipcc::Ipcc;
use super::tables::{
    MemManagerTable, BLE_SPARE_EVT_BUF, EVT_POOL, FREE_BUFF_QUEUE, LOCAL_FREE_BUF_QUEUE, SYS_SPARE_EVT_BUF,
    TL_MEM_MANAGER_TABLE, TL_REF_TABLE,
};
use super::unsafe_linked_list::LinkedListNode;

pub struct MemoryManager;

impl MemoryManager {
    pub fn enable() {
        unsafe {
            LinkedListNode::init_head(FREE_BUFF_QUEUE.as_mut_ptr());
            LinkedListNode::init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE.as_mut_ptr().write_volatile(MemManagerTable {
                spare_ble_buffer: BLE_SPARE_EVT_BUF.as_ptr().cast(),
                spare_sys_buffer: SYS_SPARE_EVT_BUF.as_ptr().cast(),
                ble_pool: EVT_POOL.as_ptr().cast(),
                ble_pool_size: POOL_SIZE as u32,
                pevt_free_buffer_queue: FREE_BUFF_QUEUE.as_mut_ptr(),
                traces_evt_pool: core::ptr::null(),
                traces_pool_size: 0,
            });
        }
    }

    pub fn drop_packet(evt_packet: *mut EventPacket) {
        unsafe {
            let list_node = evt_packet.cast();

            LinkedListNode::insert_tail(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), list_node);
        }

        Self::run_queue()
    }

    pub fn run_queue() {
        if unsafe { LinkedListNode::is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) } {
            // Needed because Ipcc::send is expected to send data; therefore will trigger the interrupt

            return;
        }

        let _ = poll_once(Ipcc::send(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, || unsafe {
            let mut node_ptr = core::ptr::null_mut();
            let pevt_free_buffer_queue =
                (*TL_REF_TABLE.as_ptr().read_volatile().mem_manager_table).pevt_free_buffer_queue;

            while !LinkedListNode::is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
                LinkedListNode::remove_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), &mut node_ptr as *mut _);
                LinkedListNode::insert_tail(pevt_free_buffer_queue, node_ptr as *mut _);
            }
        }));
    }
}
