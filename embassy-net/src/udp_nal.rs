//! UDP sockets usable through [embedded_nal_async]
//!
//! The full [embedded_nal_async::UdpStack] is *not* implemented at the moment: As its API allows
//! arbitrary creation of movable sockets, embassy's [udp::UdpSocket] type could only be crated if
//! the NAL stack had a pre-allocated pool of sockets with their respective buffers. Nothing rules
//! out such a type, but at the moment, only the bound or connected socket types are implemented
//! with their own constructors from an embassy [crate::Stack] -- for many applications, those are
//! useful enough. (FIXME: Given we construct from Socket, Stack could really be implemented on
//! `Cell<Option<Socket>>` by `.take()`ing, couldn't it?)
//!
//! The constructors of the various socket types mimick the UdpStack's socket creation functions,
//! but take an owned (uninitialized) Socket instead of a shared stack.
//!
//! No `bind_single` style constructor is currently provided. FIXME: Not sure we have all the
//! information at bind time to specialize a wildcard address into a concrete address and return
//! it. Should the NAL trait be updated to disallow using wildcard addresses on `bind_single`, and
//! merely allow unspecified ports to get an ephemeral one?

use core::future::poll_fn;

use embedded_nal_async as nal;
use smoltcp::wire::{IpAddress, IpEndpoint};

use crate::udp;

pub struct ConnectedUdp<'a> {
    remote: IpEndpoint,
    // The local port is stored in the socket, as it gets bound. This value is populated lazily:
    // embassy only decides at udp::Socket::dispatch time whence to send, and while we could
    // duplicate the code for the None case of the local_address by calling the right
    // get_source_address function, we'd still need an interface::Context / an interface to call
    // this through, and AFAICT we don't get access to that.
    local: Option<IpAddress>,
    socket: udp::UdpSocket<'a>,
}

pub struct UnconnectedUdp<'a> {
    socket: udp::UdpSocket<'a>,
}

fn sockaddr_nal2smol(sockaddr: nal::SocketAddr) -> IpEndpoint {
    todo!()
}

fn sockaddr_smol2nal(sockaddr: IpEndpoint) -> nal::SocketAddr {
    todo!()
}

fn is_unspec(addr: nal::SocketAddr) -> bool {
    // FIXME
    false
}

// FIXME: Expose details
#[derive(Debug)]
pub struct Error;

impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        todo!()
    }
}
impl From<udp::RecvError> for Error {
    fn from(_: udp::RecvError) -> Self {
        Self
    }
}
impl From<udp::SendError> for Error {
    fn from(_: udp::SendError) -> Self {
        Self
    }
}

impl<'a> ConnectedUdp<'a> {
    /// Create a ConnectedUdp.
    ///
    /// ## Prerequisites
    ///
    /// The `socket` must be open (in the sense of smoltcp's `.is_open()`) -- unbound and
    /// unconnected.
    pub async fn connect_from(
        mut socket: udp::UdpSocket<'a>,
        local: nal::SocketAddr,
        remote: nal::SocketAddr,
    ) -> Result<Self, udp::BindError> {
        socket.bind(sockaddr_nal2smol(local))?;

        Ok(ConnectedUdp {
            remote: sockaddr_nal2smol(remote),
            // FIXME: We could check if local was fully (or sufficiently, picking the port from the
            // socket) specified and store if yes -- for a first iteration, leaving that to the
            // fallback path we need anyway in case local is [::].
            local: None,
            socket,
        })
    }

    pub async fn connect(socket: udp::UdpSocket<'a> /*, ... */) -> Result<Self, udp::BindError> {
        // This is really just a copy of the provided `embedded_nal::udp::UdpStack::connect` method
        todo!()
    }
}

impl<'a> UnconnectedUdp<'a> {
    /// Create an UnconnectedUdp.
    ///
    /// The `local` address may be anything from fully specified (address and port) to fully
    /// unspecified (port 0, all-zeros address).
    ///
    /// ## Prerequisites
    ///
    /// The `socket` must be open (in the sense of smoltcp's `.is_open()`) -- unbound and
    /// unconnected.
    pub async fn bind_multiple(mut socket: udp::UdpSocket<'a>, local: nal::SocketAddr) -> Result<Self, udp::BindError> {
        socket.bind(sockaddr_nal2smol(local))?;

        Ok(UnconnectedUdp { socket })
    }
}

impl<'a> nal::UnconnectedUdp for UnconnectedUdp<'a> {
    type Error = Error;
    async fn send(
        &mut self,
        local: embedded_nal_async::SocketAddr,
        remote: embedded_nal_async::SocketAddr,
        buf: &[u8],
    ) -> Result<(), Error> {
        // TODO: debug_assert!(local.port == 0 || local.port == self.socket.where-do-we-get-the-bound-port-here,
        // "Attempted to send from different port than the socket was bound to")

        let remote_endpoint = smoltcp::socket::udp::UdpMetadata {
            // A conversion of the addr part only might be cheaper
            local_address: if is_unspec(local) {
                None
            } else {
                Some(sockaddr_nal2smol(local).addr)
            },
            ..sockaddr_nal2smol(remote).into()
        };
        poll_fn(move |cx| self.socket.poll_send_to(buf, remote_endpoint, cx)).await?;
        Ok(())
    }
    async fn receive_into(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(usize, embedded_nal_async::SocketAddr, embedded_nal_async::SocketAddr), Error> {
        // FIXME: The truncation is an issue -- we may need to change poll_recv_from to poll_recv
        // and copy from the slice ourselves to get the trait's behavior
        let (size, metadata) = poll_fn(move |cx| self.socket.poll_recv_from(buf, cx)).await?;
        Ok((
            size,
            sockaddr_smol2nal(IpEndpoint {
                addr: metadata
                    .local_address
                    .expect("Local address is always populated on receive"),
                port: 0, // FIXME obtain or persist
            }),
            sockaddr_smol2nal(metadata.endpoint),
        ))
    }
}
