use core::mem::MaybeUninit;

use super::unsafe_linked_list::LST_init_head;
use super::{channels, BleTable, BLE_CMD_BUFFER, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::ipcc::Ipcc;

pub struct Ble;

impl Ble {
    pub fn new(ipcc: &mut Ipcc) -> Self {
        unsafe {
            LST_init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE = MaybeUninit::new(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_mut_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        ipcc.c1_set_rx_channel(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, true);

        Ble
    }
}
