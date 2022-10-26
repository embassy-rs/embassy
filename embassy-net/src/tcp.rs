use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::mem;
use core::task::Poll;

use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::tcp;
use smoltcp::time::Duration;
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};

use super::stack::Stack;
use crate::stack::SocketStack;
use crate::Device;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    ConnectionReset,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectError {
    /// The socket is already connected or listening.
    InvalidState,
    /// The remote host rejected the connection with a RST packet.
    ConnectionReset,
    /// Connect timed out.
    TimedOut,
    /// No route to host.
    NoRoute,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AcceptError {
    /// The socket is already connected or listening.
    InvalidState,
    /// Invalid listen port
    InvalidPort,
    /// The remote host rejected the connection with a RST packet.
    ConnectionReset,
}

pub struct TcpSocket<'a> {
    io: TcpIo<'a>,
}

pub struct TcpReader<'a> {
    io: TcpIo<'a>,
}

pub struct TcpWriter<'a> {
    io: TcpIo<'a>,
}

impl<'a> TcpReader<'a> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.io.read(buf).await
    }
}

impl<'a> TcpWriter<'a> {
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.io.write(buf).await
    }
}

impl<'a> TcpSocket<'a> {
    pub fn new<D: Device>(stack: &'a Stack<D>, rx_buffer: &'a mut [u8], tx_buffer: &'a mut [u8]) -> Self {
        // safety: not accessed reentrantly.
        let s = unsafe { &mut *stack.socket.get() };
        let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
        let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
        let handle = s.sockets.add(tcp::Socket::new(
            tcp::SocketBuffer::new(rx_buffer),
            tcp::SocketBuffer::new(tx_buffer),
        ));

        Self {
            io: TcpIo {
                stack: &stack.socket,
                handle,
            },
        }
    }

    pub fn split(&mut self) -> (TcpReader<'_>, TcpWriter<'_>) {
        (TcpReader { io: self.io }, TcpWriter { io: self.io })
    }

    pub async fn connect<T>(&mut self, remote_endpoint: T) -> Result<(), ConnectError>
    where
        T: Into<IpEndpoint>,
    {
        // safety: not accessed reentrantly.
        let local_port = unsafe { &mut *self.io.stack.get() }.get_local_port();

        // safety: not accessed reentrantly.
        match unsafe { self.io.with_mut(|s, i| s.connect(i, remote_endpoint, local_port)) } {
            Ok(()) => {}
            Err(tcp::ConnectError::InvalidState) => return Err(ConnectError::InvalidState),
            Err(tcp::ConnectError::Unaddressable) => return Err(ConnectError::NoRoute),
        }

        poll_fn(|cx| unsafe {
            self.io.with_mut(|s, _| match s.state() {
                tcp::State::Closed | tcp::State::TimeWait => Poll::Ready(Err(ConnectError::ConnectionReset)),
                tcp::State::Listen => unreachable!(),
                tcp::State::SynSent | tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub async fn accept<T>(&mut self, local_endpoint: T) -> Result<(), AcceptError>
    where
        T: Into<IpListenEndpoint>,
    {
        // safety: not accessed reentrantly.
        match unsafe { self.io.with_mut(|s, _| s.listen(local_endpoint)) } {
            Ok(()) => {}
            Err(tcp::ListenError::InvalidState) => return Err(AcceptError::InvalidState),
            Err(tcp::ListenError::Unaddressable) => return Err(AcceptError::InvalidPort),
        }

        poll_fn(|cx| unsafe {
            self.io.with_mut(|s, _| match s.state() {
                tcp::State::Listen | tcp::State::SynSent | tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.io.read(buf).await
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.io.write(buf).await
    }

    pub fn set_timeout(&mut self, duration: Option<Duration>) {
        unsafe { self.io.with_mut(|s, _| s.set_timeout(duration)) }
    }

    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        unsafe { self.io.with_mut(|s, _| s.set_keep_alive(interval)) }
    }

    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        unsafe { self.io.with_mut(|s, _| s.set_hop_limit(hop_limit)) }
    }

    pub fn local_endpoint(&self) -> Option<IpEndpoint> {
        unsafe { self.io.with(|s, _| s.local_endpoint()) }
    }

    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        unsafe { self.io.with(|s, _| s.remote_endpoint()) }
    }

    pub fn state(&self) -> tcp::State {
        unsafe { self.io.with(|s, _| s.state()) }
    }

    pub fn close(&mut self) {
        unsafe { self.io.with_mut(|s, _| s.close()) }
    }

    pub fn abort(&mut self) {
        unsafe { self.io.with_mut(|s, _| s.abort()) }
    }

    pub fn may_send(&self) -> bool {
        unsafe { self.io.with(|s, _| s.may_send()) }
    }

    pub fn may_recv(&self) -> bool {
        unsafe { self.io.with(|s, _| s.may_recv()) }
    }
}

impl<'a> Drop for TcpSocket<'a> {
    fn drop(&mut self) {
        // safety: not accessed reentrantly.
        let s = unsafe { &mut *self.io.stack.get() };
        s.sockets.remove(self.io.handle);
    }
}

// =======================

#[derive(Copy, Clone)]
struct TcpIo<'a> {
    stack: &'a UnsafeCell<SocketStack>,
    handle: SocketHandle,
}

impl<'d> TcpIo<'d> {
    /// SAFETY: must not call reentrantly.
    unsafe fn with<R>(&self, f: impl FnOnce(&tcp::Socket, &Interface) -> R) -> R {
        let s = &*self.stack.get();
        let socket = s.sockets.get::<tcp::Socket>(self.handle);
        f(socket, &s.iface)
    }

    /// SAFETY: must not call reentrantly.
    unsafe fn with_mut<R>(&mut self, f: impl FnOnce(&mut tcp::Socket, &mut Interface) -> R) -> R {
        let s = &mut *self.stack.get();
        let socket = s.sockets.get_mut::<tcp::Socket>(self.handle);
        let res = f(socket, &mut s.iface);
        s.waker.wake();
        res
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| unsafe {
            // CAUTION: smoltcp semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            self.with_mut(|s, _| match s.recv_slice(buf) {
                // No data ready
                Ok(0) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                // Data ready!
                Ok(n) => Poll::Ready(Ok(n)),
                // EOF
                Err(tcp::RecvError::Finished) => Poll::Ready(Ok(0)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(tcp::RecvError::InvalidState) => Poll::Ready(Err(Error::ConnectionReset)),
            })
        })
        .await
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        poll_fn(move |cx| unsafe {
            self.with_mut(|s, _| match s.send_slice(buf) {
                // Not ready to send (no space in the tx buffer)
                Ok(0) => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                // Some data sent
                Ok(n) => Poll::Ready(Ok(n)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(tcp::SendError::InvalidState) => Poll::Ready(Err(Error::ConnectionReset)),
            })
        })
        .await
    }

    #[allow(unused)]
    async fn flush(&mut self) -> Result<(), Error> {
        poll_fn(move |_| {
            Poll::Ready(Ok(())) // TODO: Is there a better implementation for this?
        })
        .await
    }
}

#[cfg(feature = "nightly")]
mod embedded_io_impls {
    use core::future::Future;

    use super::*;

    impl embedded_io::Error for ConnectError {
        fn kind(&self) -> embedded_io::ErrorKind {
            embedded_io::ErrorKind::Other
        }
    }

    impl embedded_io::Error for Error {
        fn kind(&self) -> embedded_io::ErrorKind {
            embedded_io::ErrorKind::Other
        }
    }

    impl<'d> embedded_io::Io for TcpSocket<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io::asynch::Read for TcpSocket<'d> {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.io.read(buf)
        }
    }

    impl<'d> embedded_io::asynch::Write for TcpSocket<'d> {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.io.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            self.io.flush()
        }
    }

    impl<'d> embedded_io::Io for TcpReader<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io::asynch::Read for TcpReader<'d> {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.io.read(buf)
        }
    }

    impl<'d> embedded_io::Io for TcpWriter<'d> {
        type Error = Error;
    }

    impl<'d> embedded_io::asynch::Write for TcpWriter<'d> {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.io.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            self.io.flush()
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
pub mod client {
    use core::future::Future;
    use core::mem::MaybeUninit;
    use core::ptr::NonNull;

    use atomic_polyfill::{AtomicBool, Ordering};
    use embedded_nal_async::IpAddr;

    use super::*;

    /// TCP client capable of creating up to N multiple connections with tx and rx buffers according to TX_SZ and RX_SZ.
    pub struct TcpClient<'d, D: Device, const N: usize, const TX_SZ: usize = 1024, const RX_SZ: usize = 1024> {
        stack: &'d Stack<D>,
        state: &'d TcpClientState<N, TX_SZ, RX_SZ>,
    }

    impl<'d, D: Device, const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpClient<'d, D, N, TX_SZ, RX_SZ> {
        /// Create a new TcpClient
        pub fn new(stack: &'d Stack<D>, state: &'d TcpClientState<N, TX_SZ, RX_SZ>) -> Self {
            Self { stack, state }
        }
    }

    impl<'d, D: Device, const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_nal_async::TcpConnect
        for TcpClient<'d, D, N, TX_SZ, RX_SZ>
    {
        type Error = Error;
        type Connection<'m> = TcpConnection<'m, N, TX_SZ, RX_SZ> where Self: 'm;
        type ConnectFuture<'m> = impl Future<Output = Result<Self::Connection<'m>, Self::Error>> + 'm
    where
        Self: 'm;

        fn connect<'m>(&'m self, remote: embedded_nal_async::SocketAddr) -> Self::ConnectFuture<'m> {
            async move {
                let addr: crate::IpAddress = match remote.ip() {
                    IpAddr::V4(addr) => crate::IpAddress::Ipv4(crate::Ipv4Address::from_bytes(&addr.octets())),
                    #[cfg(feature = "proto-ipv6")]
                    IpAddr::V6(addr) => crate::IpAddress::Ipv6(crate::Ipv6Address::from_bytes(&addr.octets())),
                    #[cfg(not(feature = "proto-ipv6"))]
                    IpAddr::V6(_) => panic!("ipv6 support not enabled"),
                };
                let remote_endpoint = (addr, remote.port());
                let mut socket = TcpConnection::new(&self.stack, self.state)?;
                socket
                    .socket
                    .connect(remote_endpoint)
                    .await
                    .map_err(|_| Error::ConnectionReset)?;
                Ok(socket)
            }
        }
    }

    pub struct TcpConnection<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> {
        socket: TcpSocket<'d>,
        state: &'d TcpClientState<N, TX_SZ, RX_SZ>,
        bufs: NonNull<([u8; TX_SZ], [u8; RX_SZ])>,
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpConnection<'d, N, TX_SZ, RX_SZ> {
        fn new<D: Device>(stack: &'d Stack<D>, state: &'d TcpClientState<N, TX_SZ, RX_SZ>) -> Result<Self, Error> {
            let mut bufs = state.pool.alloc().ok_or(Error::ConnectionReset)?;
            Ok(Self {
                socket: unsafe { TcpSocket::new(stack, &mut bufs.as_mut().0, &mut bufs.as_mut().1) },
                state,
                bufs,
            })
        }
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> Drop for TcpConnection<'d, N, TX_SZ, RX_SZ> {
        fn drop(&mut self) {
            unsafe {
                self.socket.close();
                self.state.pool.free(self.bufs);
            }
        }
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io::Io
        for TcpConnection<'d, N, TX_SZ, RX_SZ>
    {
        type Error = Error;
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io::asynch::Read
        for TcpConnection<'d, N, TX_SZ, RX_SZ>
    {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.socket.read(buf)
        }
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io::asynch::Write
        for TcpConnection<'d, N, TX_SZ, RX_SZ>
    {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.socket.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            self.socket.flush()
        }
    }

    /// State for TcpClient
    pub struct TcpClientState<const N: usize, const TX_SZ: usize, const RX_SZ: usize> {
        pool: Pool<([u8; TX_SZ], [u8; RX_SZ]), N>,
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpClientState<N, TX_SZ, RX_SZ> {
        pub const fn new() -> Self {
            Self { pool: Pool::new() }
        }
    }

    unsafe impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> Sync for TcpClientState<N, TX_SZ, RX_SZ> {}

    struct Pool<T, const N: usize> {
        used: [AtomicBool; N],
        data: [UnsafeCell<MaybeUninit<T>>; N],
    }

    impl<T, const N: usize> Pool<T, N> {
        const VALUE: AtomicBool = AtomicBool::new(false);
        const UNINIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

        const fn new() -> Self {
            Self {
                used: [Self::VALUE; N],
                data: [Self::UNINIT; N],
            }
        }
    }

    impl<T, const N: usize> Pool<T, N> {
        fn alloc(&self) -> Option<NonNull<T>> {
            for n in 0..N {
                if self.used[n].swap(true, Ordering::SeqCst) == false {
                    let p = self.data[n].get() as *mut T;
                    return Some(unsafe { NonNull::new_unchecked(p) });
                }
            }
            None
        }

        /// safety: p must be a pointer obtained from self.alloc that hasn't been freed yet.
        unsafe fn free(&self, p: NonNull<T>) {
            let origin = self.data.as_ptr() as *mut T;
            let n = p.as_ptr().offset_from(origin);
            assert!(n >= 0);
            assert!((n as usize) < N);
            self.used[n as usize].store(false, Ordering::SeqCst);
        }
    }
}
