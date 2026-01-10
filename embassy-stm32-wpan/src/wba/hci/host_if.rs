//! HCI Host Stack Interface
//!
//! This module provides the interface between the C link layer and the Rust HCI event handling.
//! The C layer calls into Rust via the `HostStack_Process` function to deliver HCI events.

use core::ptr;

use super::event::{Event, try_send_event};

/// Static buffer for receiving HCI event packets from C layer
/// Maximum HCI event packet size is 257 bytes (1 byte event code + 1 byte length + up to 255 bytes data)
static mut HCI_EVENT_BUFFER: [u8; 260] = [0u8; 260];

/// Flag to track if we have a pending event to process
static mut HAS_PENDING_EVENT: bool = false;

/// Initialize the HCI host interface
/// This should be called during BLE stack initialization
pub unsafe fn init() {
    // The host callback will be registered in ll_sys_ble_cntrl_init
    // Here we just ensure our buffers are ready
    HAS_PENDING_EVENT = false;
}

/// Host callback function called by the C link layer when an HCI event is available
///
/// This function is called from C code via the hst_cbk callback registered in ll_sys_ble_cntrl_init.
/// The event_ptr points to an HCI event packet in the format:
/// - Byte 0: Event code
/// - Byte 1: Parameter length
/// - Bytes 2+: Parameters
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hci_host_callback(event_ptr: *const u8, event_len: u16) {
    if event_ptr.is_null() || event_len == 0 || event_len > 260 {
        return;
    }

    // Copy event data to our buffer
    ptr::copy_nonoverlapping(event_ptr, HCI_EVENT_BUFFER.as_mut_ptr(), event_len as usize);
    HAS_PENDING_EVENT = true;

    // Note: The actual processing happens in HostStack_Process
    // which is called from the background task
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

    // Parse the event
    let event_data = &HCI_EVENT_BUFFER[..];
    if let Some(event) = Event::parse(event_data) {
        // Send all events to the general event channel
        // Note: CommandComplete/CommandStatus events are now handled synchronously
        // by the C HCI functions, so these events are mainly for informational purposes
        let _ = try_send_event(event);
    }

    HAS_PENDING_EVENT = false;
}

// Note: For WBA, the callback mechanism above is the primary event delivery method.
// The C link layer calls hci_host_callback() when events are available, and
// HostStack_Process() is called from the background task to process them.
