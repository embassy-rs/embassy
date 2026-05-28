use embassy_futures::join;
use embassy_net_driver_channel as ch;
use smoltcp::wire::Ieee802154Frame;

use crate::net::commands::DataRequest;
use crate::net::iface::{Controller, ControllerToHostPacket, ControllerToHostPacketBox, mcps};
use crate::net::indications::write_frame_from_data_indication;
use crate::net::{MTU, ZeroCopyPubSub};

pub const BUF_SIZE: usize = 3;

pub struct Runner<'a, C: Controller> {
    ch: ch::Runner<'a, MTU>,
    controller: &'a C,

    events: &'a ZeroCopyPubSub<C::Packet>,
}

impl<'a, C: Controller> Runner<'a, C> {
    pub(crate) fn new(controller: &'a C, ch: ch::Runner<'a, MTU>, events: &'a ZeroCopyPubSub<C::Packet>) -> Self {
        Self { ch, controller, events }
    }

    pub async fn run(&mut self) -> ! {
        let (_state, mut rx, mut tx) = self.ch.borrow_split();

        join::join(
            async {
                loop {
                    let Ok(pkt) = self.controller.read().await else {
                        continue;
                    };

                    // TODO: respond to association requests, etc

                    match pkt.packet() {
                        ControllerToHostPacket::Mlme(_) => self.events.publish(pkt),
                        ControllerToHostPacket::Mcps(pkt) => match pkt {
                            mcps::Packet::Indication(mcps::IndicationPacket::Data(ind)) => {
                                let mut rx_buf = rx.rx_buf().await;
                                let len = write_frame_from_data_indication(ind, &mut *rx_buf);

                                rx_buf.rx_done(len);
                            }
                            _ => continue,
                        },
                    }
                }
            },
            async {
                loop {
                    let tx_buf = tx.tx_buf().await;
                    let b = &*tx_buf;
                    let frame = Ieee802154Frame::new_unchecked(&b);

                    let Ok(request) = DataRequest::try_from(frame) else {
                        warn!("failed to make data request");
                        continue;
                    };

                    if let Err(_e) = self.controller.write(&request).await {
                        warn!("failed to send pkt");
                    } else {
                        warn!("data frame sent!");
                    }

                    tx_buf.tx_done();
                }
            },
        )
        .await;

        loop {}
    }
}
