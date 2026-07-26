//! The MLME-SAP allows the transport of management commands between the next higher layer and the MLME.
//! The MCPS-SAP supports the transport of data.

pub mod commands;
pub mod control;
pub mod iface;
pub mod indications;
mod macros;
pub mod responses;
pub mod runner;
pub mod typedefs;
mod util;

use core::marker::PhantomData;
use core::mem;

pub use ch::Device;
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;

use crate::net::control::Control;
use crate::net::iface::{Controller, ControllerToHostPacketBox, FromHciBytes, FromHciBytesError};
use crate::net::runner::Runner;
use crate::net::util::ZeroCopyPubSub;

pub const MTU: usize = 127;

pub const MAX_PAN_DESC_SUPPORTED: usize = 6;
pub const MAX_SOUNDING_LIST_SUPPORTED: usize = 6;
pub const MAX_PENDING_ADDRESS: usize = 7;
pub const MAX_ED_SCAN_RESULTS_SUPPORTED: usize = 16;

impl<'de, T: MacEvent> FromHciBytes<'de> for T {
    fn from_hci_bytes(data: &'de [u8]) -> Result<&'de Self, iface::FromHciBytesError> {
        if data.len() != mem::size_of::<Self>() {
            warn!("invalid size when parsing: {}", T::NAME);

            Err(FromHciBytesError::InvalidSize)
        } else {
            Ok(unsafe { &*(data as *const _ as *const Self) })
        }
    }
}

trait MacEvent: Sized {
    const NAME: &str;
}

/// Driver state.
pub struct State<'a, C: Controller> {
    net: NetState<'a, C::Packet>,
}

struct NetState<'a, B: ControllerToHostPacketBox> {
    ch: ch::State<MTU, 4, 4>,
    events: ZeroCopyPubSub<B>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, C: Controller> State<'a, C> {
    /// Create new driver state holder.
    pub const fn new() -> Self {
        Self {
            net: NetState {
                ch: ch::State::new(),
                events: ZeroCopyPubSub::new(),

                _lifetime: PhantomData,
            },
        }
    }
}

/// Embassy-net driver.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

pub fn new<'a, C>(
    state: &'a mut State<'a, C>,
    controller: &'a C,
    hw_addr: [u8; 8],
) -> (NetDriver<'a>, Control<'a, C>, Runner<'a, C>)
where
    C: Controller,
{
    let (ch_runner, device) = ch::new(&mut state.net.ch, ch::driver::HardwareAddress::Ieee802154(hw_addr));
    let state_ch = ch_runner.state_runner();

    state_ch.set_link_state(LinkState::Down);

    let runner = Runner::new(controller, ch_runner, &state.net.events);

    let control = Control::new(controller, state_ch, &state.net.events);

    (device, control, runner)
}
