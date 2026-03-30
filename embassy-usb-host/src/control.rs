//! Standard USB control request builders.

use crate::descriptor::descriptor_type;

/// USB request type direction bit.
const DIR_DEVICE_TO_HOST: u8 = 0x80;

/// USB request type: standard.
const TYPE_STANDARD: u8 = 0x00;

/// USB request type: class.
#[allow(dead_code)]
const TYPE_CLASS: u8 = 0x20;

/// Recipient: device.
const RECIPIENT_DEVICE: u8 = 0x00;

/// Recipient: interface.
const RECIPIENT_INTERFACE: u8 = 0x01;

/// Standard request codes.
const GET_DESCRIPTOR: u8 = 0x06;
const SET_ADDRESS: u8 = 0x05;
const SET_CONFIGURATION: u8 = 0x09;

/// Build a GET_DESCRIPTOR(Device) SETUP packet.
pub fn get_device_descriptor(max_len: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_DEVICE,
        GET_DESCRIPTOR,
        (descriptor_type::DEVICE as u16) << 8,
        0,
        max_len,
    )
}

/// Build a GET_DESCRIPTOR(Configuration) SETUP packet.
pub fn get_config_descriptor(index: u8, max_len: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_DEVICE,
        GET_DESCRIPTOR,
        ((descriptor_type::CONFIGURATION as u16) << 8) | index as u16,
        0,
        max_len,
    )
}

/// Build a SET_ADDRESS SETUP packet.
pub fn set_address(address: u8) -> [u8; 8] {
    make_setup(TYPE_STANDARD | RECIPIENT_DEVICE, SET_ADDRESS, address as u16, 0, 0)
}

/// Build a SET_CONFIGURATION SETUP packet.
pub fn set_configuration(config_value: u8) -> [u8; 8] {
    make_setup(
        TYPE_STANDARD | RECIPIENT_DEVICE,
        SET_CONFIGURATION,
        config_value as u16,
        0,
        0,
    )
}

/// Build a GET_DESCRIPTOR(HID Report Descriptor) SETUP packet (Standard, Interface).
///
/// `interface` is the HID interface number; `len` is from `HidInfo::report_descriptor_len`.
pub fn get_hid_report_descriptor(interface: u8, len: u16) -> [u8; 8] {
    // wValue = descriptor_type(0x22) << 8 | index(0)
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_INTERFACE,
        0x06,
        0x2200,
        interface as u16,
        len,
    )
}

/// Build a class-specific interface request SETUP packet (OUT, no data).
pub fn class_interface_out(request: u8, value: u16, interface: u16) -> [u8; 8] {
    make_setup(TYPE_CLASS | RECIPIENT_INTERFACE, request, value, interface, 0)
}

/// Build a class-specific interface request SETUP packet (OUT, with data).
pub fn class_interface_out_with_data(request: u8, value: u16, interface: u16, length: u16) -> [u8; 8] {
    make_setup(TYPE_CLASS | RECIPIENT_INTERFACE, request, value, interface, length)
}

/// Build a class-specific interface request SETUP packet (IN, with data).
pub fn class_interface_in_with_data(request: u8, value: u16, interface: u16, length: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_CLASS | RECIPIENT_INTERFACE,
        request,
        value,
        interface,
        length,
    )
}

fn make_setup(bm_request_type: u8, b_request: u8, w_value: u16, w_index: u16, w_length: u16) -> [u8; 8] {
    let value_bytes = w_value.to_le_bytes();
    let index_bytes = w_index.to_le_bytes();
    let length_bytes = w_length.to_le_bytes();
    [
        bm_request_type,
        b_request,
        value_bytes[0],
        value_bytes[1],
        index_bytes[0],
        index_bytes[1],
        length_bytes[0],
        length_bytes[1],
    ]
}
