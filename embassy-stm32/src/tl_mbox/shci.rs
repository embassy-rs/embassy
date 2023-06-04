//! HCI commands for system channel

use super::consts::{TL_CS_EVT_SIZE, TL_EVT_HEADER_SIZE, TL_PACKET_HEADER_SIZE};

#[allow(dead_code)]
pub const SCHI_OPCODE_BLE_INIT: u16 = 0xfc66;
pub const TL_BLE_EVT_CS_PACKET_SIZE: usize = TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE;
#[allow(dead_code)]
const TL_BLE_EVT_CS_BUFFER_SIZE: usize = TL_PACKET_HEADER_SIZE + TL_BLE_EVT_CS_PACKET_SIZE;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ShciBleInitCmdParam {
    /// NOT USED CURRENTLY
    pub p_ble_buffer_address: u32,

    /// Size of the Buffer allocated in pBleBufferAddress
    pub ble_buffer_size: u32,

    pub num_attr_record: u16,
    pub num_attr_serv: u16,
    pub attr_value_arr_size: u16,
    pub num_of_links: u8,
    pub extended_packet_length_enable: u8,
    pub pr_write_list_size: u8,
    pub mb_lock_count: u8,

    pub att_mtu: u16,
    pub slave_sca: u16,
    pub master_sca: u8,
    pub ls_source: u8,
    pub max_conn_event_length: u32,
    pub hs_startup_time: u16,
    pub viterbi_enable: u8,
    pub ll_only: u8,
    pub hw_version: u8,
}

impl Default for ShciBleInitCmdParam {
    fn default() -> Self {
        Self {
            p_ble_buffer_address: 0,
            ble_buffer_size: 0,
            num_attr_record: 68,
            num_attr_serv: 8,
            attr_value_arr_size: 1344,
            num_of_links: 2,
            extended_packet_length_enable: 1,
            pr_write_list_size: 0x3A,
            mb_lock_count: 0x79,
            att_mtu: 156,
            slave_sca: 500,
            master_sca: 0,
            ls_source: 1,
            max_conn_event_length: 0xFFFFFFFF,
            hs_startup_time: 0x148,
            viterbi_enable: 1,
            ll_only: 0,
            hw_version: 0,
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct ShciHeader {
    metadata: [u32; 3],
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ShciBleInitCommandPacket {
    pub header: ShciHeader,
    pub param: ShciBleInitCmdParam,
}
