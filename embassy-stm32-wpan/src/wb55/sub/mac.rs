use core::future::poll_fn;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::poll_once;
use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};
use embassy_sync::waitqueue::AtomicWaker;

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt;
use crate::evt::{EvtBox, EvtPacket};
use crate::mac::commands::MacCommand;
use crate::mac::event::MacEvent;
use crate::mac::typedefs::MacError;
use crate::tables::{MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER};
use crate::unsafe_linked_list::LinkedListNode;

static MAC_WAKER: AtomicWaker = AtomicWaker::new();
static MAC_EVT_OUT: AtomicBool = AtomicBool::new(false);

pub struct Mac<'a> {
    ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
    ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> Mac<'a> {
    pub(crate) fn new(
        ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
        ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
    ) -> Self {
        use crate::tables::{
            MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, Mac802_15_4Table, TL_MAC_802_15_4_TABLE,
            TL_TRACES_TABLE, TRACES_EVT_QUEUE, TracesTable,
        };

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
    pub async fn tl_read(&mut self) -> EvtBox<MacRx<'a>> {
        // Wait for the last event box to be dropped
        poll_fn(|cx| {
            MAC_WAKER.register(cx.waker());
            if MAC_EVT_OUT.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        // Return a new event box
        self.ipcc_mac_802_15_4_notification_ack_channel
            .receive(|| unsafe {
                // The closure is not async, therefore the closure must execute to completion (cannot be dropped)
                // Therefore, the event box is guaranteed to be cleaned up if it's not leaked
                MAC_EVT_OUT.store(true, Ordering::SeqCst);

                Some(EvtBox::new(MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _))
            })
            .await
    }

    pub async fn read<'b>(&mut self) -> Result<MacEvent<'b>, ()> {
        MacEvent::new(self.tl_read().await)
    }
}

impl<'a> evt::MemoryManager for MacRx<'a> {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(_: *mut EvtPacket) {
        trace!("mac drop event");

        // Write the ack
        CmdPacket::write_into(
            MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _,
            TlPacketType::OtAck,
            0,
            &[],
        );

        // Clear the rx flag
        let _ = poll_once(Ipcc::receive::<()>(3, || None));

        // Allow a new read call
        MAC_EVT_OUT.store(false, Ordering::Release);
        MAC_WAKER.wake();
    }
}
