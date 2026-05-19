use core::ptr;

use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};

use crate::mac::commands::MacCommand;
use crate::mac::event::MacEvent;
use crate::mac::typedefs::MacError;
use crate::util::Flag;
use crate::wb::channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL;
use crate::wb::cmd::CmdPacket;
use crate::wb::consts::TlPacketType;
use crate::wb::evt::{self, EvtBox, EvtPacket};
use crate::wb::tables::{
    MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, Mac802_15_4Table, TL_MAC_802_15_4_TABLE,
    TL_TRACES_TABLE, TRACES_EVT_QUEUE, TracesTable,
};
use crate::wb::unsafe_linked_list::LinkedListNode;

static MAC_EVT_OUT: Flag = Flag::new(false);

pub struct Mac<'a> {
    ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
    ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> Mac<'a> {
    pub(crate) fn new(
        ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
        ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(TRACES_EVT_QUEUE.as_mut_ptr() as *mut _);

            TL_TRACES_TABLE.as_mut_ptr().write_volatile(TracesTable {
                traces_queue: TRACES_EVT_QUEUE.as_ptr() as *const _,
            });

            TL_MAC_802_15_4_TABLE.as_mut_ptr().write_volatile(Mac802_15_4Table {
                p_cmdrsp_buffer: MAC_802_15_4_CMD_BUFFER.as_mut_ptr().cast(),
                p_notack_buffer: MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr().cast(),
                evt_queue: core::ptr::null_mut(),
            });
        };

        Self {
            ipcc_mac_802_15_4_cmd_rsp_channel,
            ipcc_mac_802_15_4_notification_ack_channel,
        }
    }

    pub const fn split(self) -> (MacRx<'a>, MacTx<'a>) {
        (
            MacRx {
                ipcc_mac_802_15_4_notification_ack_channel: self.ipcc_mac_802_15_4_notification_ack_channel,
            },
            MacTx {
                ipcc_mac_802_15_4_cmd_rsp_channel: self.ipcc_mac_802_15_4_cmd_rsp_channel,
            },
        )
    }
}

pub struct MacTx<'a> {
    ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
}

impl<'a> MacTx<'a> {
    /// `HW_IPCC_MAC_802_15_4_CmdEvtNot`
    pub async fn tl_write_and_get_response(&mut self, opcode: u16, payload: &[u8]) -> u8 {
        self.tl_write(opcode, payload).await;
        self.ipcc_mac_802_15_4_cmd_rsp_channel.flush().await;

        unsafe {
            let p_event_packet = MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EvtPacket;
            let p_mac_rsp_evt = &((*p_event_packet).evt_serial.evt.payload) as *const u8;

            ptr::read_volatile(p_mac_rsp_evt)
        }
    }

    /// `TL_MAC_802_15_4_SendCmd`
    pub async fn tl_write(&mut self, opcode: u16, payload: &[u8]) {
        self.ipcc_mac_802_15_4_cmd_rsp_channel
            .send(|| unsafe {
                CmdPacket::write_into(
                    MAC_802_15_4_CMD_BUFFER.as_mut_ptr(),
                    TlPacketType::MacCmd,
                    opcode,
                    payload,
                );
            })
            .await;
    }

    pub async fn send_command<T>(&mut self, cmd: &T) -> Result<(), MacError>
    where
        T: MacCommand,
    {
        let response = self.tl_write_and_get_response(T::OPCODE as u16, cmd.payload()).await;

        if response == 0x00 {
            Ok(())
        } else {
            Err(MacError::from(response))
        }
    }
}

pub struct MacRx<'a> {
    ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> MacRx<'a> {
    /// `HW_IPCC_MAC_802_15_4_EvtNot`
    ///
    /// This function will stall if the previous `EvtBox` has not been dropped
    pub async fn read(&mut self) -> Result<MacEvent<'a>, ()> {
        MAC_EVT_OUT.wait_for_low().await;

        // Return a new event box
        self.ipcc_mac_802_15_4_notification_ack_channel
            .receive(|| unsafe {
                Some(MacEvent::new(EvtBox::new(
                    MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _,
                )))
            })
            .await
    }
}

impl<'a> evt::MemoryManager for Mac<'a> {
    unsafe fn new_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _) {
            MAC_EVT_OUT.set_high();
        }
    }

    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _) {
            trace!("mac drop event");

            // Write the ack
            CmdPacket::write_into(
                MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _,
                TlPacketType::OtAck,
                0,
                &[],
            );

            // Clear the rx flag
            Ipcc::clear(IPCC_MAC_802_15_4_CMD_RSP_CHANNEL as u8);
            MAC_EVT_OUT.set_low();
        }
    }
}
