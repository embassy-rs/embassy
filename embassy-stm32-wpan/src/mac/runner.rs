use embassy_futures::select::{select3, Either3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::waitqueue::AtomicWaker;

use crate::mac::event::{Event, MacEvent};
use crate::mac::MTU;
use crate::sub::mac::Mac;

pub(crate) struct TxRing {
    // stores n packets of up to mtu size
    ring: [[u8; MTU]; 5],
    pending: bool,
    //    start: u8,
    //    end: u8,
}

impl TxRing {
    pub(crate) fn new() -> Self {
        Self {
            ring: [[0; MTU]; 5],
            pending: false,
        }
    }

    // wait for a free packet to become available
    pub fn is_packet_free(&self) -> bool {
        !self.pending
    }

    // get the next available free packet
    pub fn get_free_packet<'a>(&'a mut self) -> &'a mut [u8] {
        self.pending = true;

        &mut self.ring[0]
    }

    pub fn get_packet_to_transmit<'a>(&'a mut self) -> Option<&'a [u8]> {
        if self.pending {
            self.pending = false;

            Some(&self.ring[0])
        } else {
            None
        }
    }
}

pub struct Runner<'a> {
    mac_subsystem: Mac,
    pub(crate) rx_channel: Channel<CriticalSectionRawMutex, Event, 1>,
    pub(crate) tx_channel: Channel<CriticalSectionRawMutex, &'a [u8], 1>,
}

impl<'a> Runner<'a> {
    pub fn new(mac: Mac) -> Self {
        Self {
            mac_subsystem: mac,
            rx_channel: Channel::new(),
            tx_channel: Channel::new(),
        }
    }

    pub(crate) async fn init(&mut self, firmware: &[u8]) {
        debug!("wifi init done");
    }

    pub async fn run(&self) -> ! {
        loop {
            let event = self.mac_subsystem.read().await;
            if let Ok(evt) = event.mac_event() {
                match evt {
                    MacEvent::McpsDataInd(data_ind) => {
                        self.rx_channel.try_send(event);
                    }
                    _ => {}
                }
            }

            // TODO: select tx event
        }
    }
}
