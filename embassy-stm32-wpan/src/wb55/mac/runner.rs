use core::cell::RefCell;

use embassy_futures::join;
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use smoltcp::wire::Ieee802154FrameType;
use smoltcp::wire::ieee802154::Frame;

use crate::mac::MTU;
use crate::mac::commands::*;
use crate::mac::driver::NetworkState;
use crate::mac::event::MacEvent;
use crate::sub::mac::{MacRx, MacTx};

pub type ZeroCopyPubSub<M, T> = blocking_mutex::Mutex<M, RefCell<Option<Signal<NoopRawMutex, T>>>>;

pub const BUF_SIZE: usize = 3;

pub struct Runner<'a> {
    // rx event backpressure is already provided through the MacEvent drop mechanism
    // therefore, we don't need to worry about overwriting events
    rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
    rx_data_channel: &'a Channel<CriticalSectionRawMutex, MacEvent<'a>, 1>,
    mac_rx: Mutex<NoopRawMutex, &'a mut MacRx<'a>>,

    tx_data_channel: &'a Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), BUF_SIZE>,
    tx_buf_channel: &'a Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], BUF_SIZE>,
    mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx<'a>>,

    #[allow(unused)]
    network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
}

impl<'a> Runner<'a> {
    pub(crate) fn new(
        rx_event_channel: &'a ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
        rx_data_channel: &'a Channel<CriticalSectionRawMutex, MacEvent<'a>, 1>,
        mac_rx: &'a mut MacRx<'a>,
        tx_data_channel: &'a Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), BUF_SIZE>,
        tx_buf_channel: &'a Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], BUF_SIZE>,
        mac_tx: &'a Mutex<CriticalSectionRawMutex, MacTx<'a>>,
        tx_buf_queue: &'a mut [[u8; MTU]; BUF_SIZE],
        network_state: &'a blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
        short_address: [u8; 2],
        mac_address: [u8; 8],
    ) -> Self {
        for buf in tx_buf_queue {
            tx_buf_channel.try_send(buf).unwrap();
        }

        critical_section::with(|cs| {
            let mut network_state = network_state.borrow(cs).borrow_mut();

            network_state.mac_addr = mac_address;
            network_state.short_addr = short_address;
        });

        Self {
            rx_event_channel,
            rx_data_channel,
            mac_rx: Mutex::new(mac_rx),
            tx_data_channel,
            tx_buf_channel,
            mac_tx,
            network_state,
        }
    }

    async fn send_request<T: MacCommand, U: TryInto<T>>(&self, frame: U) -> Result<(), ()>
    where
        (): From<<U as TryInto<T>>::Error>,
    {
        let request: T = frame.try_into()?;
        self.mac_tx.lock().await.send_command(&request).await.map_err(|_| ())?;

        Ok(())
    }

    pub async fn run(&'a self) -> ! {
        join::join(
            async {
                loop {
                    if let Ok(mac_event) = self.mac_rx.try_lock().unwrap().read().await {
                        match mac_event {
                            MacEvent::MlmeAssociateCnf(_)
                            | MacEvent::MlmeDisassociateCnf(_)
                            | MacEvent::MlmeGetCnf(_)
                            | MacEvent::MlmeGtsCnf(_)
                            | MacEvent::MlmeResetCnf(_)
                            | MacEvent::MlmeRxEnableCnf(_)
                            | MacEvent::MlmeScanCnf(_)
                            | MacEvent::MlmeSetCnf(_)
                            | MacEvent::MlmeStartCnf(_)
                            | MacEvent::MlmePollCnf(_)
                            | MacEvent::MlmeDpsCnf(_)
                            | MacEvent::MlmeSoundingCnf(_)
                            | MacEvent::MlmeCalibrateCnf(_)
                            | MacEvent::McpsDataCnf(_)
                            | MacEvent::McpsPurgeCnf(_) => {
                                self.rx_event_channel.lock(|s| {
                                    s.borrow().as_ref().map(|signal| signal.signal(mac_event));
                                });
                            }
                            MacEvent::McpsDataInd(_) => {
                                // Pattern should match driver
                                self.rx_data_channel.send(mac_event).await;
                            }
                            _ => {
                                debug!("unhandled mac event: {:#x}", mac_event);
                            }
                        }
                    }
                }
            },
            async {
                loop {
                    let (buf, _) = self.tx_data_channel.receive().await;

                    // Smoltcp has created this frame, so there's no need to reparse it.
                    let frame = Frame::new_unchecked(&buf);

                    let result: Result<(), ()> = match frame.frame_type() {
                        Ieee802154FrameType::Beacon => Err(()),
                        Ieee802154FrameType::Data => self.send_request::<DataRequest, _>(frame).await,
                        Ieee802154FrameType::Acknowledgement => Err(()),
                        Ieee802154FrameType::MacCommand => Err(()),
                        Ieee802154FrameType::Multipurpose => Err(()),
                        Ieee802154FrameType::FragmentOrFrak => Err(()),
                        Ieee802154FrameType::Extended => Err(()),
                        _ => Err(()),
                    };

                    if result.is_err() {
                        debug!("failed to parse mac frame");
                    } else {
                        trace!("data frame sent!");
                    }

                    // The tx channel should always be of equal capacity to the tx_buf channel
                    self.tx_buf_channel.try_send(buf).unwrap();
                }
            },
        )
        .await;

        loop {}
    }
}
