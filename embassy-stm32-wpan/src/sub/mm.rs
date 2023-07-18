//! Memory manager routines
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::task::Poll;

use aligned::{Aligned, A4};
use cortex_m::interrupt;
use embassy_stm32::ipcc::Ipcc;
use embassy_sync::waitqueue::AtomicWaker;

use crate::consts::POOL_SIZE;
use crate::evt::EvtPacket;
#[cfg(feature = "ble")]
use crate::tables::BLE_SPARE_EVT_BUF;
use crate::tables::{MemManagerTable, EVT_POOL, FREE_BUF_QUEUE, SYS_SPARE_EVT_BUF, TL_MEM_MANAGER_TABLE};
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, evt};

static MM_WAKER: AtomicWaker = AtomicWaker::new();
static mut LOCAL_FREE_BUF_QUEUE: Aligned<A4, MaybeUninit<LinkedListNode>> = Aligned(MaybeUninit::uninit());

pub struct MemoryManager {
    phantom: PhantomData<MemoryManager>,
}

impl MemoryManager {
    pub(crate) fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(FREE_BUF_QUEUE.as_mut_ptr());
            LinkedListNode::init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE.as_mut_ptr().write_volatile(MemManagerTable {
                #[cfg(feature = "ble")]
                spare_ble_buffer: BLE_SPARE_EVT_BUF.as_ptr().cast(),
                #[cfg(not(feature = "ble"))]
                spare_ble_buffer: core::ptr::null(),
                spare_sys_buffer: SYS_SPARE_EVT_BUF.as_ptr().cast(),
                blepool: EVT_POOL.as_ptr().cast(),
                blepoolsize: POOL_SIZE as u32,
                pevt_free_buffer_queue: FREE_BUF_QUEUE.as_mut_ptr(),
                traces_evt_pool: core::ptr::null(),
                tracespoolsize: 0,
            });
        }

        Self { phantom: PhantomData }
    }

    pub async fn run_queue(&self) {
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

impl evt::MemoryManager for MemoryManager {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        interrupt::free(|_| unsafe {
            LinkedListNode::insert_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), evt as *mut _);
        });

        MM_WAKER.wake();
    }
}
