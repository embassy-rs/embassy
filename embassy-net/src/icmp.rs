//! ICMP sockets.

use core::future::{poll_fn, Future};
use core::mem;
use core::task::{Context, Poll};

use smoltcp::iface::{Interface, SocketHandle};
pub use smoltcp::phy::ChecksumCapabilities;
use smoltcp::socket::icmp;
pub use smoltcp::socket::icmp::{Endpoint as IcmpEndpoint, PacketMetadata};
use smoltcp::wire::IpAddress;
#[cfg(feature = "proto-ipv4")]
pub use smoltcp::wire::{Icmpv4Message, Icmpv4Packet, Icmpv4Repr};
#[cfg(feature = "proto-ipv6")]
pub use smoltcp::wire::{Icmpv6Message, Icmpv6Packet, Icmpv6Repr};

use crate::Stack;

/// Error returned by [`IcmpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket was already open.
    InvalidState,
    /// The endpoint isn't specified
    InvalidEndpoint,
    /// No route to host.
    NoRoute,
}

/// Error returned by [`IcmpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// No route to host.
    NoRoute,
    /// Socket not bound to an outgoing port.
    SocketNotBound,
    /// There is not enough transmit buffer capacity to ever send this packet.
    PacketTooLarge,
}

/// Error returned by [`IcmpSocket::recv_from`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// Provided buffer was smaller than the received packet.
    Truncated,
}

/// An ICMP socket.
pub struct IcmpSocket<'a> {
    stack: Stack<'a>,
    handle: SocketHandle,
}

impl<'a> IcmpSocket<'a> {
    /// Create a new ICMP socket using the provided stack and buffers.
    pub fn new(
        stack: Stack<'a>,
        rx_meta: &'a mut [PacketMetadata],
        rx_buffer: &'a mut [u8],
        tx_meta: &'a mut [PacketMetadata],
        tx_buffer: &'a mut [u8],
    ) -> Self {
        let handle = stack.with_mut(|i| {
            let rx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(rx_meta) };
            let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
            let tx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(tx_meta) };
            let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
            i.sockets.add(icmp::Socket::new(
                icmp::PacketBuffer::new(rx_meta, rx_buffer),
                icmp::PacketBuffer::new(tx_meta, tx_buffer),
            ))
        });

        Self { stack, handle }
    }

    /// Bind the socket to the given endpoint.
    pub fn bind<T>(&mut self, endpoint: T) -> Result<(), BindError>
    where
        T: Into<IcmpEndpoint>,
    {
        let endpoint = endpoint.into();

        if !endpoint.is_specified() {
            return Err(BindError::InvalidEndpoint);
        }

        match self.with_mut(|s, _| s.bind(endpoint)) {
            Ok(()) => Ok(()),
            Err(icmp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(icmp::BindError::Unaddressable) => Err(BindError::NoRoute),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&icmp::Socket, &Interface) -> R) -> R {
        self.stack.with(|i| {
            let socket = i.sockets.get::<icmp::Socket>(self.handle);
            f(socket, &i.iface)
        })
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut icmp::Socket, &mut Interface) -> R) -> R {
        self.stack.with_mut(|i| {
            let socket = i.sockets.get_mut::<icmp::Socket>(self.handle);
            let res = f(socket, &mut i.iface);
            i.waker.wake();
            res
        })
    }

    /// Wait until the socket becomes readable.
    ///
    /// A socket is readable when a packet has been received, or when there are queued packets in
    /// the buffer.
    pub fn wait_recv_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.poll_recv_ready(cx))
    }

    /// Wait until a datagram can be read.
    ///
    /// When no datagram is readable, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready`.
    pub fn poll_recv_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s, _| {
            if s.can_recv() {
                Poll::Ready(())
            } else {
                // socket buffer is empty wait until at least one byte has arrived
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a datagram.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// Returns the number of bytes received and the remote endpoint.
    pub fn recv_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, IpAddress), RecvError>> + 's {
        poll_fn(|cx| self.poll_recv_from(buf, cx))
    }

    /// Receive a datagram.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready` with the
    /// number of bytes received and the remote endpoint.
    pub fn poll_recv_from(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<(usize, IpAddress), RecvError>> {
        self.with_mut(|s, _| match s.recv_slice(buf) {
            Ok((n, meta)) => Poll::Ready(Ok((n, meta))),
            // No data ready
            Err(icmp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(icmp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Dequeue a packet received from a remote endpoint and calls the provided function with the
    /// slice of the packet and the remote endpoint address and returns `Poll::Ready` with the
    /// function's returned value.
    ///
    /// **Note**: when the size of the provided buffer is smaller than the size of the payload,
    /// the packet is dropped and a `RecvError::Truncated` error is returned.
    pub async fn recv_from_with<F, R>(&self, f: F) -> Result<R, RecvError>
    where
        F: FnOnce((&[u8], IpAddress)) -> R,
    {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s, _| match s.recv() {
                Ok(x) => Poll::Ready(Ok(unwrap!(f.take())(x))),
                Err(icmp::RecvError::Exhausted) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(icmp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            })
        })
        .await
    }

    /// Wait until the socket becomes writable.
    ///
    /// A socket becomes writable when there is space in the buffer, from initial memory or after
    /// dispatching datagrams on a full buffer.
    pub fn wait_send_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| self.poll_send_ready(cx))
    }

    /// Wait until a datagram can be sent.
    ///
    /// When no datagram can be sent (i.e. the buffer is full), this method will return
    /// `Poll::Pending` and register the current task to be notified when
    /// space is freed in the buffer after a datagram has been dispatched.
    ///
    /// When a datagram can be sent, this method will return `Poll::Ready`.
    pub fn poll_send_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s, _| {
            if s.can_send() {
                Poll::Ready(())
            } else {
                // socket buffer is full wait until a datagram has been dispatched
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// If the socket's send buffer is too small to fit `buf`, this method will return `Err(SendError::PacketTooLarge)`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::NoRoute)`
    pub async fn send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), SendError>
    where
        T: Into<IpAddress>,
    {
        let remote_endpoint: IpAddress = remote_endpoint.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote_endpoint, cx)).await
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the socket's send buffer is full, this method will return `Poll::Pending`
    /// and register the current task to be notified when the buffer has space available.
    ///
    /// If the socket's send buffer is too small to fit `buf`, this method will return `Poll::Ready(Err(SendError::PacketTooLarge))`
    ///
    /// When the remote endpoint is not reachable, this method will return `Poll::Ready(Err(Error::NoRoute))`.
    pub fn poll_send_to<T>(&self, buf: &[u8], remote_endpoint: T, cx: &mut Context<'_>) -> Poll<Result<(), SendError>>
    where
        T: Into<IpAddress>,
    {
        // Don't need to wake waker in `with_mut` if the buffer will never fit the icmp tx_buffer.
        let send_capacity_too_small = self.with(|s, _| s.payload_send_capacity() < buf.len());
        if send_capacity_too_small {
            return Poll::Ready(Err(SendError::PacketTooLarge));
        }

        self.with_mut(|s, _| match s.send_slice(buf, remote_endpoint.into()) {
            // Entire datagram has been sent
            Ok(()) => Poll::Ready(Ok(())),
            Err(icmp::SendError::BufferFull) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
            Err(icmp::SendError::Unaddressable) => {
                // If no sender/outgoing port is specified, there is not really "no route"
                if s.is_open() {
                    Poll::Ready(Err(SendError::NoRoute))
                } else {
                    Poll::Ready(Err(SendError::SocketNotBound))
                }
            }
        })
    }

    /// Enqueue a packet to be sent to a given remote address with a zero-copy function.
    ///
    /// This method will wait until the buffer can fit the requested size before
    /// calling the function to fill its contents.
    pub async fn send_to_with<T, F, R>(&mut self, size: usize, remote_endpoint: T, f: F) -> Result<R, SendError>
    where
        T: Into<IpAddress>,
        F: FnOnce(&mut [u8]) -> R,
    {
        // Don't need to wake waker in `with_mut` if the buffer will never fit the icmp tx_buffer.
        let send_capacity_too_small = self.with(|s, _| s.payload_send_capacity() < size);
        if send_capacity_too_small {
            return Err(SendError::PacketTooLarge);
        }

        let mut f = Some(f);
        let remote_endpoint = remote_endpoint.into();
        poll_fn(move |cx| {
            self.with_mut(|s, _| match s.send(size, remote_endpoint) {
                Ok(buf) => Poll::Ready(Ok({ unwrap!(f.take())(buf) })),
                Err(icmp::SendError::BufferFull) => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                Err(icmp::SendError::Unaddressable) => Poll::Ready(Err(SendError::NoRoute)),
            })
        })
        .await
    }

    /// Flush the socket.
    ///
    /// This method will wait until the socket is flushed.
    pub fn flush(&mut self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            self.with_mut(|s, _| {
                if s.send_queue() == 0 {
                    Poll::Ready(())
                } else {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
    }

    /// Check whether the socket is open.
    pub fn is_open(&self) -> bool {
        self.with(|s, _| s.is_open())
    }

    /// Returns whether the socket is ready to send data, i.e. it has enough buffer space to hold a packet.
    pub fn may_send(&self) -> bool {
        self.with(|s, _| s.can_send())
    }

    /// Returns whether the socket is ready to receive data, i.e. it has received a packet that's now in the buffer.
    pub fn may_recv(&self) -> bool {
        self.with(|s, _| s.can_recv())
    }

    /// Return the maximum number packets the socket can receive.
    pub fn packet_recv_capacity(&self) -> usize {
        self.with(|s, _| s.packet_recv_capacity())
    }

    /// Return the maximum number packets the socket can receive.
    pub fn packet_send_capacity(&self) -> usize {
        self.with(|s, _| s.packet_send_capacity())
    }

    /// Return the maximum number of bytes inside the recv buffer.
    pub fn payload_recv_capacity(&self) -> usize {
        self.with(|s, _| s.payload_recv_capacity())
    }

    /// Return the maximum number of bytes inside the transmit buffer.
    pub fn payload_send_capacity(&self) -> usize {
        self.with(|s, _| s.payload_send_capacity())
    }

    /// Return the time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    pub fn hop_limit(&self) -> Option<u8> {
        self.with(|s, _| s.hop_limit())
    }

    /// Set the hop limit field in the IP header of sent packets.
    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        self.with_mut(|s, _| s.set_hop_limit(hop_limit))
    }
}

impl Drop for IcmpSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.sockets.remove(self.handle));
    }
}

pub mod ping {
    //! Ping utilities.
    //!
    //! This module allows for an easy ICMP Echo message interface used to
    //! ping devices with an [ICMP Socket](IcmpSocket).
    //!
    //! ## Usage
    //!
    //! ```
    //! use core::net::Ipv4Addr;
    //! use core::str::FromStr;
    //!
    //! use embassy_net::icmp::ping::{PingManager, PingParams};
    //! use embassy_net::icmp::PacketMetadata;
    //!
    //! let mut rx_buffer = [0; 256];
    //! let mut tx_buffer = [0; 256];
    //! let mut rx_meta = [PacketMetadata::EMPTY];
    //! let mut tx_meta = [PacketMetadata::EMPTY];
    //!
    //! let mut ping_manager = PingManager::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    //! let addr = "192.168.8.1";
    //! let mut ping_params = PingParams::new(Ipv4Addr::from_str(addr).unwrap());
    //! ping_params.set_payload(b"Hello, router!");
    //! match ping_manager.ping(&ping_params).await {
    //!     Ok(time) => info!("Ping time of {}: {}ms", addr, time.as_millis()),
    //!     Err(ping_error) => warn!("{:?}", ping_error),
    //! };
    //! ```

    use core::net::IpAddr;
    #[cfg(feature = "proto-ipv6")]
    use core::net::Ipv6Addr;

    use embassy_time::{Duration, Instant, Timer, WithTimeout};
    #[cfg(feature = "proto-ipv6")]
    use smoltcp::wire::IpAddress;
    #[cfg(feature = "proto-ipv6")]
    use smoltcp::wire::Ipv6Address;

    use super::*;

    /// Error returned by [`ping()`](PingManager::ping).
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PingError {
        /// The target did not respond.
        ///
        /// The packet was sent but the Reply packet has not been recieved
        /// in the timeout set by [`set_timeout()`](PingParams::set_timeout).
        DestinationHostUnreachable,
        /// The target has not been specified.
        InvalidTargetAddress,
        /// The source has not been specified (Ipv6 only).
        #[cfg(feature = "proto-ipv6")]
        InvalidSourceAddress,
        /// The socket could not queue the packet in the buffer.
        SocketSendTimeout,
        /// Container error for [`icmp::BindError`].
        SocketBindError(BindError),
        /// Container error for [`icmp::SendError`].
        SocketSendError(SendError),
        /// Container error for [`icmp::RecvError`].
        SocketRecvError(RecvError),
    }

    /// Manages ICMP ping operations.
    ///
    /// This struct provides functionality to send ICMP echo requests (pings) to a specified target
    /// and measure the round-trip time for the requests. It supports both IPv4 and IPv6, depending
    /// on the enabled features.
    ///
    /// # Fields
    ///
    /// * `stack` - The network stack instance used for managing network operations.
    /// * `rx_meta` - Metadata buffer for receiving packets.
    /// * `rx_buffer` - Buffer for receiving packets.
    /// * `tx_meta` - Metadata buffer for transmitting packets.
    /// * `tx_buffer` - Buffer for transmitting packets.
    /// * `ident` - Identifier for the ICMP echo requests.
    ///
    /// # Methods
    ///
    /// * [`new`](PingManager::new) - Creates a new instance of `PingManager` with the specified stack and buffers.
    /// * [`ping`](PingManager::ping) - Sends ICMP echo requests to the specified target and returns the average round-trip time.
    pub struct PingManager<'d> {
        stack: Stack<'d>,
        rx_meta: &'d mut [PacketMetadata],
        rx_buffer: &'d mut [u8],
        tx_meta: &'d mut [PacketMetadata],
        tx_buffer: &'d mut [u8],
        ident: u16,
    }

    impl<'d> PingManager<'d> {
        /// Creates a new instance of [`PingManager`] with a [`Stack`] instance
        /// and the buffers used for RX and TX.
        ///
        /// **note**: This does not yet creates the ICMP socket.
        pub fn new(
            stack: Stack<'d>,
            rx_meta: &'d mut [PacketMetadata],
            rx_buffer: &'d mut [u8],
            tx_meta: &'d mut [PacketMetadata],
            tx_buffer: &'d mut [u8],
        ) -> Self {
            Self {
                stack,
                rx_meta,
                rx_buffer,
                tx_meta,
                tx_buffer,
                ident: 0,
            }
        }

        /// Sends ICMP echo requests to the specified target and returns the average round-trip time.
        ///
        /// # Arguments
        ///
        /// * `params` - Parameters for configuring the ping operation.
        ///
        /// # Returns
        ///
        /// * `Ok(Duration)` - The average round-trip time for the ping requests.
        /// * `Err(PingError)` - An error occurred during the ping operation.
        pub async fn ping<'a>(&mut self, params: &PingParams<'a>) -> Result<Duration, PingError> {
            // Input validation
            if params.target().is_none() {
                return Err(PingError::InvalidTargetAddress);
            }
            #[cfg(feature = "proto-ipv6")]
            if params.target().unwrap().is_ipv6() && params.source().is_none() {
                return Err(PingError::InvalidSourceAddress);
            }
            // Increment the ident (wrapping u16) to respect standards
            self.ident = self.ident.wrapping_add(1u16);
            // Used to calculate the average duration
            let mut total_duration = Duration::default();
            let mut num_of_durations = 0u16;
            // Increment the sequence number as per standards
            for seq_no in 0..params.count() {
                // Make sure each ping takes at least 1 second to respect standards
                let rate_limit_start = Instant::now();

                // make a single ping
                // - shorts out errors
                // - select the ip version
                let ping_duration = match params.target.unwrap() {
                    #[cfg(feature = "proto-ipv4")]
                    IpAddress::Ipv4(_) => self.single_ping_v4(params, seq_no).await?,
                    #[cfg(feature = "proto-ipv6")]
                    IpAddress::Ipv6(_) => self.single_ping_v6(params, seq_no).await?,
                };

                // safely add up the durations of each ping
                if let Some(dur) = total_duration.checked_add(ping_duration) {
                    total_duration = dur;
                    num_of_durations += 1;
                }

                // 1 sec min per ping
                let rate_limit_end = rate_limit_start.elapsed();
                if rate_limit_end <= params.rate_limit {
                    Timer::after(params.rate_limit.checked_sub(rate_limit_end).unwrap()).await;
                }
            }
            // calculate and return the average duration
            Ok(total_duration.checked_div(num_of_durations as u32).unwrap())
        }

        #[cfg(feature = "proto-ipv4")]
        fn create_repr_ipv4<'b>(&self, params: &PingParams<'b>, seq_no: u16) -> Icmpv4Repr<'b> {
            Icmpv4Repr::EchoRequest {
                ident: self.ident,
                seq_no,
                data: params.payload,
            }
        }

        #[cfg(feature = "proto-ipv6")]
        fn create_repr_ipv6<'b>(&self, params: &PingParams<'b>, seq_no: u16) -> Icmpv6Repr<'b> {
            Icmpv6Repr::EchoRequest {
                ident: self.ident,
                seq_no,
                data: params.payload,
            }
        }

        #[cfg(feature = "proto-ipv4")]
        async fn single_ping_v4(&mut self, params: &PingParams<'_>, seq_no: u16) -> Result<Duration, PingError> {
            let ping_repr = self.create_repr_ipv4(params, seq_no);

            // Create the socket and set hop limit and bind it to the endpoint with the ident
            let mut socket = IcmpSocket::new(self.stack, self.rx_meta, self.rx_buffer, self.tx_meta, self.tx_buffer);
            socket.set_hop_limit(params.hop_limit);
            if let Err(e) = socket.bind(IcmpEndpoint::Ident(self.ident)) {
                return Err(PingError::SocketBindError(e));
            }

            // Helper func to fill the buffer when sending the ICMP packet
            fn fill_packet_buffer(buf: &mut [u8], ping_repr: Icmpv4Repr<'_>) -> Instant {
                let mut icmp_packet = Icmpv4Packet::new_unchecked(buf);
                ping_repr.emit(&mut icmp_packet, &ChecksumCapabilities::default());
                Instant::now()
            }

            // Send with timeout the ICMP packet filling it with the helper function
            let send_result = socket
                .send_to_with(ping_repr.buffer_len(), params.target.unwrap(), |buf| {
                    fill_packet_buffer(buf, ping_repr)
                })
                .with_timeout(Duration::from_millis(100))
                .await;
            // Filter and translate potential errors from sending the packet
            let now = match send_result {
                Ok(send_result) => match send_result {
                    Ok(i) => i,
                    Err(e) => return Err(PingError::SocketSendError(e)),
                },
                Err(_) => return Err(PingError::SocketSendTimeout),
            };

            // Helper function for the recieve helper function to validate the echo reply
            fn filter_pong(buf: &[u8], seq_no: u16) -> bool {
                let pong_packet = match Icmpv4Packet::new_checked(buf) {
                    Ok(pak) => pak,
                    Err(_) => return false,
                };
                pong_packet.echo_seq_no() == seq_no
            }

            // Helper function to recieve and return the correct echo reply when it finds it
            async fn recv_pong(socket: &IcmpSocket<'_>, seq_no: u16) -> Result<(), PingError> {
                while match socket.recv_from_with(|(buf, _)| filter_pong(buf, seq_no)).await {
                    Ok(b) => !b,
                    Err(e) => return Err(PingError::SocketRecvError(e)),
                } {}
                Ok(())
            }

            // Calls the recieve helper function with a timeout
            match recv_pong(&socket, seq_no).with_timeout(params.timeout).await {
                Ok(res) => res?,
                Err(_) => return Err(PingError::DestinationHostUnreachable),
            }

            // Return the round trip duration
            Ok(now.elapsed())
        }

        #[cfg(feature = "proto-ipv6")]
        async fn single_ping_v6(&mut self, params: &PingParams<'_>, seq_no: u16) -> Result<Duration, PingError> {
            let ping_repr = self.create_repr_ipv6(params, seq_no);

            // Create the socket and set hop limit and bind it to the endpoint with the ident
            let mut socket = IcmpSocket::new(self.stack, self.rx_meta, self.rx_buffer, self.tx_meta, self.tx_buffer);
            socket.set_hop_limit(params.hop_limit);
            if let Err(e) = socket.bind(IcmpEndpoint::Ident(self.ident)) {
                return Err(PingError::SocketBindError(e));
            }

            // Helper func to fill the buffer when sending the ICMP packet
            fn fill_packet_buffer(buf: &mut [u8], ping_repr: Icmpv6Repr<'_>, params: &PingParams<'_>) -> Instant {
                let mut icmp_packet = Icmpv6Packet::new_unchecked(buf);
                let target = match params.target().unwrap() {
                    IpAddr::V4(_) => unreachable!(),
                    IpAddr::V6(addr) => addr,
                };
                ping_repr.emit(
                    &params.source().unwrap(),
                    &target,
                    &mut icmp_packet,
                    &ChecksumCapabilities::default(),
                );
                Instant::now()
            }

            // Send with timeout the ICMP packet filling it with the helper function
            let send_result = socket
                .send_to_with(ping_repr.buffer_len(), params.target.unwrap(), |buf| {
                    fill_packet_buffer(buf, ping_repr, params)
                })
                .with_timeout(Duration::from_millis(100))
                .await;
            let now = match send_result {
                Ok(send_result) => match send_result {
                    Ok(i) => i,
                    Err(e) => return Err(PingError::SocketSendError(e)),
                },
                Err(_) => return Err(PingError::SocketSendTimeout),
            };

            // Helper function for the recieve helper function to validate the echo reply
            fn filter_pong(buf: &[u8], seq_no: u16) -> bool {
                let pong_packet = match Icmpv6Packet::new_checked(buf) {
                    Ok(pak) => pak,
                    Err(_) => return false,
                };
                pong_packet.echo_seq_no() == seq_no
            }

            // Helper function to recieve and return the correct echo reply when it finds it
            async fn recv_pong(socket: &IcmpSocket<'_>, seq_no: u16) -> Result<(), PingError> {
                while match socket.recv_from_with(|(buf, _)| filter_pong(buf, seq_no)).await {
                    Ok(b) => !b,
                    Err(e) => return Err(PingError::SocketRecvError(e)),
                } {}
                Ok(())
            }

            // Calls the recieve helper function with a timeout
            match recv_pong(&socket, seq_no).with_timeout(params.timeout).await {
                Ok(res) => res?,
                Err(_) => return Err(PingError::DestinationHostUnreachable),
            }

            // Return the round trip duration
            Ok(now.elapsed())
        }
    }

    /// Parameters for configuring the ping operation.
    ///
    /// This struct provides various configuration options for performing ICMP ping operations,
    /// including the target IP address, payload data, hop limit, number of pings, and timeout duration.
    ///
    /// # Fields
    ///
    /// * `target` - The target IP address for the ping operation.
    /// * `source` - The source IP address for the ping operation (IPv6 only).
    /// * `payload` - The data to be sent in the payload field of the ping.
    /// * `hop_limit` - The hop limit to be used by the socket.
    /// * `count` - The number of pings to be sent in one ping operation.
    /// * `timeout` - The timeout duration before returning a [`PingError::DestinationHostUnreachable`] error.
    /// * `rate_limit` - The minimum time per echo request.
    pub struct PingParams<'a> {
        target: Option<IpAddress>,
        #[cfg(feature = "proto-ipv6")]
        source: Option<Ipv6Address>,
        payload: &'a [u8],
        hop_limit: Option<u8>,
        count: u16,
        timeout: Duration,
        rate_limit: Duration,
    }

    impl Default for PingParams<'_> {
        fn default() -> Self {
            Self {
                target: None,
                #[cfg(feature = "proto-ipv6")]
                source: None,
                payload: b"embassy-net",
                hop_limit: None,
                count: 4,
                timeout: Duration::from_secs(4),
                rate_limit: Duration::from_secs(1),
            }
        }
    }

    impl<'a> PingParams<'a> {
        /// Creates a new instance of [`PingParams`] with the specified target IP address.
        pub fn new<T: Into<IpAddr>>(target: T) -> Self {
            Self {
                target: Some(PingParams::ip_addr_to_smoltcp(target)),
                #[cfg(feature = "proto-ipv6")]
                source: None,
                payload: b"embassy-net",
                hop_limit: None,
                count: 4,
                timeout: Duration::from_secs(4),
                rate_limit: Duration::from_secs(1),
            }
        }

        fn ip_addr_to_smoltcp<T: Into<IpAddr>>(ip_addr: T) -> IpAddress {
            match ip_addr.into() {
                #[cfg(feature = "proto-ipv4")]
                IpAddr::V4(v4) => IpAddress::Ipv4(v4),
                #[cfg(not(feature = "proto-ipv4"))]
                IpAddr::V4(_) => unreachable!(),
                #[cfg(feature = "proto-ipv6")]
                IpAddr::V6(v6) => IpAddress::Ipv6(v6),
                #[cfg(not(feature = "proto-ipv6"))]
                IpAddr::V6(_) => unreachable!(),
            }
        }

        /// Sets the target IP address for the ping.
        pub fn set_target<T: Into<IpAddr>>(&mut self, target: T) -> &mut Self {
            self.target = Some(PingParams::ip_addr_to_smoltcp(target));
            self
        }

        /// Retrieves the target IP address for the ping.
        pub fn target(&self) -> Option<IpAddr> {
            self.target.map(|t| t.into())
        }

        /// Sets the source IP address for the ping (IPv6 only).
        #[cfg(feature = "proto-ipv6")]
        pub fn set_source<T: Into<Ipv6Address>>(&mut self, source: T) -> &mut Self {
            self.source = Some(source.into());
            self
        }

        /// Retrieves the source IP address for the ping (IPv6 only).
        #[cfg(feature = "proto-ipv6")]
        pub fn source(&self) -> Option<Ipv6Addr> {
            self.source
        }

        /// Sets the data used in the payload field of the ping with the provided slice.
        pub fn set_payload(&mut self, payload: &'a [u8]) -> &mut Self {
            self.payload = payload;
            self
        }

        /// Gives a reference to the slice of data that's going to be sent in the payload field
        /// of the ping.
        pub fn payload(&self) -> &'a [u8] {
            self.payload
        }

        /// Sets the hop limit that will be used by the socket with [`set_hop_limit()`](IcmpSocket::set_hop_limit).
        ///
        /// **Note**: A hop limit of [`Some(0)`](Some()) is equivalent to a hop limit of [`None`].
        pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) -> &mut Self {
            let mut hop_limit = hop_limit;
            if hop_limit.is_some_and(|x| x == 0) {
                hop_limit = None
            }
            self.hop_limit = hop_limit;
            self
        }

        /// Retrieves the hop limit that will be used by the socket with [`set_hop_limit()`](IcmpSocket::set_hop_limit).
        pub fn hop_limit(&self) -> Option<u8> {
            self.hop_limit
        }

        /// Sets the count used for specifying the number of pings done on one
        /// [`ping()`](PingManager::ping) call.
        ///
        /// **Note**: A count of 0 will be set as 1.
        pub fn set_count(&mut self, count: u16) -> &mut Self {
            let mut count = count;
            if count == 0 {
                count = 1;
            }
            self.count = count;
            self
        }

        /// Retrieve the count used for specifying the number of pings done on one
        /// [`ping()`](PingManager::ping) call.
        pub fn count(&self) -> u16 {
            self.count
        }

        /// Sets the timeout used before returning [`PingError::DestinationHostUnreachable`]
        /// when waiting for the Echo Reply icmp packet.
        pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
            self.timeout = timeout;
            self
        }

        /// Retrieve the timeout used before returning [`PingError::DestinationHostUnreachable`]
        /// when waiting for the Echo Reply icmp packet.
        pub fn timeout(&self) -> Duration {
            self.timeout
        }

        /// Sets the `rate_limit`: minimum time per echo request.
        pub fn set_rate_limit(&mut self, rate_limit: Duration) -> &mut Self {
            self.rate_limit = rate_limit;
            self
        }

        /// Retrieve the rate_limit.
        pub fn rate_limit(&self) -> Duration {
            self.rate_limit
        }
    }
}
