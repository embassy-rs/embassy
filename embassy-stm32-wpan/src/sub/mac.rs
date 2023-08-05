use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::poll_once;
use embassy_stm32::ipcc::Ipcc;
use embassy_sync::waitqueue::AtomicWaker;

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::{EvtBox, EvtPacket};
use crate::mac::commands::MacCommand;
use crate::mac::event::MacEvent;
use crate::mac::typedefs::MacError;
use crate::tables::{MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER};
use crate::{channels, evt};

static MAC_WAKER: AtomicWaker = AtomicWaker::new();
static MAC_EVT_OUT: AtomicBool = AtomicBool::new(false);

pub struct Mac {
    phantom: PhantomData<Mac>,
}

impl Mac {
    pub(crate) fn new() -> Self {
        Self { phantom: PhantomData }
    }

    /// `HW_IPCC_MAC_802_15_4_EvtNot`
    ///
    /// This function will stall if the previous `EvtBox` has not been dropped
    pub async fn tl_read(&self) -> EvtBox<Self> {
        // Wait for the last event box to be dropped
        poll_fn(|cx| {
            MAC_WAKER.register(cx.waker());
            if MAC_EVT_OUT.load(Ordering::SeqCst) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        // Return a new event box
        Ipcc::receive(channels::cpu2::IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL, || unsafe {
            // The closure is not async, therefore the closure must execute to completion (cannot be dropped)
            // Therefore, the event box is guaranteed to be cleaned up if it's not leaked
            MAC_EVT_OUT.store(true, Ordering::SeqCst);

            Some(EvtBox::new(MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _))
        })
        .await
    }

    /// `HW_IPCC_MAC_802_15_4_CmdEvtNot`
    pub async fn tl_write_and_get_response(&self, opcode: u16, payload: &[u8]) -> u8 {
        self.tl_write(opcode, payload).await;
        Ipcc::flush(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL).await;

        unsafe {
            let p_event_packet = MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EvtPacket;
            let p_mac_rsp_evt = &((*p_event_packet).evt_serial.evt.payload) as *const u8;

            ptr::read_volatile(p_mac_rsp_evt)
        }
    }

    /// `TL_MAC_802_15_4_SendCmd`
    pub async fn tl_write(&self, opcode: u16, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL, || unsafe {
            CmdPacket::write_into(
                MAC_802_15_4_CMD_BUFFER.as_mut_ptr(),
                TlPacketType::MacCmd,
                opcode,
                payload,
            );
        })
        .await;
    }

    pub async fn send_command<T>(&self, cmd: &T) -> Result<(), MacError>
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

    pub async fn read(&self) -> Result<MacEvent<'_>, ()> {
        MacEvent::new(self.tl_read().await)
    }
}

impl evt::MemoryManager for Mac {
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
        let _ = poll_once(Ipcc::receive::<()>(
            channels::cpu2::IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL,
            || None,
        ));

        // Allow a new read call
        MAC_EVT_OUT.store(false, Ordering::SeqCst);
        MAC_WAKER.wake();
    }
}
