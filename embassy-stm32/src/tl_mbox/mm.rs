use core::mem::MaybeUninit;

use super::unsafe_linked_list::LST_init_head;
use super::{
    MemManagerTable, BLE_SPARE_EVT_BUF, EVT_POOL, FREE_BUFF_QUEUE, LOCAL_FREE_BUF_QUEUE, POOL_SIZE, SYS_SPARE_EVT_BUF,
    TL_MEM_MANAGER_TABLE,
};

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
}
