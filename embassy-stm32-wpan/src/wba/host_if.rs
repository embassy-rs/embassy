//! HCI Host Stack Interface
//!
//! This module provides the interface between the C link layer and the Rust HCI event handling.
//! The C layer calls into Rust via the `HostStack_Process` function to deliver HCI events.

use core::ptr;

// Event delivery is handled by BLECB_Indication (linklayer_plat.rs), not here.
use crate::util_seq;
use crate::wba::bindings::link_layer::ble_buff_hdr_t;
use crate::wba::bindings::mac;

// Task ID for BLE Host processing (next available after CFG_TASK_NBR=9)
pub const CFG_TASK_BLE_HOST: u32 = 9;
pub const TASK_BLE_HOST_MASK: u32 = 1 << CFG_TASK_BLE_HOST;
pub const TASK_PRIO_BLE_HOST: u32 = 0; // CFG_SEQ_PRIO_0
// Link Layer background task
pub const TASK_LINK_LAYER_MASK: u32 = 1 << mac::CFG_TASK_ID_T_CFG_TASK_LINK_LAYER;

pub const MAX_BLE_PKT_SIZE: usize = 280;

/// Static buffer for receiving HCI event packets from C layer
/// Maximum HCI event packet size is 257 bytes (1 byte event code + 1 byte length + up to 255 bytes data)
static mut HCI_EVENT_BUFFER: [u8; 260] = [0u8; 260];
static mut HCI_EVENT_LEN: u16 = 0;

/// Flag to track if we have a pending event to process
static mut HAS_PENDING_EVENT: bool = false;

/// Initialize the HCI host interface
/// This should be called during BLE stack initialization
pub unsafe fn init() {
    // The host callback will be registered in ll_sys_ble_cntrl_init
    // Here we just ensure our buffers are ready
    HAS_PENDING_EVENT = false;
    HCI_EVENT_LEN = 0;
}

/// Host callback function called by the C link layer when an HCI event is available
///
/// This function is called from C code via the hst_cbk callback registered in ll_sys_ble_cntrl_init.
/// The ptr_evnt_hdr points to a ble_buff_hdr_t structure containing:
/// - buff_start: Pointer to the data buffer
/// - data_offset: Offset from buff_start to actual data
/// - data_size: Size of the data
///
/// Returns 0 on success, non-zero on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hci_host_callback(ptr_evnt_hdr: *mut ble_buff_hdr_t) -> u8 {
    if ptr_evnt_hdr.is_null() {
        return 1;
    }

    let hdr = &*ptr_evnt_hdr;
    let data_ptr = hdr.buff_start.add(hdr.data_offset as usize);
    let data_len = hdr.data_size;

    if data_ptr.is_null() || data_len == 0 || data_len > 260 {
        return 1;
    }

    // TODO: it appears HCI_EVENT_BUFFER is never read

    // Copy event data to our buffer
    ptr::copy_nonoverlapping(data_ptr, HCI_EVENT_BUFFER.as_mut_ptr(), data_len as usize);
    HCI_EVENT_LEN = data_len;
    HAS_PENDING_EVENT = true;

    // Note: The actual processing happens in HostStack_Process
    // which is called from the background task

    0 // Success
}

/// Process pending HCI events from the link layer.
///
/// This function is called from ll_sys_bg_process (the link layer background task).
/// It schedules the BLE Host task so BleStack_Process runs.
///
/// NOTE: Event delivery to the application is handled exclusively by BLECB_Indication
/// (in linklayer_plat.rs), which is called by the BLE Host stack after it processes
/// events internally. We do NOT parse events here to avoid delivering duplicates.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HostStack_Process() {
    // Clear any pending flag — the event has been consumed by the C host stack
    HAS_PENDING_EVENT = false;
    HCI_EVENT_LEN = 0;

    // Schedule BLE Host task to process events (matches ST's BleStackCB_Process)
    // This is CRITICAL - it's what keeps BleStack_Process running!
    util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);
}

// Note: For WBA, the callback mechanism above is the primary event delivery method.
// The C link layer calls hci_host_callback() when events are available, and
// HostStack_Process() is called from the background task to process them.
