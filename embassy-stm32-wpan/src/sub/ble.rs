use core::marker::PhantomData;
use core::ptr;

use embassy_stm32::ipcc::Ipcc;
use hci::Opcode;

use crate::cmd::CmdPacket;
use crate::consts::{TlPacketType, TL_BLEEVT_CC_OPCODE, TL_BLEEVT_CS_OPCODE};
use crate::evt::{EvtBox, EvtPacket, EvtStub};
use crate::sub::mm;
use crate::tables::{BleTable, BLE_CMD_BUFFER, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, evt};

pub struct Ble {
    phantom: PhantomData<Ble>,
}

impl Ble {
    pub(crate) fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE.as_mut_ptr().write_volatile(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Self { phantom: PhantomData }
    }
    /// `HW_IPCC_BLE_EvtNot`
    pub async fn tl_read(&self) -> EvtBox<Self> {
        Ipcc::receive(channels::cpu2::IPCC_BLE_EVENT_CHANNEL, || unsafe {
            if let Some(node_ptr) = LinkedListNode::remove_head(EVT_QUEUE.as_mut_ptr()) {
                Some(EvtBox::new(node_ptr.cast()))
            } else {
                None
            }
        })
        .await
    }

    /// `TL_BLE_SendCmd`
    pub async fn tl_write(&self, opcode: u16, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_BLE_CMD_CHANNEL, || unsafe {
            CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
        })
        .await;
    }

    /// `TL_BLE_SendAclData`
    pub async fn acl_write(&self, handle: u16, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL, || unsafe {
            CmdPacket::write_into(
                HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _,
                TlPacketType::AclData,
                handle,
                payload,
            );
        })
        .await;
    }
}

impl evt::MemoryManager for Ble {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        let stub = unsafe {
            let p_evt_stub = &(*evt).evt_serial as *const _ as *const EvtStub;

            ptr::read_volatile(p_evt_stub)
        };

        if !(stub.evt_code == TL_BLEEVT_CS_OPCODE || stub.evt_code == TL_BLEEVT_CC_OPCODE) {
            mm::MemoryManager::drop_event_packet(evt);
        }
    }
}

pub extern crate stm32wb_hci as hci;

impl hci::Controller for Ble {
    async fn controller_write(&mut self, opcode: Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();

        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}
