use core::task::Context;

use embassy_net_driver::{Capabilities, Checksum, Driver, Medium, RxToken, TxToken};
use smoltcp::phy;
use smoltcp::time::Instant;

pub(crate) struct DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    // must be Some when actually using this to rx/tx
    pub cx: Option<&'d mut Context<'c>>,
    pub inner: &'d mut T,
}

impl<'d, 'c, T> phy::Device for DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    type RxToken<'a> = RxTokenAdapter<T::RxToken<'a>> where Self: 'a;
    type TxToken<'a> = TxTokenAdapter<T::TxToken<'a>> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner
            .receive(unwrap!(self.cx.as_deref_mut()))
            .map(|(rx, tx)| (RxTokenAdapter(rx), TxTokenAdapter(tx)))
    }

    /// Construct a transmit token.
    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.inner.transmit(unwrap!(self.cx.as_deref_mut())).map(TxTokenAdapter)
    }

    /// Get a description of device capabilities.
    fn capabilities(&self) -> phy::DeviceCapabilities {
        fn convert(c: Checksum) -> phy::Checksum {
            match c {
                Checksum::Both => phy::Checksum::Both,
                Checksum::Tx => phy::Checksum::Tx,
                Checksum::Rx => phy::Checksum::Rx,
                Checksum::None => phy::Checksum::None,
            }
        }
        let caps: Capabilities = self.inner.capabilities();
        let mut smolcaps = phy::DeviceCapabilities::default();

        smolcaps.max_transmission_unit = caps.max_transmission_unit;
        smolcaps.max_burst_size = caps.max_burst_size;
        smolcaps.medium = match caps.medium {
            #[cfg(feature = "medium-ethernet")]
            Medium::Ethernet => phy::Medium::Ethernet,
            #[cfg(feature = "medium-ip")]
            Medium::Ip => phy::Medium::Ip,
            #[cfg(feature = "medium-ieee802154")]
            Medium::Ieee802154 => phy::Medium::Ieee802154,
            #[allow(unreachable_patterns)]
            _ => panic!(
                "Unsupported medium {:?}. Make sure to enable it in embassy-net's Cargo features.",
                caps.medium
            ),
        };
        smolcaps.checksum.ipv4 = convert(caps.checksum.ipv4);
        smolcaps.checksum.tcp = convert(caps.checksum.tcp);
        smolcaps.checksum.udp = convert(caps.checksum.udp);
        #[cfg(feature = "proto-ipv4")]
        {
            smolcaps.checksum.icmpv4 = convert(caps.checksum.icmpv4);
        }
        #[cfg(feature = "proto-ipv6")]
        {
            smolcaps.checksum.icmpv6 = convert(caps.checksum.icmpv6);
        }

        smolcaps
    }
}

pub(crate) struct RxTokenAdapter<T>(T)
where
    T: RxToken;

impl<T> phy::RxToken for RxTokenAdapter<T>
where
    T: RxToken,
{
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
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
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.0.consume(len, |buf| f(buf))
    }
}
