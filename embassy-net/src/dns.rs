//! DNS client compatible with the `embedded-nal-async` traits.
//!
//! This exists only for compatibility with crates that use `embedded-nal-async`.
//! Prefer using [`Stack::dns_query`](crate::Stack::dns_query) directly if you're
//! not using `embedded-nal-async`.

use heapless::Vec;
pub use smoltcp::socket::dns::{DnsQuery, Socket};
pub(crate) use smoltcp::socket::dns::{GetQueryResultError, StartQueryError};
pub use smoltcp::wire::{DnsQueryType, IpAddress};

use crate::Stack;

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
pub struct DnsSocket<'a> {
    stack: Stack<'a>,
}

impl<'a> DnsSocket<'a> {
    /// Create a new DNS socket using the provided stack.
    ///
    /// NOTE: If using DHCP, make sure it has reconfigured the stack to ensure the DNS servers are updated.
    pub fn new(stack: Stack<'a>) -> Self {
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

impl<'a> embedded_nal_async::Dns for DnsSocket<'a> {
    type Error = Error;

    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<core::net::IpAddr, Self::Error> {
        use core::net::IpAddr;

        use embedded_nal_async::AddrType;

        let (qtype, secondary_qtype) = match addr_type {
            AddrType::IPv4 => (DnsQueryType::A, None),
            AddrType::IPv6 => (DnsQueryType::Aaaa, None),
            AddrType::Either => {
                #[cfg(not(feature = "proto-ipv6"))]
                let v6_first = false;
                #[cfg(feature = "proto-ipv6")]
                let v6_first = self.stack.config_v6().is_some();
                match v6_first {
                    true => (DnsQueryType::Aaaa, Some(DnsQueryType::A)),
                    false => (DnsQueryType::A, Some(DnsQueryType::Aaaa)),
                }
            }
        };
        let mut addrs = self.query(host, qtype).await?;
        if addrs.is_empty() {
            if let Some(qtype) = secondary_qtype {
                addrs = self.query(host, qtype).await?
            }
        }
        if let Some(first) = addrs.get(0) {
            Ok(match first {
                #[cfg(feature = "proto-ipv4")]
                IpAddress::Ipv4(addr) => IpAddr::V4(*addr),
                #[cfg(feature = "proto-ipv6")]
                IpAddress::Ipv6(addr) => IpAddr::V6(*addr),
            })
        } else {
            Err(Error::Failed)
        }
    }

    async fn get_host_by_address(&self, _addr: core::net::IpAddr, _result: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

fn _assert_covariant<'a, 'b: 'a>(x: DnsSocket<'b>) -> DnsSocket<'a> {
    x
}
