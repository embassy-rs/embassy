use core::cell::RefCell;

use embassy_futures::join;
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;

use crate::mac::MTU;
use crate::mac::commands::*;
use crate::mac::driver::NetworkState;
use crate::mac::event::MacEvent;
use crate::mac::typedefs::*;
use crate::sub::mac::{MacRx, MacTx};

pub type ZeroCopyPubSub<M, T> = blocking_mutex::Mutex<M, RefCell<Option<Signal<NoopRawMutex, T>>>>;

pub const BUF_SIZE: usize = 3;

pub struct Runner<'a> {
    // rx event backpressure is already provided through the MacEvent drop mechanism
    // therefore, we don't need to worry about overwriting events
    rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
    rx_data_channel: &'a Channel<CriticalSectionRawMutex, MacEvent<'a>, 1>,
    mac_rx: &'a mut MacRx,

    tx_data_channel: &'a Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), BUF_SIZE>,
    tx_buf_channel: &'a Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], BUF_SIZE>,
    mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx>,

    #[allow(unused)]
    network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
}

impl<'a> Runner<'a> {
    pub fn new(
        rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
        rx_data_channel: &'a Channel<CriticalSectionRawMutex, MacEvent<'a>, 1>,
        mac_rx: &'a mut MacRx,
        tx_data_channel: &'a Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), BUF_SIZE>,
        tx_buf_channel: &'a Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], BUF_SIZE>,
        mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx>,
        tx_buf_queue: &'a mut [[u8; MTU]; BUF_SIZE],
        network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
    ) -> Self {
        for buf in tx_buf_queue {
            tx_buf_channel.try_send(buf).unwrap();
        }

        Self {
            rx_event_channel,
            rx_data_channel,
            mac_rx,
            tx_data_channel,
            tx_buf_channel,
            mac_tx,
            network_state,
        }
    }

    pub async fn run(&'a self) -> ! {
        join::join(
            async {
                loop {
                    if let Ok(mac_event) = self.mac_rx.read().await {
                        match mac_event {
                            MacEvent::McpsDataInd(_) => {
                                self.rx_data_channel.send(mac_event).await;
                            }
                            _ => {
                                self.rx_event_channel.lock(|s| {
                                    match &*s.borrow() {
                                        Some(signal) => {
                                            signal.signal(mac_event);
                                        }
                                        None => {}
                                    };
                                });
                            }
                        }
                    }
                }
            },
            async {
                let mut msdu_handle = 0x02;

                loop {
                    let (buf, len) = self.tx_data_channel.receive().await;
                    let mac_tx = self.mac_tx.lock().await;

                    // TODO: skip this if the link state is down

                    // The mutex should be dropped on the next loop iteration
                    mac_tx
                        .send_command(
                            DataRequest {
                                src_addr_mode: AddressMode::Short,
                                dst_addr_mode: AddressMode::Short,
                                dst_pan_id: PanId([0x1A, 0xAA]),
                                dst_address: MacAddress::BROADCAST,
                                msdu_handle: msdu_handle,
                                ack_tx: 0x00,
                                gts_tx: false,
                                security_level: SecurityLevel::Unsecure,
                                ..Default::default()
                            }
                            .set_buffer(&buf[..len]),
                        )
                        .await
                        .unwrap();

                    msdu_handle = msdu_handle.wrapping_add(1);

                    // The tx channel should always be of equal capacity to the tx_buf channel
                    self.tx_buf_channel.try_send(buf).unwrap();
                }
            },
        )
        .await;

        loop {}
    }
}
