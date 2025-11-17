use core::cell::RefCell;
use core::future::Future;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task;
use core::task::Poll;

use embassy_net_driver::LinkState;
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use futures_util::FutureExt;

use crate::mac::commands::*;
use crate::mac::driver::NetworkState;
use crate::mac::event::MacEvent;
use crate::mac::runner::ZeroCopyPubSub;
use crate::mac::typedefs::*;
use crate::sub::mac::MacTx;

pub struct Control<'a> {
    rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
    mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx>,
    #[allow(unused)]
    network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
}

impl<'a> Control<'a> {
    pub(crate) fn new(
        rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
        mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx>,
        network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
    ) -> Self {
        Self {
            rx_event_channel,
            mac_tx,
            network_state,
        }
    }

    pub async fn init_link(&mut self, pan_id: [u8; 2]) {
        debug!("resetting");

        debug!(
            "{:#x}",
            self.send_command_and_get_response(&ResetRequest {
                set_default_pib: true,
                ..Default::default()
            })
            .await
            .unwrap()
            .await
        );

        let (short_address, mac_address) = critical_section::with(|cs| {
            let mut network_state = self.network_state.borrow(cs).borrow_mut();

            network_state.pan_id = pan_id;

            (network_state.short_addr, network_state.mac_addr)
        });

        debug!("setting extended address");
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&SetRequest {
                pib_attribute_ptr: &u64::from_be_bytes(mac_address) as *const _ as *const u8,
                pib_attribute: PibId::ExtendedAddress,
            })
            .await
            .unwrap()
            .await
        );

        debug!("setting short address");
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&SetRequest {
                pib_attribute_ptr: &u16::from_be_bytes(short_address) as *const _ as *const u8,
                pib_attribute: PibId::ShortAddress,
            })
            .await
            .unwrap()
            .await
        );

        debug!("setting association permit");
        let association_permit: bool = true;
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&SetRequest {
                pib_attribute_ptr: &association_permit as *const _ as *const u8,
                pib_attribute: PibId::AssociationPermit,
            })
            .await
            .unwrap()
            .await
        );

        debug!("setting TX power");
        let transmit_power: i8 = 2;
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&SetRequest {
                pib_attribute_ptr: &transmit_power as *const _ as *const u8,
                pib_attribute: PibId::TransmitPower,
            })
            .await
            .unwrap()
            .await
        );

        debug!("starting FFD device");
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&StartRequest {
                pan_id: PanId(pan_id),
                channel_number: MacChannel::Channel16,
                beacon_order: 0x0F,
                superframe_order: 0x0F,
                pan_coordinator: true,
                battery_life_extension: false,
                ..Default::default()
            })
            .await
            .unwrap()
            .await
        );

        debug!("setting RX on when idle");
        let rx_on_while_idle: bool = true;
        debug!(
            "{:#x}",
            self.send_command_and_get_response(&SetRequest {
                pib_attribute_ptr: &rx_on_while_idle as *const _ as *const u8,
                pib_attribute: PibId::RxOnWhenIdle,
            })
            .await
            .unwrap()
            .await
        );

        critical_section::with(|cs| {
            let mut network_state = self.network_state.borrow(cs).borrow_mut();

            network_state.link_state = LinkState::Up;
            network_state.link_waker.wake();
        });
    }

    pub async fn send_command<T>(&self, cmd: &T) -> Result<(), MacError>
    where
        T: MacCommand,
    {
        self.mac_tx.lock().await.send_command(cmd).await
    }

    pub async fn send_command_and_get_response<T>(&self, cmd: &T) -> Result<EventToken<'a>, MacError>
    where
        T: MacCommand,
    {
        let token = EventToken::new(self.rx_event_channel);

        compiler_fence(Ordering::Release);

        self.mac_tx.lock().await.send_command(cmd).await?;

        Ok(token)
    }
}

pub struct EventToken<'a> {
    rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
}

impl<'a> EventToken<'a> {
    pub(crate) fn new(rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>) -> Self {
        // Enable event receiving
        rx_event_channel.lock(|s| {
            *s.borrow_mut() = Some(Signal::new());
        });

        Self { rx_event_channel }
    }
}

impl<'a> Future for EventToken<'a> {
    type Output = MacEvent<'a>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.rx_event_channel
            .lock(|s| s.borrow_mut().as_mut().unwrap().wait().poll_unpin(cx))
    }
}

impl<'a> Drop for EventToken<'a> {
    fn drop(&mut self) {
        // Disable event receiving
        // This will also drop the contained event, if it exists, and will free up receiving the next event
        self.rx_event_channel.lock(|s| {
            *s.borrow_mut() = None;
        });
    }
}
