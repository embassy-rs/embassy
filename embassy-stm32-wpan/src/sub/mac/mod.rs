use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::poll_once;
use embassy_stm32::ipcc::Ipcc;
use embassy_sync::waitqueue::AtomicWaker;

use self::commands::MacCommand;
use self::typedefs::MacStatus;
use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::{EvtBox, EvtPacket};
use crate::tables::{MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER};
use crate::{channels, evt};

mod opcodes;
pub mod commands;
pub mod responses;
pub mod typedefs;

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
    pub async fn read(&self) -> EvtBox<Self> {
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
    pub async fn write_and_get_response(&self, opcode: u16, payload: &[u8]) -> u8 {
        self.write(opcode, payload).await;
        Ipcc::flush(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL).await;

        unsafe {
            let p_event_packet = MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EvtPacket;
            let p_mac_rsp_evt = &((*p_event_packet).evt_serial.evt.payload) as *const u8;

            let evt_serial = (MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EvtPacket)
                .read_volatile()
                .evt_serial;
            let kind = (evt_serial).kind;
            let evt_code = evt_serial.evt.evt_code;
            let payload_len = evt_serial.evt.payload_len;
            let payload = evt_serial.evt.payload;

            debug!(
                "evt kind {} evt_code {} len {} payload {}",
                kind, evt_code, payload_len, payload
            );

            ptr::read_volatile(p_mac_rsp_evt)
        }
    }

    /// `TL_MAC_802_15_4_SendCmd`
    pub async fn write(&self, opcode: u16, payload: &[u8]) {
        Ipcc::send(channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL, || unsafe {
            CmdPacket::write_into(
                MAC_802_15_4_CMD_BUFFER.as_mut_ptr(),
                TlPacketType::MacCmd,
                opcode,
                payload,
            );
        })
        .await;

        unsafe {
            let typ = MAC_802_15_4_CMD_BUFFER.as_ptr().read_volatile().cmdserial.ty;
            let cmd_code = MAC_802_15_4_CMD_BUFFER.as_ptr().read_volatile().cmdserial.cmd.cmd_code;
            let payload_len = MAC_802_15_4_CMD_BUFFER
                .as_ptr()
                .read_volatile()
                .cmdserial
                .cmd
                .payload_len;
            let payload = MAC_802_15_4_CMD_BUFFER.as_ptr().read_volatile().cmdserial.cmd.payload;

            debug!(
                "serial type {} cmd_code {} len {} payload {}",
                typ, cmd_code, payload_len, payload
            );
        }
    }

    pub async fn send_command<T>(&self, cmd: T) -> Result<MacStatus, ()>
    where
        T: MacCommand,
    {
        let mut payload = [0u8; MAX_PACKET_SIZE];
        cmd.copy_into_slice(&mut payload);

        debug!("sending {:#x}", payload[..T::SIZE]);

        let response = self.write_and_get_response(T::OPCODE as u16, &payload[..T::SIZE]).await;

        MacStatus::try_from(response)
    }
}

const MAX_PACKET_SIZE: usize = 255;

impl evt::MemoryManager for Mac {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(_: *mut EvtPacket) {
        // Write the ack
        CmdPacket::write_into(
            MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _,
            TlPacketType::OtAck,
            0,
            &[],
        );

        // Clear the rx flag
        let _ = poll_once(Ipcc::receive::<bool>(
            channels::cpu2::IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL,
            || None,
        ));

        // Allow a new read call
        MAC_EVT_OUT.store(false, Ordering::SeqCst);
        MAC_WAKER.wake();
    }
}
