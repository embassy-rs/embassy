#![deny(unused_must_use)]

use core::cell::RefCell;
use core::task::Context;

use embassy_net_driver::{Capabilities, HardwareAddress, LinkState};
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::waitqueue::AtomicWaker;

use crate::mac::event::MacEvent;
use crate::mac::indications::{write_frame_from_beacon_indication, write_frame_from_data_indication};
use crate::mac::runner::{BUF_SIZE, ZeroCopyPubSub};
use crate::mac::{Control, MTU, Runner};
use crate::sub::mac::{Mac, MacRx, MacTx};

pub struct NetworkState {
    pub mac_addr: [u8; 8],
    pub short_addr: [u8; 2],
    pub pan_id: [u8; 2],
    pub link_state: LinkState,
    pub link_waker: AtomicWaker,
}

impl NetworkState {
    pub const fn new() -> Self {
        Self {
            mac_addr: [0u8; 8],
            short_addr: [0u8; 2],
            pan_id: [0u8; 2],
            link_state: LinkState::Down,
            link_waker: AtomicWaker::new(),
        }
    }
}

pub struct DriverState<'d> {
    pub mac_tx: Mutex<CriticalSectionRawMutex, MacTx>,
    pub mac_rx: MacRx,
    pub rx_event_channel: ZeroCopyPubSub<CriticalSectionRawMutex, MacEvent<'d>>,
    pub rx_data_channel: Channel<CriticalSectionRawMutex, MacEvent<'d>, 1>,
    pub tx_data_channel: Channel<CriticalSectionRawMutex, (&'d mut [u8; MTU], usize), BUF_SIZE>,
    pub tx_buf_channel: Channel<CriticalSectionRawMutex, &'d mut [u8; MTU], BUF_SIZE>,
    pub tx_buf_queue: [[u8; MTU]; BUF_SIZE],
    pub network_state: blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
}

impl<'d> DriverState<'d> {
    pub const fn new(mac: Mac) -> Self {
        let (mac_rx, mac_tx) = mac.split();
        let mac_tx = Mutex::new(mac_tx);

        Self {
            mac_tx,
            mac_rx,
            rx_event_channel: ZeroCopyPubSub::new(RefCell::new(None)),
            rx_data_channel: Channel::new(),
            tx_data_channel: Channel::new(),
            tx_buf_channel: Channel::new(),
            tx_buf_queue: [[0u8; MTU]; BUF_SIZE],
            network_state: blocking_mutex::Mutex::new(RefCell::new(NetworkState::new())),
        }
    }
}

pub struct Driver<'d> {
    tx_data_channel: &'d Channel<CriticalSectionRawMutex, (&'d mut [u8; MTU], usize), BUF_SIZE>,
    tx_buf_channel: &'d Channel<CriticalSectionRawMutex, &'d mut [u8; MTU], BUF_SIZE>,
    rx_data_channel: &'d Channel<CriticalSectionRawMutex, MacEvent<'d>, 1>,
    network_state: &'d blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<NetworkState>>,
}

impl<'d> Driver<'d> {
    pub fn new(
        driver_state: &'d mut DriverState<'d>,
        short_address: [u8; 2],
        mac_address: [u8; 8],
    ) -> (Self, Runner<'d>, Control<'d>) {
        (
            Self {
                tx_data_channel: &driver_state.tx_data_channel,
                tx_buf_channel: &driver_state.tx_buf_channel,
                rx_data_channel: &driver_state.rx_data_channel,
                network_state: &driver_state.network_state,
            },
            Runner::new(
                &driver_state.rx_event_channel,
                &driver_state.rx_data_channel,
                &mut driver_state.mac_rx,
                &driver_state.tx_data_channel,
                &driver_state.tx_buf_channel,
                &driver_state.mac_tx,
                &mut driver_state.tx_buf_queue,
                &driver_state.network_state,
                short_address,
                mac_address,
            ),
            Control::new(
                &driver_state.rx_event_channel,
                &driver_state.mac_tx,
                &driver_state.network_state,
            ),
        )
    }
}

impl<'d> embassy_net_driver::Driver for Driver<'d> {
    // type RxToken<'a> = RxToken<'a, 'd> where Self: 'a;
    // type TxToken<'a> = TxToken<'a, 'd> where Self: 'a;
    type RxToken<'a>
        = RxToken<'d>
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken<'d>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.rx_data_channel.poll_ready_to_receive(cx).is_ready()
            && self.tx_buf_channel.poll_ready_to_receive(cx).is_ready()
        {
            Some((
                RxToken {
                    rx: self.rx_data_channel,
                },
                TxToken {
                    tx: self.tx_data_channel,
                    tx_buf: self.tx_buf_channel,
                },
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        if self.tx_buf_channel.poll_ready_to_receive(cx).is_ready() {
            Some(TxToken {
                tx: self.tx_data_channel,
                tx_buf: self.tx_buf_channel,
            })
        } else {
            None
        }
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        // caps.max_burst_size = Some(self.tx.len());
        caps
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        critical_section::with(|cs| {
            let network_state = self.network_state.borrow(cs).borrow_mut();

            // Unconditionally register the waker to avoid a race
            network_state.link_waker.register(cx.waker());
            network_state.link_state
        })
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ieee802154(critical_section::with(|cs| {
            self.network_state.borrow(cs).borrow().mac_addr
        }))
    }
}

pub struct RxToken<'d> {
    rx: &'d Channel<CriticalSectionRawMutex, MacEvent<'d>, 1>,
}

impl<'d> embassy_net_driver::RxToken for RxToken<'d> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = [0u8; MTU];
        match self.rx.try_receive().unwrap() {
            MacEvent::McpsDataInd(data_event) => write_frame_from_data_indication(data_event, &mut buffer),
            MacEvent::MlmeBeaconNotifyInd(data_event) => write_frame_from_beacon_indication(data_event, &mut buffer),
            _ => {}
        };

        f(&mut buffer[..])
    }
}

pub struct TxToken<'d> {
    tx: &'d Channel<CriticalSectionRawMutex, (&'d mut [u8; MTU], usize), BUF_SIZE>,
    tx_buf: &'d Channel<CriticalSectionRawMutex, &'d mut [u8; MTU], BUF_SIZE>,
}

impl<'d> embassy_net_driver::TxToken for TxToken<'d> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // Only valid tx buffers should be put into the queue
        let buf = self.tx_buf.try_receive().unwrap();
        let r = f(&mut buf[..len]);

        // The tx channel should always be of equal capacity to the tx_buf channel
        self.tx.try_send((buf, len)).unwrap();

        r
    }
}
