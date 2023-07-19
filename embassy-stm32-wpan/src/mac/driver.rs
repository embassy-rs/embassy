#![allow(incomplete_features)]
#![deny(unused_must_use)]

use core::task::Context;

use embassy_net_driver::{Capabilities, LinkState, Medium};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::mac::event::{Event, MacEvent};
use crate::mac::runner::Runner;
use crate::mac::MTU;

pub struct Driver<'d> {
    runner: &'d Runner<'d>,
}

impl<'d> Driver<'d> {
    pub(crate) fn new(runner: &'d Runner<'d>) -> Self {
        Self { runner: runner }
    }
}

impl<'d> embassy_net_driver::Driver for Driver<'d> {
    // type RxToken<'a> = RxToken<'a, 'd> where Self: 'a;
    // type TxToken<'a> = TxToken<'a, 'd> where Self: 'a;
    type RxToken<'a> = RxToken<'d> where Self: 'a;
    type TxToken<'a> = TxToken<'d> where Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.runner.rx_channel.poll_ready_to_receive(cx) && self.runner.tx_buf_channel.poll_ready_to_receive(cx) {
            Some((
                RxToken {
                    rx: &self.runner.rx_channel,
                },
                TxToken {
                    tx: &self.runner.tx_channel,
                    tx_buf: &self.runner.tx_buf_channel,
                },
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        if self.runner.tx_buf_channel.poll_ready_to_receive(cx) {
            Some(TxToken {
                tx: &self.runner.tx_channel,
                tx_buf: &self.runner.tx_buf_channel,
            })
        } else {
            None
        }
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        // caps.max_burst_size = Some(self.tx.len());

        caps.medium = Medium::Ieee802154;
        caps
    }

    fn link_state(&mut self, _cx: &mut Context) -> LinkState {
        //        if self.phy.poll_link(&mut self.station_management, cx) {
        //            LinkState::Up
        //        } else {
        //            LinkState::Down
        //        }

        LinkState::Down
    }

    fn ethernet_address(&self) -> [u8; 6] {
        // self.mac_addr

        [0; 6]
    }
}

pub struct RxToken<'d> {
    rx: &'d Channel<CriticalSectionRawMutex, Event<'d>, 1>,
}

impl<'d> embassy_net_driver::RxToken for RxToken<'d> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // Only valid data events should be put into the queue

        let data_event = match *self.rx.try_recv().unwrap() {
            MacEvent::McpsDataInd(data_event) => data_event,
            _ => unreachable!(),
        };

        f(&mut data_event.payload())
    }
}

pub struct TxToken<'d> {
    tx: &'d Channel<CriticalSectionRawMutex, (&'d mut [u8; MTU], usize), 5>,
    tx_buf: &'d Channel<CriticalSectionRawMutex, &'d mut [u8; MTU], 5>,
}

impl<'d> embassy_net_driver::TxToken for TxToken<'d> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // Only valid tx buffers should be put into the queue
        let buf = self.tx_buf.try_recv().unwrap();
        let r = f(&mut buf[..len]);

        // The tx channel should always be of equal capacity to the tx_buf channel
        self.tx.try_send((buf, len)).unwrap();

        r
    }
}
