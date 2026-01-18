//! HCI Host Stack Interface
//!
//! This module provides the interface between the C link layer and the Rust HCI event handling.
//! The C layer calls into Rust via the `HostStack_Process` function to deliver HCI events.

use core::ptr;

use super::event::{Event, try_send_event};
use crate::bindings::link_layer::ble_buff_hdr_t;

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

    // Copy event data to our buffer
    ptr::copy_nonoverlapping(data_ptr, HCI_EVENT_BUFFER.as_mut_ptr(), data_len as usize);
    HCI_EVENT_LEN = data_len;
    HAS_PENDING_EVENT = true;

    // Note: The actual processing happens in HostStack_Process
    // which is called from the background task

    0 // Success
}

/// Process pending HCI events
///
/// This function is called from ll_sys_bg_process (the link layer background task).
/// It checks if there are pending events from the C layer and processes them.
/// This is the implementation of the weak HostStack_Process function in the C code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HostStack_Process() {
    if !HAS_PENDING_EVENT {
        return;
    }

    // Parse the event using the stored length
    let event_len = HCI_EVENT_LEN as usize;
    if event_len > 0 && event_len <= HCI_EVENT_BUFFER.len() {
        let event_data = &HCI_EVENT_BUFFER[..event_len];
        if let Some(event) = Event::parse(event_data) {
            // Send all events to the general event channel
            // Note: CommandComplete/CommandStatus events are now handled synchronously
            // by the C HCI functions, so these events are mainly for informational purposes
            let _ = try_send_event(event);
        }
    }

    HAS_PENDING_EVENT = false;
    HCI_EVENT_LEN = 0;
}

// Note: For WBA, the callback mechanism above is the primary event delivery method.
// The C link layer calls hci_host_callback() when events are available, and
// HostStack_Process() is called from the background task to process them.
