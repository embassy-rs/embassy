//! Memory manager routines

use core::future::poll_fn;
use core::task::Poll;

use cortex_m::interrupt;
use embassy_stm32::ipcc::Ipcc;
use embassy_sync::waitqueue::AtomicWaker;

use crate::evt::EvtPacket;
use crate::tables::MemManagerTable;
use crate::unsafe_linked_list::LinkedListNode;
use crate::{
    channels, BLE_SPARE_EVT_BUF, EVT_POOL, FREE_BUF_QUEUE, LOCAL_FREE_BUF_QUEUE, POOL_SIZE, SYS_SPARE_EVT_BUF,
    TL_MEM_MANAGER_TABLE,
};

static MM_WAKER: AtomicWaker = AtomicWaker::new();

pub struct MemoryManager;

impl MemoryManager {
    pub fn enable() {
        unsafe {
            LinkedListNode::init_head(FREE_BUF_QUEUE.as_mut_ptr());
            LinkedListNode::init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE.as_mut_ptr().write_volatile(MemManagerTable {
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

    /// SAFETY: passing a pointer to something other than an event packet is UB
    pub unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        interrupt::free(|_| unsafe {
            LinkedListNode::insert_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), evt as *mut _);
        });

        MM_WAKER.wake();
    }

    pub async fn run_queue() {
        loop {
            poll_fn(|cx| unsafe {
                MM_WAKER.register(cx.waker());
                if LinkedListNode::is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;

            Ipcc::send(channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL, || {
                interrupt::free(|_| unsafe {
                    // CS required while moving nodes
                    while let Some(node_ptr) = LinkedListNode::remove_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
                        LinkedListNode::insert_head(FREE_BUF_QUEUE.as_mut_ptr(), node_ptr);
                    }
                })
            })
            .await;
        }
    }
}
