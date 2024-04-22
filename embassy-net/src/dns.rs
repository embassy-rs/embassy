//! DNS client compatible with the `embedded-nal-async` traits.
//!
//! This exists only for compatibility with crates that use `embedded-nal-async`.
//! Prefer using [`Stack::dns_query`](crate::Stack::dns_query) directly if you're
//! not using `embedded-nal-async`.

use heapless::Vec;
pub use smoltcp::socket::dns::{DnsQuery, Socket};
pub(crate) use smoltcp::socket::dns::{GetQueryResultError, StartQueryError};
pub use smoltcp::wire::{DnsQueryType, IpAddress};

use crate::{Driver, Stack};

/// Errors returned by DnsSocket.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Invalid name
    InvalidName,
    /// Name too long
    NameTooLong,
    /// Name lookup failed
    Failed,
}

impl From<GetQueryResultError> for Error {
    fn from(_: GetQueryResultError) -> Self {
        Self::Failed
    }
}

impl From<StartQueryError> for Error {
    fn from(e: StartQueryError) -> Self {
        match e {
            StartQueryError::NoFreeSlot => Self::Failed,
            StartQueryError::InvalidName => Self::InvalidName,
            StartQueryError::NameTooLong => Self::NameTooLong,
        }
    }
}

/// DNS client compatible with the `embedded-nal-async` traits.
///
/// This exists only for compatibility with crates that use `embedded-nal-async`.
/// Prefer using [`Stack::dns_query`](crate::Stack::dns_query) directly if you're
/// not using `embedded-nal-async`.
pub struct DnsSocket<'a, D>
where
    D: Driver + 'static,
{
    stack: &'a Stack<D>,
}

impl<'a, D> DnsSocket<'a, D>
where
    D: Driver + 'static,
{
    /// Create a new DNS socket using the provided stack.
    ///
    /// NOTE: If using DHCP, make sure it has reconfigured the stack to ensure the DNS servers are updated.
    pub fn new(stack: &'a Stack<D>) -> Self {
        Self { stack }
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    pub async fn query(
        &self,
        name: &str,
        qtype: DnsQueryType,
    ) -> Result<Vec<IpAddress, { smoltcp::config::DNS_MAX_RESULT_COUNT }>, Error> {
        self.stack.dns_query(name, qtype).await
    }
}

impl<'a, D> embedded_nal_async::Dns for DnsSocket<'a, D>
where
    D: Driver + 'static,
{
    type Error = Error;

    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<embedded_nal_async::IpAddr, Self::Error> {
        use embedded_nal_async::{AddrType, IpAddr};
        let qtype = match addr_type {
            AddrType::IPv6 => DnsQueryType::Aaaa,
            _ => DnsQueryType::A,
        };
        let addrs = self.query(host, qtype).await?;
        if let Some(first) = addrs.get(0) {
            Ok(match first {
                #[cfg(feature = "proto-ipv4")]
                IpAddress::Ipv4(addr) => IpAddr::V4(addr.0.into()),
                #[cfg(feature = "proto-ipv6")]
                IpAddress::Ipv6(addr) => IpAddr::V6(addr.0.into()),
            })
        } else {
            Err(Error::Failed)
        }
    }

    async fn get_host_by_address(
        &self,
        _addr: embedded_nal_async::IpAddr,
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        todo!()
    }
}
