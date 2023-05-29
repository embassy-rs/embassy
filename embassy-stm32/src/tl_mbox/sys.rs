use super::cmd::CommandPacket;
use super::consts::TlPacketType;
use super::evt::{CommandChannelEvent, EventBox};
use super::tables::{SysTable, SYSTEM_EVT_QUEUE, SYS_CMD_BUF, TL_REF_TABLE, TL_SYS_TABLE};
use super::unsafe_linked_list::LinkedListNode;
use super::{channels, Ipcc};

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

    pub async fn read(&mut self) -> Result<EventBox, ()> {
        Ipcc::receive(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, || unsafe {
            EventBox::from_node((*TL_REF_TABLE.as_ptr().read_volatile().sys_table).sys_queue as *mut _)
        })
        .await
    }

    /// TL_Sys_SendCmd
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        Ipcc::send(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, || unsafe {
            // ((TL_CommandPacket_t *)(TL_RefTable.p_sys_table->pcmd_buffer))->cmdserial.type = TL_SYSCMD_PKT_TYPE;
            CommandPacket::copy_into_packet_from_slice(
                (*TL_REF_TABLE.as_ptr().read_volatile().sys_table).pcmd_buffer,
                buf,
                TlPacketType::SysCmd,
            );
        })
        .await;

        Ok(buf.len())
    }

    pub async fn wait_for_command_complete(&mut self) -> Result<CommandChannelEvent, ()> {
        Ipcc::flush(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL).await;

        unsafe {
            CommandChannelEvent::from_node((*TL_REF_TABLE.as_ptr().read_volatile().sys_table).pcmd_buffer as *mut _)
        }
    }
}
