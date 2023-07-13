use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::poll_once;
use embassy_stm32::ipcc::Ipcc;
use embassy_sync::waitqueue::AtomicWaker;

use self::commands::MacCommand;
use self::event::MacEvent;
use self::typedefs::MacError;
use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::{EvtBox, EvtPacket};
use crate::sub::mac;
use crate::tables::{MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER};
use crate::{channels, evt};

pub mod commands;
mod consts;
pub mod event;
mod helpers;
pub mod indications;
mod macros;
mod opcodes;
pub mod responses;
pub mod typedefs;

static MAC_WAKER: AtomicWaker = AtomicWaker::new();
static MAC_EVT_OUT: AtomicBool = AtomicBool::new(false);

pub struct Mac {
    mac: mac::Mac,
}

impl Mac {
    pub fn new(mac: mac::Mac) -> Self {
        Self { mac: mac }
    }

    pub async fn send_command<T>(&self, cmd: &T) -> Result<(), MacError>
    where
        T: MacCommand,
    {
        let mut payload = [0u8; MAX_PACKET_SIZE];
        cmd.copy_into_slice(&mut payload);

        debug!("sending {}", &payload[..T::SIZE]);

        let response = self
            .mac
            .tl_write_and_get_response(T::OPCODE as u16, &payload[..T::SIZE])
            .await;

        if response == 0x00 {
            Ok(())
        } else {
            Err(MacError::from(response))
        }
    }

    pub async fn read(&self) -> Result<MacEvent, ()> {
        let evt_box = self.mac.tl_read().await;
        let payload = evt_box.payload();

        MacEvent::try_from(payload)
    }
}

const MAX_PACKET_SIZE: usize = 255;
