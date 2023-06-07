use core::{mem, ptr};

use atomic_polyfill::{compiler_fence, Ordering};

use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::{CommandChannelEvent, EventPacket, EventSerial};
use super::shci::{SchiCommandStatus, ShciBleInitCmdParam, ShciOpcode};
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

    #[cfg(feature = "ble")]
    pub async fn schi_c2_ble_init(&mut self, param: ShciBleInitCmdParam) -> SchiCommandStatus {
        self.write_and_get_response(
            |command_packet| unsafe {
                let mut cmd_serial = CommandSerial::default();
                cmd_serial.cmd.cmd_code = ShciOpcode::BleInit as u16;
                cmd_serial.cmd.payload_len = mem::size_of::<ShciBleInitCmdParam>() as u8;
                cmd_serial.typ = TlPacketType::SysCmd as u8;

                ptr::write(&cmd_serial.cmd.payload as *const _ as *mut _, param);
                ptr::write_volatile(&mut command_packet.read_volatile().cmd_serial, cmd_serial);
            },
            |command_event| unsafe {
                // let command_event = command_event.read_volatile();
                // let payload = command_event.payload[0];

                let event_serial = SYS_CMD_BUF.as_mut_ptr() as *const EventSerial;
                let command_event = event_serial.read_volatile().event.payload.as_ptr() as *const CommandChannelEvent;

                let payload = command_event.read_volatile().payload[0];

                info!("payload: {}", payload);

                // payload.try_into().unwrap()
                SchiCommandStatus::ShciErrInvalidHciCmdParams
            },
        )
        .await
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
    pub async fn write_and_get_response<R>(
        &mut self,
        f: impl FnOnce(*mut CommandPacket),
        mut r: impl FnMut(*const CommandChannelEvent) -> R,
    ) -> R {
        self.write(f).await;
        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;

        let event_serial = unsafe { SYS_CMD_BUF.as_ptr() as *const EventSerial };
        let command_event =
            unsafe { event_serial.read_volatile().event.payload.as_ptr() as *const CommandChannelEvent };

        r(command_event)
    }

    /// `TL_Sys_SendCmd`
    pub async fn write(&mut self, f: impl FnOnce(*mut CommandPacket)) {
        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || unsafe {
            f(SYS_CMD_BUF.as_mut_ptr());

            // let command_packet = SYS_CMD_BUF.as_mut_ptr() as *mut CommandPacket;
            // let typ = &command_packet.read().cmd_serial.typ as *const _ as *mut u8;
            // typ.write_volatile(TlPacketType::SysCmd as u8);
        })
        .await;
    }
}
