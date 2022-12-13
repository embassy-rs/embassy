use core::task::Context;

use smoltcp::phy;
pub use smoltcp::phy::{Checksum, ChecksumCapabilities, DeviceCapabilities, Medium};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LinkState {
    Down,
    Up,
}

pub trait Device {
    type RxToken<'a>: RxToken
    where
        Self: 'a;
    type TxToken<'a>: TxToken
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)>;
    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>>;
    fn link_state(&mut self, cx: &mut Context) -> LinkState;

    fn capabilities(&self) -> phy::DeviceCapabilities;
    fn ethernet_address(&self) -> [u8; 6];
}

impl<T: ?Sized + Device> Device for &mut T {
    type RxToken<'a> = T::RxToken<'a>
    where
        Self: 'a;
    type TxToken<'a> = T::TxToken<'a>
    where
        Self: 'a;

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        T::transmit(self, cx)
    }
    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        T::receive(self, cx)
    }
    fn capabilities(&self) -> phy::DeviceCapabilities {
        T::capabilities(self)
    }
    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        T::link_state(self, cx)
    }
    fn ethernet_address(&self) -> [u8; 6] {
        T::ethernet_address(self)
    }
}

/// A token to receive a single network packet.
pub trait RxToken {
    /// Consumes the token to receive a single network packet.
    ///
    /// This method receives a packet and then calls the given closure `f` with the raw
    /// packet bytes as argument.
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

/// A token to transmit a single network packet.
pub trait TxToken {
    /// Consumes the token to send a single network packet.
    ///
    /// This method constructs a transmit buffer of size `len` and calls the passed
    /// closure `f` with a mutable reference to that buffer. The closure should construct
    /// a valid network packet (e.g. an ethernet packet) in the buffer. When the closure
    /// returns, the transmit buffer is sent out.
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

///////////////////////////

pub(crate) struct DeviceAdapter<'d, 'c, T>
where
    T: Device,
{
    // must be Some when actually using this to rx/tx
    pub cx: Option<&'d mut Context<'c>>,
    pub inner: &'d mut T,
}

impl<'d, 'c, T> phy::Device for DeviceAdapter<'d, 'c, T>
where
    T: Device,
{
    type RxToken<'a> = RxTokenAdapter<T::RxToken<'a>> where Self: 'a;
    type TxToken<'a> = TxTokenAdapter<T::TxToken<'a>> where Self: 'a;

    fn receive(&mut self) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner
            .receive(self.cx.as_deref_mut().unwrap())
            .map(|(rx, tx)| (RxTokenAdapter(rx), TxTokenAdapter(tx)))
    }

    /// Construct a transmit token.
    fn transmit(&mut self) -> Option<Self::TxToken<'_>> {
        self.inner.transmit(self.cx.as_deref_mut().unwrap()).map(TxTokenAdapter)
    }

    /// Get a description of device capabilities.
    fn capabilities(&self) -> phy::DeviceCapabilities {
        self.inner.capabilities()
    }
}

pub(crate) struct RxTokenAdapter<T>(T)
where
    T: RxToken;

impl<T> phy::RxToken for RxTokenAdapter<T>
where
    T: RxToken,
{
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        self.0.consume(|buf| f(buf))
    }
}

pub(crate) struct TxTokenAdapter<T>(T)
where
    T: TxToken;

impl<T> phy::TxToken for TxTokenAdapter<T>
where
    T: TxToken,
{
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        self.0.consume(len, |buf| f(buf))
    }
}
