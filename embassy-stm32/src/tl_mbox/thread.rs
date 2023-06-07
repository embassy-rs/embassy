pub struct ThreadSubsystem;

impl ThreadSubsystem {
    /// TL_THREAD_Init
    pub fn new() -> Self {
        Self {}
    }

    pub async fn read(&mut self) {
        todo!()

        //        Ipcc::receive(channels::cpu2::IPCC_THREAD_NOTIFICATION_ACK_CHANNEL, || unsafe {
        //            // EventBox::from_node((*TL_REF_TABLE.as_ptr().read_volatile().thread_table).no_stack_buffer as *mut _)
        //        })
        //        .await
    }

    /// TL_Sys_SendCmd
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        todo!()

        //        Ipcc::send(channels::cpu1::IPCC_THREAD_OT_CMD_RSP_CHANNEL, || unsafe {
        //            // ((TL_CommandPacket_t *)(TL_RefTable.p_sys_table->pcmd_buffer))->cmdserial.type = TL_SYSCMD_PKT_TYPE;
        //            CommandPacket::copy_into_packet_from_slice(
        //                (*TL_REF_TABLE.as_ptr().read_volatile().sys_table).pcmd_buffer,
        //                buf,
        //                TlPacketType::SysCmd,
        //            );
        //        })
        //        .await;
        //
        //        Ok(buf.len())
    }
}
