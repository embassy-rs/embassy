use core::ptr;

use embassy_stm32::ipcc::Ipcc;
use hci::Opcode;

use crate::cmd::CmdPacket;
use crate::consts::{TL_BLEEVT_CC_OPCODE, TL_BLEEVT_CS_OPCODE, TlPacketType};
use crate::evt::{EvtBox, EvtPacket, EvtStub};
use crate::sub::mm;
use crate::tables::{BLE_CMD_BUFFER, BleTable, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::unsafe_linked_list::LinkedListNode;
use crate::{channels, evt};

/// A guard that, once constructed, may be used to send BLE commands to CPU2.
///
/// It is the responsibility of the caller to ensure that they have awaited an event via
/// [crate::sub::sys::Sys::read] before sending any of these commands, and to call
/// [crate::sub::sys::Sys::shci_c2_ble_init] and await the HCI_COMMAND_COMPLETE_EVENT before
/// sending any other commands.
///
/// # Example
///
/// ```
/// # embassy_stm32::bind_interrupts!(struct Irqs{
/// #     IPCC_C1_RX => ReceiveInterruptHandler;
/// #     IPCC_C1_TX => TransmitInterruptHandler;
/// # });
/// #
/// # let p = embassy_stm32::init(embassy_stm32::Config::default());
/// # let mut mbox = embassy_stm32_wpan::TlMbox::init(p.IPCC, Irqs, embassy_stm32::ipcc::Config::default());
/// #
/// # let sys_event = mbox.sys_subsystem.read().await;
/// # let _command_status = mbox.sys_subsystem.shci_c2_ble_init(Default::default());
/// # // BLE commands may now be sent
/// #
/// # mbox.ble_subsystem.reset().await;
/// # let _reset_response = mbox.ble_subsystem.read().await;
/// ```
pub struct Ble {
    _private: (),
}

impl Ble {
    /// Constructs a guard that allows for BLE commands to be sent to CPU2.
    ///
    /// This takes the place of `TL_BLE_Init`, completing that step as laid out in AN5289, Fig 66.
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

        Self { _private: () }
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
