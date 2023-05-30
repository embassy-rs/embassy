use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::EventBox;
use super::tables::{
    Mac802_15_4Table, MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, TL_MAC_802_15_4_TABLE,
};
use super::{channels, Ipcc, TL_REF_TABLE};

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

    pub async fn read(&mut self) -> Result<EventBox, ()> {
        Ipcc::receive(channels::cpu2::IPCC_THREAD_NOTIFICATION_ACK_CHANNEL, || unsafe {
            EventBox::from_node((*TL_REF_TABLE.as_ptr().read_volatile().mac_802_15_4_table).pnotack_buffer as *mut _)
        })
        .await
    }

    /// TL_MAC_802_15_4_SendCmd
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        Ipcc::send(channels::cpu1::IPCC_THREAD_OT_CMD_RSP_CHANNEL, || unsafe {
            // ((TL_CommandPacket_t *)(TL_RefTable.p_sys_table->pcmd_buffer))->cmdserial.type = TL_SYSCMD_PKT_TYPE;
            CommandPacket::copy_into_packet_from_slice(
                (*TL_REF_TABLE.as_ptr().read_volatile().mac_802_15_4_table).pcmd_rsp_buffer,
                buf,
                TlPacketType::OtCmd,
            );
        })
        .await;

        Ok(buf.len())
    }
}
