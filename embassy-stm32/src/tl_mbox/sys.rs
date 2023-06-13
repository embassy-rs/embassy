use core::mem::MaybeUninit;
use core::{mem, ptr};

use atomic_polyfill::{compiler_fence, Ordering};

use super::cmd::{Command, CommandPacket};
use super::consts::TlPacketType;
use super::evt::{CommandChannelEvent, EventPacket, EventSerial};
use super::shci::{SchiCommandStatus, ShciBleInitCmdParam, ShciConfigParam, ShciOpcode};
use super::tables::{SysTable, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_SYS_TABLE};
use super::unsafe_linked_list::LinkedListNode;
use super::{channels, mm, Ipcc};
use crate::tl_mbox::cmd::CommandSerial;

// #[derive(Debug, Copy, Clone, Default)]
// #[repr(C, packed)]
// pub struct ShciHeader {
//     meta_data: [u32; 3],
// }
//
// #[derive(Copy, Clone)]
// #[repr(C, packed)]
// pub struct ShciBleInitCmdPacket {
//     header: ShciHeader,
//     param: ShciBleInitCmdParam,
// }

pub struct SysSubsystem;

impl SysSubsystem {
    /// TL_Sys_Init
    pub fn new() -> Self {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE.as_mut_ptr().write_volatile(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }

        Self {}
    }

    pub async fn shci_c2_config(&mut self, param: ShciConfigParam) -> SchiCommandStatus {
        let command_event = self
            .write_and_get_response(TlPacketType::SysCmd, ShciOpcode::Config as u16, param.payload())
            .await;

        let payload = command_event.payload[0];
        // info!("payload: {:x}", payload);

        payload.try_into().unwrap()
    }

    #[cfg(feature = "ble")]
    pub async fn shci_c2_ble_init(&mut self, param: ShciBleInitCmdParam) -> SchiCommandStatus {
        let command_event = self
            .write_and_get_response(TlPacketType::SysCmd, ShciOpcode::BleInit as u16, param.payload())
            .await;

        let payload = command_event.payload[0];
        // info!("payload: {:x}", payload);

        payload.try_into().unwrap()
    }

    #[cfg(feature = "mac")]
    pub async fn schi_c2_mac_802_15_4_init(&mut self) -> SchiCommandStatus {
        self.write_and_get_response(
            |command_packet| unsafe {
                // info!("inner schi_c2_mac_802_15_4_init");

                let mut cmd_serial = CommandSerial::default();
                cmd_serial.cmd.cmd_code = ShciOpcode::Mac802_15_4Init as u16;
                cmd_serial.cmd.payload_len = 0;

                ptr::copy_nonoverlapping(&cmd_serial, &mut command_packet.cmd_serial, 1);
            },
            |command_event| unsafe {
                (&command_event.payload as *const u8)
                    .read_volatile()
                    .try_into()
                    .unwrap()
            },
        )
        .await
    }

    /// `HW_IPCC_SYS_EvtNot`
    pub async fn read<R>(&mut self, mut r: impl FnMut(&EventPacket) -> R) -> R {
        Ipcc::receive(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, || unsafe {
            let node = SYSTEM_EVT_QUEUE.as_mut_ptr();
            let mut node_ptr = core::ptr::null_mut();
            if LinkedListNode::is_empty(node) {
                None
            } else {
                LinkedListNode::remove_head(node, &mut node_ptr);

                let node_ptr: *const EventPacket = node_ptr.cast();
                let ret = r(mem::transmute(node_ptr));

                compiler_fence(Ordering::SeqCst);
                mm::MemoryManager::drop_packet(node_ptr as *mut _);

                Some(ret)
            }
        })
        .await
    }

    /// `HW_IPCC_SYS_CmdEvtNot`
    pub async fn write_and_get_response(
        &mut self,
        typ: TlPacketType,
        opcode: u16,
        payload: &[u8],
    ) -> CommandChannelEvent {
        self.write(typ, opcode, payload).await;
        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;

        unsafe {
            let p_event_packet = SYS_CMD_BUF.as_ptr() as *const EventPacket;
            let p_command_event =
                &((*p_event_packet).event_serial.event.payload) as *const _ as *const CommandChannelEvent;

            p_command_event.read_volatile()
        }
    }

    /// `TL_Sys_SendCmd`
    pub async fn write(&mut self, typ: TlPacketType, opcode: u16, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || unsafe {
            let p_cmd_serial = &mut SYS_CMD_BUF.assume_init().cmd_serial as *mut _;

            // TODO: Optimize this so that only the required payload is written
            let mut command_serial = CommandSerial {
                typ: typ as u8,
                command: Command {
                    command_code: opcode,
                    payload_len: payload.len() as u8,
                    payload: [0; 255],
                },
            };

            command_serial.command.payload[..payload.len()].copy_from_slice(payload);

            info!("sys cmd: {:x} payload: {:x}", opcode, payload);

            ptr::write_volatile(p_cmd_serial, command_serial);
        })
        .await;
    }
}
