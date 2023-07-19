use embassy_futures::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::mac::commands::DataRequest;
use crate::mac::event::{Event, MacEvent};
use crate::mac::typedefs::{AddressMode, MacAddress, PanId, SecurityLevel};
use crate::mac::MTU;
use crate::sub::mac::Mac;

pub struct Runner<'a> {
    mac_subsystem: Mac,
    pub(crate) rx_channel: Channel<CriticalSectionRawMutex, Event<'a>, 1>,
    pub(crate) tx_channel: Channel<CriticalSectionRawMutex, (&'a mut [u8; MTU], usize), 5>,
    pub(crate) tx_buf_channel: Channel<CriticalSectionRawMutex, &'a mut [u8; MTU], 5>,
}

impl<'a> Runner<'a> {
    pub fn new(mac: Mac, tx_buf_queue: [&'a mut [u8; MTU]; 5]) -> Self {
        let this = Self {
            mac_subsystem: mac,
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
                        match *mac_event {
                            MacEvent::McpsDataInd(_) => {
                                self.rx_channel.send(mac_event).await;
                            }
                            _ => {}
                        }
                    }
                }
            },
            async {
                loop {
                    let (buf, len) = self.tx_channel.recv().await;

                    self.mac_subsystem
                        .send_command(
                            DataRequest {
                                src_addr_mode: AddressMode::Short,
                                dst_addr_mode: AddressMode::Short,
                                dst_pan_id: PanId([0x1A, 0xAA]),
                                dst_address: MacAddress::BROADCAST,
                                msdu_handle: 0x02,
                                ack_tx: 0x00,
                                gts_tx: false,
                                security_level: SecurityLevel::Unsecure,
                                ..Default::default()
                            }
                            .set_buffer(&buf[..len]),
                        )
                        .await
                        .unwrap();

                    // The tx channel should always be of equal capacity to the tx_buf channel
                    self.tx_buf_channel.try_send(buf).unwrap();
                }
            },
        )
        .await;

        loop {}
    }
}
