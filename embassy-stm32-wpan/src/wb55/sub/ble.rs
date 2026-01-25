use core::ptr;

use embassy_stm32::ipcc::{IpccRxChannel, IpccTxChannel};
use hci::Opcode;

use crate::cmd::CmdPacket;
use crate::consts::{TL_BLEEVT_CC_OPCODE, TL_BLEEVT_CS_OPCODE, TlPacketType};
use crate::evt;
use crate::evt::{EvtBox, EvtPacket, EvtStub};
use crate::sub::mm;
use crate::tables::{BLE_CMD_BUFFER, BleTable, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::unsafe_linked_list::LinkedListNode;

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
pub struct Ble<'a> {
    hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
    ipcc_ble_event_channel: IpccRxChannel<'a>,
    ipcc_hci_acl_data_channel: IpccTxChannel<'a>,
}

/// BLE for only sending commands to CPU2
pub struct BleTx<'a> {
    hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
    ipcc_hci_acl_data_channel: IpccTxChannel<'a>,
}

/// BLE for only receive commands from CPU2
pub struct BleRx<'a> {
    ipcc_ble_event_channel: IpccRxChannel<'a>,
}

impl<'a> Ble<'a> {
    /// Constructs a guard that allows for BLE commands to be sent to CPU2.
    ///
    /// This takes the place of `TL_BLE_Init`, completing that step as laid out in AN5289, Fig 66.
    pub(crate) fn new(
        hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
        ipcc_ble_event_channel: IpccRxChannel<'a>,
        ipcc_hci_acl_data_channel: IpccTxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE.as_mut_ptr().write_volatile(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Self {
            hw_ipcc_ble_cmd_channel,
            ipcc_ble_event_channel,
            ipcc_hci_acl_data_channel,
        }
    }

    /// Split current BLE into BleTx and BleRx
    pub fn split(self) -> (BleTx<'a>, BleRx<'a>) {
        (
            BleTx {
                hw_ipcc_ble_cmd_channel: self.hw_ipcc_ble_cmd_channel,
                ipcc_hci_acl_data_channel: self.ipcc_hci_acl_data_channel,
            },
            BleRx {
                ipcc_ble_event_channel: self.ipcc_ble_event_channel,
            },
        )
    }

    /// `HW_IPCC_BLE_EvtNot`
    pub async fn tl_read(&mut self) -> EvtBox<Self> {
        self.ipcc_ble_event_channel
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await
    }

    /// `TL_BLE_SendCmd`
    pub async fn tl_write(&mut self, opcode: u16, payload: &[u8]) {
        self.hw_ipcc_ble_cmd_channel
            .send(|| unsafe {
                CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
            })
            .await;
    }

    /// `TL_BLE_SendAclData`
    pub async fn acl_write(&mut self, handle: u16, payload: &[u8]) {
        self.ipcc_hci_acl_data_channel
            .send(|| unsafe {
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

impl<'a> evt::MemoryManager for Ble<'a> {
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

impl<'a> hci::Controller for Ble<'a> {
    async fn controller_write(&mut self, opcode: Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&mut self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();

        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}

impl<'a> BleTx<'a> {
    /// `TL_BLE_SendCmd`
    pub async fn tl_write(&mut self, opcode: u16, payload: &[u8]) {
        self.hw_ipcc_ble_cmd_channel
            .send(|| unsafe {
                CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
            })
            .await;
    }

    /// `TL_BLE_SendAclData`
    pub async fn acl_write(&mut self, handle: u16, payload: &[u8]) {
        self.ipcc_hci_acl_data_channel
            .send(|| unsafe {
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

impl<'a> BleRx<'a> {
    /// `HW_IPCC_BLE_EvtNot`
    pub async fn tl_read(&mut self) -> EvtBox<Self> {
        self.ipcc_ble_event_channel
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await
    }
}

impl<'a> evt::MemoryManager for BleRx<'a> {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        // Reuse the logic from the original BLE implementation
        let stub = unsafe {
            let p_evt_stub = &(*evt).evt_serial as *const _ as *const EvtStub;
            ptr::read_volatile(p_evt_stub)
        };

        if !(stub.evt_code == TL_BLEEVT_CS_OPCODE || stub.evt_code == TL_BLEEVT_CC_OPCODE) {
            mm::MemoryManager::drop_event_packet(evt);
        }
    }
}

/// Implement Controller for TX (Write only)
impl<'a> hci::Controller for BleTx<'a> {
    async fn controller_write(&mut self, opcode: Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&mut self, _buf: &mut [u8]) {
        panic!("BleTx cannot read!");
    }
}

/// Implement Controller for RX (Read only)
impl<'a> hci::Controller for BleRx<'a> {
    async fn controller_write(&mut self, _opcode: Opcode, _payload: &[u8]) {
        panic!("BleRx cannot write!");
    }

    async fn controller_read_into(&mut self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();
        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}
