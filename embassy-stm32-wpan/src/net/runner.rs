use embassy_futures::join;
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::PacketBuf;

use crate::net::commands::DataRequest;
use crate::net::iface::{Controller, ControllerToHostPacket, ControllerToHostPacketBox, mcps};
use crate::net::indications::write_frame_from_data_indication;
use crate::net::{MTU, ZeroCopyPubSub};

pub const BUF_SIZE: usize = 3;

pub struct Runner<'a, C: Controller> {
    ch: ch::Runner<'a>,
    controller: &'a C,

    events: &'a ZeroCopyPubSub<C::Packet>,
}

impl<'a, C: Controller> Runner<'a, C> {
    pub(crate) fn new(controller: &'a C, ch: ch::Runner<'a>, events: &'a ZeroCopyPubSub<C::Packet>) -> Self {
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
                                rx.rx_ready().await;
                                let Some(mut rx_buf) = PacketBuf::try_new() else {
                                    warn!("packet pool empty, dropping received frame");
                                    continue;
                                };
                                rx_buf.set_len(MTU);
                                let len = write_frame_from_data_indication(ind, &mut rx_buf);
                                rx_buf.set_len(len);

                                rx.rx(rx_buf).await;
                            }
                            _ => continue,
                        },
                    }
                }
            },
            async {
                loop {
                    let tx_buf = tx.tx().await;

                    let Ok(request) = DataRequest::try_from(&tx_buf[..]) else {
                        warn!("failed to make data request");
                        continue;
                    };

                    if let Err(_e) = self.controller.write(&request).await {
                        warn!("failed to send pkt");
                    } else {
                        warn!("data frame sent!");
                    }
                }
            },
        )
        .await;

        loop {}
    }
}
