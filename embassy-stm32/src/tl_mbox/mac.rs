use core::{mem, ptr};

use atomic_polyfill::{compiler_fence, Ordering};

use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::EventPacket;
use super::tables::{
    Mac802_15_4Table, MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, TL_MAC_802_15_4_TABLE,
};
use super::unsafe_linked_list::LinkedListNode;
use super::{channels, mm, Ipcc};

pub struct MacSubsystem;

impl MacSubsystem {
    /// TL_MAC_802_15_4_Init
    pub fn new() -> Self {
        unsafe {
            TL_MAC_802_15_4_TABLE.as_mut_ptr().write_volatile(Mac802_15_4Table {
                pcmd_rsp_buffer: MAC_802_15_4_CMD_BUFFER.as_mut_ptr().cast(),
                pnotack_buffer: MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr().cast(),
                evt_queue: core::ptr::null_mut(),
            });
        }

        Self {}
    }

    /// `HW_IPCC_MAC_802_15_4_EvtNot`
    pub async fn read<R>(&mut self, r: impl Fn(&EventPacket) -> R) -> R {
        Ipcc::receive(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, || unsafe {
            let node = MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut LinkedListNode;
            let mut node_ptr = core::ptr::null_mut();
            if LinkedListNode::is_empty(node) {
                // `TL_MAC_802_15_4_SendAck` resets the buffer before requesting more
                let command_packet = MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut CommandPacket;
                let typ = &command_packet.read().cmd_serial.typ as *const _ as *mut u8;
                typ.write_volatile(TlPacketType::OtAck as u8);

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

    /// `HW_IPCC_MAC_802_15_4_CmdEvtNot`
    pub async fn write_and_get_response<R>(
        &mut self,
        f: impl FnOnce(&mut CommandPacket),
        r: impl FnOnce(&EventPacket) -> R,
    ) -> R {
        self.write(f).await;
        Ipcc::flush(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL).await;

        let event_packet = unsafe { MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EventPacket };
        let event_packet = unsafe { mem::transmute(event_packet) };

        r(event_packet)
    }

    /// `TL_MAC_802_15_4_SendCmd`
    pub async fn write(&mut self, f: impl FnOnce(&mut CommandPacket)) {
        Ipcc::send(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL, || unsafe {
            f(&mut MAC_802_15_4_CMD_BUFFER.assume_init());

            let command_packet = MAC_802_15_4_CMD_BUFFER.as_mut_ptr() as *mut CommandPacket;
            let typ = &command_packet.read().cmd_serial.typ as *const _ as *mut u8;
            typ.write_volatile(TlPacketType::OtCmd as u8);
        })
        .await;
    }
}
