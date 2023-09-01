use core::cell::RefCell;

use embassy_futures::join;
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;

use crate::mac::commands::DataRequest;
use crate::mac::event::MacEvent;
use crate::mac::typedefs::{AddressMode, MacAddress, PanId, SecurityLevel};
use crate::mac::MTU;
use crate::sub::mac::Mac;

type ZeroCopyPubSub<M, T> = blocking_mutex::Mutex<M, RefCell<Option<Signal<NoopRawMutex, T>>>>;

pub struct Runner<'a> {
    pub(crate) mac_subsystem: Mac,
    // rx event backpressure is already provided through the MacEvent drop mechanism
    // therefore, we don't need to worry about overwriting events
    pub(crate) rx_event_channel: ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'a>>,
    pub(crate) read_mutex: Mutex<CriticalSectionRawMutex, ()>,
    pub(crate) write_mutex: Mutex<CriticalSectionRawMutex, ()>,
    pub(crate) rx_channel: Channel<CriticalSectionRawMutex, MacEvent<'a>, 1>,
    pub(crate) tx_channel: Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), 5>,
    pub(crate) tx_buf_channel: Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], 5>,
}

impl<'a> Runner<'a> {
    pub fn new(mac: Mac, tx_buf_queue: [&'a mut [u8; MTU]; 5]) -> Self {
        let this = Self {
            mac_subsystem: mac,
            rx_event_channel: blocking_mutex::Mutex::new(RefCell::new(None)),
            read_mutex: Mutex::new(()),
            write_mutex: Mutex::new(()),
            rx_channel: Channel::new(),
            tx_channel: Channel::new(),
            tx_buf_channel: Channel::new(),
        };

        for buf in tx_buf_queue {
            this.tx_buf_channel.try_send(buf).unwrap();
        }

        this
    }

    pub async fn run(&'a self) -> ! {
        join::join(
            async {
                loop {
                    if let Ok(mac_event) = self.mac_subsystem.read().await {
                        match mac_event {
                            MacEvent::McpsDataInd(_) => {
                                self.rx_channel.send(mac_event).await;
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
                    let (buf, len) = self.tx_channel.receive().await;
                    let _wm = self.write_mutex.lock().await;

                    // The mutex should be dropped on the next loop iteration
                    self.mac_subsystem
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
