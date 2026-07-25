use core::marker::PhantomData;
use core::task::Context;

#[cfg(feature = "ptp")]
use embassy_net_driver::Timestamp;
use embassy_net_driver::{Capabilities, Checksum, Driver, PacketMeta, RxToken, TxToken};
#[cfg(feature = "ptp")]
use smoltcp::iface::SocketHandle;
use smoltcp::phy::{self, Medium};
use smoltcp::time::Instant;

#[cfg(feature = "ptp")]
use crate::TimestampSink;
#[cfg(feature = "ptp")]
use crate::map_util::DynLinearMap;

pub(crate) struct DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    // must be Some when actually using this to rx/tx
    pub cx: Option<&'d mut Context<'c>>,
    pub inner: &'d mut T,
    pub medium: Medium,
    pub tx_exhausted: bool,
    #[cfg(feature = "ptp")]
    pub sinks: &'d mut dyn DynLinearMap<SocketHandle, TimestampSink>,
    #[cfg(feature = "ptp")]
    pub source: &'d mut [(u32, Timestamp)],
    #[cfg(feature = "ptp")]
    pub source_index: &'d mut u32,
    #[cfg(feature = "ptp")]
    pub next_id: &'d mut u32,
    #[cfg(feature = "ptp")]
    pub last_rx_id: &'d mut u32,
}

impl<'d, 'c, T> phy::Device for DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    type RxToken<'a>
        = RxTokenAdapter<T::RxToken<'a>>
    where
        Self: 'a;
    type TxToken<'a>
        = TxTokenAdapter<'a, T::TxToken<'a>>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner.receive(unwrap!(self.cx.as_deref_mut())).map(|(rx, tx)| {
            // TODO: if the id does not match the last rx id for the receive tokens, store the timestamp in the timestamp sink

            (
                RxTokenAdapter(rx),
                TxTokenAdapter {
                    token: tx,
                    #[cfg(feature = "ptp")]
                    sinks: self.sinks,
                    _lifetime: PhantomData,
                },
            )
        })
    }

    /// Construct a transmit token.
    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // TODO: for the transmit tokens, refactor the token so that a call to set meta stores the associated id in the sink
        let token = self
            .inner
            .transmit(unwrap!(self.cx.as_deref_mut()))
            .map(|tx| TxTokenAdapter {
                token: tx,
                #[cfg(feature = "ptp")]
                sinks: self.sinks,
                _lifetime: PhantomData,
            });

        self.tx_exhausted = token.is_none();

        token
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
        smolcaps.medium = self.medium;
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
        F: FnOnce(&[u8]) -> R,
    {
        self.0.consume(|buf| {
            #[cfg(feature = "packet-trace")]
            trace!("embassy device rx: {:02x}", buf);
            f(buf)
        })
    }

    fn meta(&self) -> phy::PacketMeta {
        into_smoltcp_meta(self.0.meta())
    }
}

pub(crate) struct TxTokenAdapter<'d, T>
where
    T: TxToken,
{
    token: T,
    #[cfg(feature = "ptp")]
    sinks: &'d mut dyn DynLinearMap<SocketHandle, TimestampSink>,
    _lifetime: PhantomData<&'d ()>,
}

impl<'d, T> phy::TxToken for TxTokenAdapter<'d, T>
where
    T: TxToken,
{
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.token.consume(len, |buf| {
            let r = f(buf);
            #[cfg(feature = "packet-trace")]
            trace!("embassy device tx: {:02x}", buf);
            r
        })
    }

    fn set_meta(&mut self, meta: phy::PacketMeta) {
        #[cfg(feature = "ptp")]
        // store the packet ID into the sink
        for (_socket, sink) in self.sinks.iter_mut() {
            if !sink.tx.get(&meta.id).is_some() {
                continue;
            }

            if sink.tx_assoc.insert(self.token.id() as u32, meta.id).is_err() {
                warn!("failed to insert timestamp into map during set_meta");
            }
        }

        self.token.set_meta(into_embassy_net_meta(meta));
    }
}

#[allow(unused, reason = "meta isn't used if no features are enabled")]
pub(crate) fn into_smoltcp_meta(meta: PacketMeta) -> phy::PacketMeta {
    let mut out_meta = phy::PacketMeta::default();
    #[cfg(feature = "packetmeta-id")]
    {
        out_meta.id = meta.id;
    }
    out_meta
}

#[allow(unused, reason = "meta isn't used if no features are enabled")]
pub(crate) fn into_embassy_net_meta(meta: phy::PacketMeta) -> PacketMeta {
    let mut out_meta = PacketMeta::default();
    #[cfg(feature = "packetmeta-id")]
    {
        out_meta.id = meta.id;
    }
    out_meta
}
