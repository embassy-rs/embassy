//! DNS socket with async support.
use core::cell::RefCell;
use core::future::poll_fn;
use core::mem;
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_net_driver::Driver;
use heapless::Vec;
use managed::ManagedSlice;
use smoltcp::iface::{Interface, SocketHandle};
pub use smoltcp::socket::dns::DnsQuery;
use smoltcp::socket::dns::{self, GetQueryResultError, StartQueryError, MAX_ADDRESS_COUNT};
pub use smoltcp::wire::{DnsQueryType, IpAddress};

use crate::{SocketStack, Stack};

/// Errors returned by DnsSocket.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// No available query slot
    NoFreeSlot,
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
            StartQueryError::NoFreeSlot => Self::NoFreeSlot,
            StartQueryError::InvalidName => Self::InvalidName,
            StartQueryError::NameTooLong => Self::NameTooLong,
        }
    }
}

/// Async socket for making DNS queries.
pub struct DnsSocket<'a> {
    stack: &'a RefCell<SocketStack>,
    handle: SocketHandle,
}

impl<'a> DnsSocket<'a> {
    /// Create a new DNS socket using the provided stack and query storage.
    ///
    /// DNS servers are derived from the stack configuration.
    ///
    /// NOTE: If using DHCP, make sure it has reconfigured the stack to ensure the DNS servers are updated.
    pub fn new<D, Q>(stack: &'a Stack<D>, queries: Q) -> Self
    where
        D: Driver + 'static,
        Q: Into<ManagedSlice<'a, Option<DnsQuery>>>,
    {
        let servers = stack
            .config()
            .map(|c| {
                let v: Vec<IpAddress, 3> = c.dns_servers.iter().map(|c| IpAddress::Ipv4(*c)).collect();
                v
            })
            .unwrap_or(Vec::new());
        let s = &mut *stack.socket.borrow_mut();
        let queries: ManagedSlice<'static, Option<DnsQuery>> = unsafe { mem::transmute(queries.into()) };

        let handle = s.sockets.add(dns::Socket::new(&servers[..], queries));
        Self {
            stack: &stack.socket,
            handle,
        }
    }

    fn with_mut<R>(&mut self, f: impl FnOnce(&mut dns::Socket, &mut Interface) -> R) -> R {
        let s = &mut *self.stack.borrow_mut();
        let socket = s.sockets.get_mut::<dns::Socket>(self.handle);
        let res = f(socket, &mut s.iface);
        s.waker.wake();
        res
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    pub async fn query(&mut self, name: &str, qtype: DnsQueryType) -> Result<Vec<IpAddress, MAX_ADDRESS_COUNT>, Error> {
        let query = match { self.with_mut(|s, i| s.start_query(i.context(), name, qtype)) } {
            Ok(handle) => handle,
            Err(e) => return Err(e.into()),
        };

        let handle = self.handle;
        let drop = OnDrop::new(|| {
            let s = &mut *self.stack.borrow_mut();
            let socket = s.sockets.get_mut::<dns::Socket>(handle);
            socket.cancel_query(query);
            s.waker.wake();
        });

        let res = poll_fn(|cx| {
            self.with_mut(|s, _| match s.get_query_result(query) {
                Ok(addrs) => Poll::Ready(Ok(addrs)),
                Err(GetQueryResultError::Pending) => {
                    s.register_query_waker(query, cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(e.into())),
            })
        })
        .await;

        drop.defuse();
        res
    }
}

impl<'a> Drop for DnsSocket<'a> {
    fn drop(&mut self) {
        self.stack.borrow_mut().sockets.remove(self.handle);
    }
}
