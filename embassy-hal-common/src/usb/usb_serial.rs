use core::cell::RefCell;
use core::marker::{PhantomData, Unpin};
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy::io::{self, AsyncBufRead, AsyncWrite};
use embassy::waitqueue::WakerRegistration;
use usb_device::bus::UsbBus;
use usb_device::class_prelude::*;
use usb_device::UsbError;

use super::cdc_acm::CdcAcmClass;
use super::StateInner;
use crate::peripheral::PeripheralMutex;
use crate::ring_buffer::RingBuffer;
use crate::usb::{ClassSet, SerialState, USBInterrupt};

pub struct ReadInterface<'a, 'bus, 'c, I, B, T, INT>
where
    I: Unpin,
    B: UsbBus,
    T: SerialState<'bus, 'c, B, I> + ClassSet<B>,
    INT: USBInterrupt,
{
    // Don't you dare moving out `PeripheralMutex`
    pub(crate) inner: &'a RefCell<PeripheralMutex<'bus, StateInner<'bus, B, T, INT>>>,
    pub(crate) _buf_lifetime: PhantomData<&'c T>,
    pub(crate) _index: PhantomData<I>,
}

/// Write interface for USB CDC_ACM
///
/// This interface is buffered, meaning that after the write returns the bytes might not be fully
/// on the wire just yet
pub struct WriteInterface<'a, 'bus, 'c, I, B, T, INT>
where
    I: Unpin,
    B: UsbBus,
    T: SerialState<'bus, 'c, B, I> + ClassSet<B>,
    INT: USBInterrupt,
{
    // Don't you dare moving out `PeripheralMutex`
    pub(crate) inner: &'a RefCell<PeripheralMutex<'bus, StateInner<'bus, B, T, INT>>>,
    pub(crate) _buf_lifetime: PhantomData<&'c T>,
    pub(crate) _index: PhantomData<I>,
}

impl<'a, 'bus, 'c, I, B, T, INT> AsyncBufRead for ReadInterface<'a, 'bus, 'c, I, B, T, INT>
where
    I: Unpin,
    B: UsbBus,
    T: SerialState<'bus, 'c, B, I> + ClassSet<B>,
    INT: USBInterrupt,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let this = self.get_mut();
        let mut mutex = this.inner.borrow_mut();
        mutex.with(|state| {
            let serial = state.classes.get_serial();
            let serial = Pin::new(serial);

            match serial.poll_fill_buf(cx) {
                Poll::Ready(Ok(buf)) => {
                    let buf: &[u8] = buf;
                    // NOTE(unsafe) This part of the buffer won't be modified until the user calls
                    // consume, which will invalidate this ref
                    let buf: &[u8] = unsafe { core::mem::transmute(buf) };
                    Poll::Ready(Ok(buf))
                }
                Poll::Ready(Err(_)) => Poll::Ready(Err(io::Error::Other)),
                Poll::Pending => Poll::Pending,
            }
        })
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = self.get_mut();
        let mut mutex = this.inner.borrow_mut();
        mutex.with(|state| {
            let serial = state.classes.get_serial();
            let serial = Pin::new(serial);

            serial.consume(amt);
        })
    }
}

impl<'a, 'bus, 'c, I, B, T, INT> AsyncWrite for WriteInterface<'a, 'bus, 'c, I, B, T, INT>
where
    I: Unpin,
    B: UsbBus,
    T: SerialState<'bus, 'c, B, I> + ClassSet<B>,
    INT: USBInterrupt,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let mut mutex = this.inner.borrow_mut();
        mutex.with(|state| {
            let serial = state.classes.get_serial();
            let serial = Pin::new(serial);

            serial.poll_write(cx, buf)
        })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let mut mutex = this.inner.borrow_mut();
        mutex.with(|state| {
            let serial = state.classes.get_serial();
            let serial = Pin::new(serial);

            serial.poll_flush(cx)
        })
    }
}

pub struct UsbSerial<'bus, 'a, B: UsbBus> {
    inner: CdcAcmClass<'bus, B>,
    read_buf: RingBuffer<'a>,
    write_buf: RingBuffer<'a>,
    read_waker: WakerRegistration,
    write_waker: WakerRegistration,
    write_state: WriteState,
    read_error: bool,
    write_error: bool,
}

impl<'bus, 'a, B: UsbBus> AsyncBufRead for UsbSerial<'bus, 'a, B> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let this = self.get_mut();

        if this.read_error {
            this.read_error = false;
            return Poll::Ready(Err(io::Error::Other));
        }

        let buf = this.read_buf.pop_buf();
        if buf.is_empty() {
            this.read_waker.register(cx.waker());
            return Poll::Pending;
        }
        Poll::Ready(Ok(buf))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.get_mut().read_buf.pop(amt);
    }
}

impl<'bus, 'a, B: UsbBus> AsyncWrite for UsbSerial<'bus, 'a, B> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        if this.write_error {
            this.write_error = false;
            return Poll::Ready(Err(io::Error::Other));
        }

        let write_buf = this.write_buf.push_buf();
        if write_buf.is_empty() {
            trace!("buf full, registering waker");
            this.write_waker.register(cx.waker());
            return Poll::Pending;
        }

        let count = write_buf.len().min(buf.len());
        write_buf[..count].copy_from_slice(&buf[..count]);
        this.write_buf.push(count);

        this.flush_write();
        Poll::Ready(Ok(count))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Keeps track of the type of the last written packet.
enum WriteState {
    /// No packets in-flight
    Idle,

    /// Short packet currently in-flight
    Short,

    /// Full packet current in-flight. A full packet must be followed by a short packet for the host
    /// OS to see the transaction. The data is the number of subsequent full packets sent so far. A
    /// short packet is forced every SHORT_PACKET_INTERVAL packets so that the OS sees data in a
    /// timely manner.
    Full(usize),
}

impl<'bus, 'a, B: UsbBus> UsbSerial<'bus, 'a, B> {
    pub fn new(
        alloc: &'bus UsbBusAllocator<B>,
        read_buf: &'a mut [u8],
        write_buf: &'a mut [u8],
    ) -> Self {
        Self {
            inner: CdcAcmClass::new(alloc, 64),
            read_buf: RingBuffer::new(read_buf),
            write_buf: RingBuffer::new(write_buf),
            read_waker: WakerRegistration::new(),
            write_waker: WakerRegistration::new(),
            write_state: WriteState::Idle,
            read_error: false,
            write_error: false,
        }
    }

    fn flush_write(&mut self) {
        /// If this many full size packets have been sent in a row, a short packet will be sent so that the
        /// host sees the data in a timely manner.
        const SHORT_PACKET_INTERVAL: usize = 10;

        let full_size_packets = match self.write_state {
            WriteState::Full(c) => c,
            _ => 0,
        };

        let ep_size = self.inner.max_packet_size() as usize;
        let max_size = if full_size_packets > SHORT_PACKET_INTERVAL {
            ep_size - 1
        } else {
            ep_size
        };

        let buf = {
            let buf = self.write_buf.pop_buf();
            if buf.len() > max_size {
                &buf[..max_size]
            } else {
                buf
            }
        };

        if !buf.is_empty() {
            trace!("writing packet len {}", buf.len());
            let count = match self.inner.write_packet(buf) {
                Ok(c) => {
                    trace!("write packet: OK {}", c);
                    c
                }
                Err(UsbError::WouldBlock) => {
                    trace!("write packet: WouldBlock");
                    0
                }
                Err(_) => {
                    trace!("write packet: error");
                    self.write_error = true;
                    return;
                }
            };

            if buf.len() == ep_size {
                self.write_state = WriteState::Full(full_size_packets + 1);
            } else {
                self.write_state = WriteState::Short;
            }
            self.write_buf.pop(count);
        } else if full_size_packets > 0 {
            trace!("writing empty packet");
            match self.inner.write_packet(&[]) {
                Ok(_) => {
                    trace!("write empty packet: OK");
                }
                Err(UsbError::WouldBlock) => {
                    trace!("write empty packet: WouldBlock");
                    return;
                }
                Err(_) => {
                    trace!("write empty packet: Error");
                    self.write_error = true;
                    return;
                }
            }
            self.write_state = WriteState::Idle;
        }
    }
}

impl<B> UsbClass<B> for UsbSerial<'_, '_, B>
where
    B: UsbBus,
{
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<(), UsbError> {
        self.inner.get_configuration_descriptors(writer)
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.read_buf.clear();
        self.write_buf.clear();
        self.write_state = WriteState::Idle;
        self.read_waker.wake();
        self.write_waker.wake();
    }

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        trace!("DONE endpoint_in_complete");
        if addr == self.inner.write_ep_address() {
            trace!("DONE writing packet, waking");
            self.write_waker.wake();

            self.flush_write();
        }
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        if addr == self.inner.read_ep_address() {
            let buf = self.read_buf.push_buf();
            let count = match self.inner.read_packet(buf) {
                Ok(c) => c,
                Err(UsbError::WouldBlock) => 0,
                Err(_) => {
                    self.read_error = true;
                    return;
                }
            };

            if count > 0 {
                self.read_buf.push(count);
                self.read_waker.wake();
            }
        }
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        self.inner.control_in(xfer);
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        self.inner.control_out(xfer);
    }
}
