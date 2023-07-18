#![allow(incomplete_features)]
#![deny(unused_must_use)]

use core::task::Context;

use embassy_net_driver::{Capabilities, LinkState, Medium};

use crate::mac::runner::Runner;
use crate::mac::MTU;

pub struct Driver<'d> {
    runner: &'d Runner,
}

impl<'d> Driver<'d> {
    pub(crate) fn new(runner: &'d Runner) -> Self {
        Self { runner: runner }
    }
}

impl<'d> embassy_net_driver::Driver for Driver<'d> {
    // type RxToken<'a> = RxToken<'a, 'd> where Self: 'a;
    // type TxToken<'a> = TxToken<'a, 'd> where Self: 'a;
    type RxToken<'a> = RxToken where Self: 'a;
    type TxToken<'a> = TxToken where Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.runner.rx_waker.register(cx.waker());

        // WAKER.register(cx.waker());
        //        if self.rx.available().is_some() && self.tx.available().is_some() {
        //            Some((RxToken { rx: &mut self.rx }, TxToken { tx: &mut self.tx }))
        //        } else {
        //            None
        //        }

        None
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        self.runner.tx_waker.register(cx.waker());

        // WAKER.register(cx.waker());
        // /        if self.tx.available().is_some() {
        // /            Some(TxToken { tx: &mut self.tx })
        // /        } else {
        // /            None
        // /        }

        None
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        // caps.max_burst_size = Some(self.tx.len());

        caps.medium = Medium::Ieee802154;
        caps
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
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

pub struct RxToken {
    // rx: &'a mut RDesRing<'d>,
}

impl embassy_net_driver::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        // let pkt = unwrap!(self.rx.available());

        let pkt = &mut [];
        let r = f(&mut pkt[0..]);
        // self.rx.pop_packet();
        r
    }
}

pub struct TxToken {
    // tx: &'a mut TDesRing<'d>,
}

impl embassy_net_driver::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        // let pkt = unwrap!(self.tx.available());
        let pkt = &mut [];
        let r = f(&mut pkt[..len]);
        // self.tx.transmit(len);
        r
    }
}
