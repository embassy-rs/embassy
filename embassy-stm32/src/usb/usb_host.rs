#![macro_use]
#![allow(missing_docs)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Timer;
use embassy_usb_driver::host::{channel, ChannelError, HostError, UsbChannel, UsbHostDriver};
use embassy_usb_driver::EndpointType;

use super::{DmPin, DpPin, Instance};
use crate::pac::usb::regs;
use crate::pac::usb::vals::{EpType, Stat};
use crate::pac::USBRAM;
use crate::{interrupt, Peripheral};

/// The number of registers is 8, allowing up to 16 mono-
/// directional/single-buffer or up to 7 double-buffer endpoints in any combination. For
/// example the USB peripheral can be programmed to have 4 double buffer endpoints
/// and 8 single-buffer/mono-directional endpoints.
const USB_MAX_PIPES: usize = 8;

/// Interrupt handler.
pub struct USBHostInterruptHandler<I: Instance> {
    _phantom: PhantomData<I>,
}

impl<I: Instance> interrupt::typelevel::Handler<I::Interrupt> for USBHostInterruptHandler<I> {
    unsafe fn on_interrupt() {
        let regs = I::regs();
        // let x = regs.istr().read().0;
        // trace!("USB IRQ: {:08x}", x);

        let istr = regs.istr().read();

        // Detect device connect/disconnect
        if istr.reset() {
            trace!("USB IRQ: device connect/disconnect");

            // Write 0 to clear.
            let mut clear = regs::Istr(!0);
            clear.set_reset(false);
            regs.istr().write_value(clear);

            // Wake main thread.
            BUS_WAKER.wake();
        }

        if istr.ctr() {
            let index = istr.ep_id() as usize;

            let epr = regs.epr(index).read();

            match epr.stat_rx() {
                Stat::DISABLED => {} // debug!("Stat::DISABLED"),
                Stat::STALL => debug!("Stat::STALL"),
                Stat::NAK => {} //debug!("Stat::NAK"),
                Stat::VALID => debug!("Stat::VALID"),
            }
            if epr.ctr_rx() {
                EP_IN_WAKERS[index].wake();
            }
            if epr.ctr_tx() {
                EP_OUT_WAKERS[index].wake();
            }

            // Clear ctr flags
            let mut epr_value = invariant(epr);
            if epr.err_tx() {
                epr_value.set_err_tx(false);
                warn!("err_tx");
            }
            if epr.err_rx() {
                epr_value.set_err_rx(false);
                warn!("err_rx");
            }
            epr_value.set_ctr_rx(!epr.ctr_rx());
            epr_value.set_ctr_tx(!epr.ctr_tx());
            regs.epr(index).write_value(epr_value);
        }

        if istr.err() {
            debug!("USB IRQ: err");
            regs.istr().write_value(regs::Istr(!0));

            // Write 0 to clear.
            let mut clear = regs::Istr(!0);
            clear.set_err(false);
            regs.istr().write_value(clear);

            let index = istr.ep_id() as usize;
            let mut epr = regs.epr(index).read();
            // Toggle endponit to disabled
            epr.set_stat_rx(epr.stat_rx());
            epr.set_stat_tx(epr.stat_tx());
            regs.epr(index).write_value(epr);
        }
    }
}

const EP_COUNT: usize = 8;

#[cfg(any(usbram_16x1_512, usbram_16x2_512))]
const USBRAM_SIZE: usize = 512;
#[cfg(any(usbram_16x2_1024, usbram_32_1024))]
const USBRAM_SIZE: usize = 1024;
#[cfg(usbram_32_2048)]
const USBRAM_SIZE: usize = 2048;

#[cfg(not(any(usbram_32_2048, usbram_32_1024)))]
const USBRAM_ALIGN: usize = 2;
#[cfg(any(usbram_32_2048, usbram_32_1024))]
const USBRAM_ALIGN: usize = 4;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];

fn convert_type(t: EndpointType) -> EpType {
    match t {
        EndpointType::Bulk => EpType::BULK,
        EndpointType::Control => EpType::CONTROL,
        EndpointType::Interrupt => EpType::INTERRUPT,
        EndpointType::Isochronous => EpType::ISO,
    }
}

fn invariant(mut r: regs::Epr) -> regs::Epr {
    r.set_ctr_rx(true); // don't clear
    r.set_ctr_tx(true); // don't clear
    r.set_dtog_rx(false); // don't toggle
    r.set_dtog_tx(false); // don't toggle
    r.set_stat_rx(Stat::from_bits(0));
    r.set_stat_tx(Stat::from_bits(0));
    r
}

fn align_len_up(len: u16) -> u16 {
    ((len as usize + USBRAM_ALIGN - 1) / USBRAM_ALIGN * USBRAM_ALIGN) as u16
}

/// Calculates the register field values for configuring receive buffer descriptor.
/// Returns `(actual_len, len_bits)`
///
/// `actual_len` length in bytes rounded up to USBRAM_ALIGN
/// `len_bits` should be placed on the upper 16 bits of the register value
fn calc_receive_len_bits(len: u16) -> (u16, u16) {
    match len {
        // NOTE: this could be 2..=62 with 16bit USBRAM, but not with 32bit. Limit it to 60 for simplicity.
        2..=60 => (align_len_up(len), align_len_up(len) / 2 << 10),
        61..=1024 => ((len + 31) / 32 * 32, (((len + 31) / 32 - 1) << 10) | 0x8000),
        _ => panic!("invalid OUT length {}", len),
    }
}

#[cfg(any(usbram_32_2048, usbram_32_1024))]
mod btable {
    use super::*;

    pub(super) fn write_in<I: Instance>(_index: usize, _addr: u16) {}

    // // Write to Transmit Buffer Descriptor for Channel/endpoint (USB_CHEP_TXRXBD_n)
    // // Device: IN endpoint
    // // Host: Out endpoint
    // // Address offset: n*8 [bytes] or n*2 in 32 bit words
    // pub(super) fn write_in_len<I: Instance>(index: usize, addr: u16, len: u16) {
    //     USBRAM.mem(index * 2).write_value((addr as u32) | ((len as u32) << 16));
    // }

    // TODO: Replaces write_in_len
    /// Writes to Transmit Buffer Descriptor for Channel/endpoint `index``
    /// For Device this is an IN endpoint for Host an OUT endpoint
    pub(super) fn write_transmit_buffer_descriptor<I: Instance>(index: usize, addr: u16, len: u16) {
        // Address offset: index*8 [bytes] thus index*2 in 32 bit words
        USBRAM.mem(index * 2).write_value((addr as u32) | ((len as u32) << 16));
    }

    // Replaces write_out
    /// Writes to Receive Buffer Descriptor for Channel/endpoint `index``
    /// For Device this is an OUT endpoint for Host an IN endpoint
    pub(super) fn write_receive_buffer_descriptor<I: Instance>(index: usize, addr: u16, max_len_bits: u16) {
        // Address offset: index*8 + 4 [bytes] thus index*2 + 1 in 32 bit words
        USBRAM
            .mem(index * 2 + 1)
            .write_value((addr as u32) | ((max_len_bits as u32) << 16));
    }

    pub(super) fn read_out_len<I: Instance>(index: usize) -> u16 {
        (USBRAM.mem(index * 2 + 1).read() >> 16) as u16
    }
}

// Maybe replace with struct that only knows its index
struct EndpointBuffer<I: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<I>,
}

impl<I: Instance> EndpointBuffer<I> {
    fn new(addr: u16, len: u16) -> Self {
        EndpointBuffer {
            addr,
            len,
            _phantom: PhantomData,
        }
    }

    fn read(&mut self, buf: &mut [u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..(buf.len() + USBRAM_ALIGN - 1) / USBRAM_ALIGN {
            let val = USBRAM.mem(self.addr as usize / USBRAM_ALIGN + i).read();
            let n = USBRAM_ALIGN.min(buf.len() - i * USBRAM_ALIGN);
            buf[i * USBRAM_ALIGN..][..n].copy_from_slice(&val.to_le_bytes()[..n]);
        }
    }

    fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..(buf.len() + USBRAM_ALIGN - 1) / USBRAM_ALIGN {
            let mut val = [0u8; USBRAM_ALIGN];
            let n = USBRAM_ALIGN.min(buf.len() - i * USBRAM_ALIGN);
            val[..n].copy_from_slice(&buf[i * USBRAM_ALIGN..][..n]);

            #[cfg(not(any(usbram_32_2048, usbram_32_1024)))]
            let val = u16::from_le_bytes(val);
            #[cfg(any(usbram_32_2048, usbram_32_1024))]
            let val = u32::from_le_bytes(val);
            USBRAM.mem(self.addr as usize / USBRAM_ALIGN + i).write_value(val);
        }
    }
}

/// First bit is used to indicate control pipes
/// bitfield for keeping track of used channels
static ALLOCATED_PIPES: AtomicU32 = AtomicU32::new(0);
static EP_MEM_FREE: AtomicU16 = AtomicU16::new(0);

/// USB host driver.
pub struct USBHostDriver<'d, I: Instance> {
    phantom: PhantomData<&'d mut I>,
    // first free address in EP mem, in bytes.
    // ep_mem_free: u16,
}

impl<'d, I: Instance> USBHostDriver<'d, I> {
    /// Create a new USB driver.
    pub fn new(
        _usb: impl Peripheral<P = I> + 'd,
        _irq: impl interrupt::typelevel::Binding<I::Interrupt, USBHostInterruptHandler<I>> + 'd,
        dp: impl Peripheral<P = impl DpPin<I>> + 'd,
        dm: impl Peripheral<P = impl DmPin<I>> + 'd,
    ) -> Self {
        into_ref!(dp, dm);

        super::super::common_init::<I>();

        let regs = I::regs();

        regs.cntr().write(|w| {
            w.set_pdwn(false);
            w.set_fres(true);
            w.set_host(true);
        });

        // Wait for voltage reference
        #[cfg(feature = "time")]
        embassy_time::block_for(embassy_time::Duration::from_millis(100));
        #[cfg(not(feature = "time"))]
        cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 10);

        #[cfg(not(usb_v4))]
        regs.btable().write(|w| w.set_btable(0));

        #[cfg(not(stm32l1))]
        {
            use crate::gpio::{AfType, OutputType, Speed};
            dp.set_as_af(dp.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));
            dm.set_as_af(dm.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));
        }
        #[cfg(stm32l1)]
        let _ = (dp, dm); // suppress "unused" warnings.

        EP_MEM_FREE.store(EP_COUNT as u16 * 8, Ordering::Relaxed);
        Self {
            phantom: PhantomData,
            // ep_mem_free: EP_COUNT as u16 * 8, // for each EP, 4 regs, so 8 bytes
            // control_channel_in: Channel::new(0, 0, 0, 0),
            // control_channel_out: Channel::new(0, 0, 0, 0),
            // channels_used: 0,
            // channels_out_used: 0,
        }
    }

    /// Start the USB peripheral
    pub fn start(&mut self) {
        // let _ = self.reconfigure_channel0(8, 0);

        let regs = I::regs();

        regs.cntr().write(|w| {
            w.set_host(true);
            w.set_pdwn(false);
            w.set_fres(false);
            // Masks
            w.set_resetm(true);
            w.set_suspm(false);
            w.set_wkupm(false);
            w.set_ctrm(true);
            w.set_errm(false);
        });

        // Enable pull downs on DP and DM lines for host mode
        #[cfg(any(usb_v3, usb_v4))]
        regs.bcdr().write(|w| w.set_dppu(true));

        #[cfg(stm32l1)]
        crate::pac::SYSCFG.pmc().modify(|w| w.set_usb_pu(true));

        // HostControlPipe::new(ep_in, ep_out, control_max_packet_size)
    }

    pub fn get_status(&self) -> u32 {
        let regs = I::regs();

        let istr = regs.istr().read();

        istr.0
    }

    fn reset_alloc(&mut self) {
        // Reset alloc pointer.
        // self.ep_mem_free = EP_COUNT as u16 * 8; // for each EP, 4 regs, so 8 bytes

        // self.channels_used = 0;
        // self.channels_out_used = 0;
    }

    fn alloc_channel_mem(&self, len: u16) -> Result<u16, ()> {
        assert!(len as usize % USBRAM_ALIGN == 0);
        let addr = EP_MEM_FREE.load(Ordering::Relaxed);
        if addr + len > USBRAM_SIZE as _ {
            // panic!("Endpoint memory full");
            error!("Endpoint memory full");
            return Err(());
        }
        EP_MEM_FREE.store(addr + len, Ordering::Relaxed);
        Ok(addr)
    }

    // fn claim_channel_in(
    //     &mut self,
    //     index: usize,
    //     max_packet_size: u16,
    //     ep_type: EpType,
    //     dev_addr: u8,
    // ) -> Result<Channel<'d, T, In>, ()> {
    //     if self.channels_in_used & (1 << index) != 0 {
    //         error!("Channel {} In already in use", index);
    //         return Err(());
    //     }

    //     self.channels_in_used |= 1 << index;

    //     let (len, len_bits) = calc_receive_len_bits(max_packet_size);
    //     let Ok(addr) = self.alloc_channel_mem(len) else {
    //         return Err(());
    //     };

    //     btable::write_receive_buffer_descriptor::<I>(index, addr, len_bits);

    //     let in_channel: Channel<I, In> = Channel::new(index, addr, len, max_packet_size);

    //     // configure channel register
    //     let epr_reg = I::regs().epr(index);
    //     let mut epr = invariant(epr_reg.read());
    //     epr.set_devaddr(dev_addr);
    //     epr.set_ep_type(ep_type);
    //     epr.set_ea(index as _);
    //     epr_reg.write_value(epr);

    //     Ok(in_channel)
    // }

    // fn claim_channel_out(
    //     &mut self,
    //     index: usize,
    //     max_packet_size: u16,
    //     ep_type: EpType,
    //     dev_addr: u8,
    // ) -> Result<Channel<'d, T, Out>, ()> {
    //     if self.channels_out_used & (1 << index) != 0 {
    //         error!("Channel {} In already in use", index);
    //         return Err(());
    //     }
    //     self.channels_out_used |= 1 << index;

    //     let len = align_len_up(max_packet_size);
    //     let Ok(addr) = self.alloc_channel_mem(len) else {
    //         return Err(());
    //     };

    //     // ep_in_len is written when actually TXing packets.
    //     btable::write_in::<I>(index, addr);

    //     let out_channel: Channel<I, Out> = Channel::new(index, addr, len, max_packet_size);

    //     // configure channel register
    //     let epr_reg = I::regs().epr(index);
    //     let mut epr = invariant(epr_reg.read());
    //     epr.set_devaddr(dev_addr);
    //     epr.set_ep_type(ep_type);
    //     epr.set_ea(index as _);
    //     epr_reg.write_value(epr);

    //     Ok(out_channel)
    // }
}

// struct EndpointBuffer

/// USB endpoint. Only implements single buffer mode.
pub struct Channel<'d, I: Instance, D: channel::Direction, T: channel::Type> {
    _phantom: PhantomData<(&'d mut I, D, T)>,
    /// Register index (there are 8 in total)
    index: usize,
    max_packet_size_in: u16,
    max_packet_size_out: u16,
    buf_in: Option<EndpointBuffer<I>>,
    buf_out: Option<EndpointBuffer<I>>,
}

impl<'d, I: Instance, D: channel::Direction, T: channel::Type> Channel<'d, I, D, T> {
    fn new(
        index: usize,
        buf_in: Option<EndpointBuffer<I>>,
        buf_out: Option<EndpointBuffer<I>>,
        max_packet_size_in: u16,
        max_packet_size_out: u16,
    ) -> Self {
        Self {
            _phantom: PhantomData,
            index,
            max_packet_size_in,
            max_packet_size_out,
            buf_in,
            buf_out,
        }
    }
    fn read_data(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError> {
        let index = self.index;
        let rx_len = btable::read_out_len::<I>(index) as usize & 0x3FF;
        trace!("READ DONE, rx_len = {}", rx_len);
        if rx_len > buf.len() {
            return Err(ChannelError::BufferOverflow);
        }
        self.buf_in.as_mut().unwrap().read(&mut buf[..rx_len]);
        Ok(rx_len)
    }

    fn write_data(&mut self, buf: &[u8]) {
        let index = self.index;
        if let Some(buf_out) = self.buf_out.as_mut() {
            buf_out.write(buf);
            btable::write_transmit_buffer_descriptor::<I>(index, buf_out.addr, buf.len() as _);
        }
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, ChannelError> {
        self.write_data(buf);

        let index = self.index;

        let regs = I::regs();

        let epr = regs.epr(index).read();
        let current_stat_tx = epr.stat_tx().to_bits();
        // stat_rx can only be toggled by writing a 1.
        // We want to set it to Active (0b11)
        let stat_valid = Stat::from_bits(!current_stat_tx & 0x3);

        let mut epr = invariant(epr);
        epr.set_stat_tx(stat_valid);
        regs.epr(index).write_value(epr);

        let stat = poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            let regs = I::regs();
            let stat = regs.epr(index).read().stat_rx();
            if matches!(stat, Stat::STALL | Stat::DISABLED) {
                Poll::Ready(stat)
            } else {
                Poll::Pending
            }
        })
        .await;

        if stat == Stat::STALL {
            return Err(ChannelError::Stall);
        }

        Ok(buf.len())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError> {
        let index = self.index;

        let regs = I::regs();
        let epr = regs.epr(index);
        let epr_val = epr.read();
        let current_stat_rx = epr_val.stat_rx().to_bits();
        // stat_rx can only be toggled by writing a 1.
        // We want to set it to Active (0b11)
        let stat_valid = Stat::from_bits(!current_stat_rx & 0x3);

        let mut epr_val = invariant(epr_val);
        epr_val.set_stat_rx(stat_valid);
        regs.epr(index).write_value(epr_val);

        let mut count: usize = 0;

        let res = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());

            let stat = regs.epr(index).read().stat_rx();
            match stat {
                Stat::DISABLED => {
                    // data available
                    let idest = &mut buf[count..];
                    let n = self.read_data(idest)?;
                    count += n;
                    // If transfer is smaller than max_packet_size, we are done
                    // If we have read buf.len() bytes, we are done
                    if count == buf.len() || n < self.max_packet_size_in as usize {
                        Poll::Ready(Ok(count))
                    } else {
                        // issue another read
                        let mut epr_val = invariant(epr.read());
                        epr_val.set_stat_rx(Stat::VALID);
                        epr.write_value(epr_val);
                        Poll::Pending
                    }
                }
                Stat::STALL => {
                    // error
                    Poll::Ready(Err(ChannelError::Stall))
                }
                Stat::NAK => {
                    // pending
                    Poll::Pending
                }
                Stat::VALID => {
                    // not started yet?
                    Poll::Pending
                }
            }
        })
        .await;

        res
    }
}

// impl<'d, I: Instance> Channel<'d, D, In> {
// }

impl<'d, I: Instance, T: channel::Type, D: channel::Direction> UsbChannel<T, D> for Channel<'d, I, D, T> {
    async fn control_in(
        &mut self,
        setup: &embassy_usb_driver::host::SetupPacket,
        buf: &mut [u8],
    ) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsIn,
    {
        let epr0 = I::regs().epr(0);

        // setup stage
        let mut epr_val = invariant(epr0.read());
        epr_val.set_setup(true);
        epr0.write_value(epr_val);

        self.write(setup.as_bytes()).await?;

        // data stage
        let count = self.read(buf).await?;

        // status stage

        // Send 0 bytes
        let zero: [u8; 0] = [0u8; 0];
        self.write(&zero).await?;

        Ok(count)
    }

    async fn control_out(
        &mut self,
        setup: &embassy_usb_driver::host::SetupPacket,
        buf: &[u8],
    ) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsOut,
    {
        let epr0 = I::regs().epr(0);

        // setup stage
        let mut epr_val = invariant(epr0.read());
        epr_val.set_setup(true);
        epr0.write_value(epr_val);
        self.write(setup.as_bytes()).await?;

        if buf.is_empty() {
            // TODO data stage
            todo!("implement data stage");
        } else {
            self.write(buf).await?;
        }

        // Status stage
        let mut status = [0u8; 0];
        self.read(&mut status).await?;

        Ok(buf.len())
    }

    fn retarget_channel(
        &mut self,
        addr: u8,
        endpoint: &embassy_usb_driver::EndpointInfo,
        pre: bool,
    ) -> Result<(), embassy_usb_driver::host::HostError> {
        let eptype = endpoint.ep_type;
        let index = self.index;

        // configure channel register
        let epr_reg = I::regs().epr(index);
        let mut epr = invariant(epr_reg.read());
        epr.set_devaddr(addr);
        epr.set_ep_type(convert_type(eptype));
        epr.set_ea(index as _);
        epr_reg.write_value(epr);

        Ok(())
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsIn,
    {
        self.read(buf).await
    }

    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsOut,
    {
        self.write(buf).await
    }
}

impl<'d, I: Instance> UsbHostDriver for USBHostDriver<'d, I> {
    type Channel<T: channel::Type, D: channel::Direction> = Channel<'d, I, D, T>;

    fn alloc_channel<T: channel::Type, D: channel::Direction>(
        &self,
        addr: u8,
        endpoint: &embassy_usb_driver::EndpointInfo,
        pre: bool,
    ) -> Result<Self::Channel<T, D>, embassy_usb_driver::host::HostError> {
        let new_index = if T::ep_type() == EndpointType::Control {
            // Only a single control channel is available
            0
        } else {
            loop {
                let pipes = ALLOCATED_PIPES.load(Ordering::Relaxed);

                // Ignore index 0
                let new_index = (pipes | 1).trailing_ones();
                if new_index as usize >= USB_MAX_PIPES {
                    Err(HostError::OutOfChannels)?;
                }

                ALLOCATED_PIPES.store(pipes | 1 << new_index, Ordering::Relaxed);

                // TODO make this thread safe using atomics or critical section?
                // cortex m0 does not have compare_exchange_weak, only load and store
                // if ALLOCATED_PIPES
                //     .compare_exchange_weak(
                //         pipes,
                //         pipes | 1 << new_index,
                //         core::sync::atomic::Ordering::Acquire,
                //         core::sync::atomic::Ordering::Relaxed,
                //     )
                //     .is_ok()
                // {
                //     break new_index;
                // }
                break new_index;
            }
        };

        let max_packet_size = endpoint.max_packet_size;

        let buffer_in = if D::is_in() {
            let (len, len_bits) = calc_receive_len_bits(max_packet_size);
            let Ok(buffer_addr) = self.alloc_channel_mem(len) else {
                return Err(HostError::OutOfSlots);
            };

            btable::write_receive_buffer_descriptor::<I>(new_index as usize, buffer_addr, len_bits);

            Some(EndpointBuffer::new(buffer_addr, len))
        } else {
            None
        };

        let buffer_out = if D::is_out() {
            let len = align_len_up(max_packet_size);
            let Ok(buffer_addr) = self.alloc_channel_mem(len) else {
                return Err(HostError::OutOfSlots);
            };

            // ep_in_len is written when actually TXing packets.
            btable::write_in::<I>(new_index as usize, buffer_addr);

            Some(EndpointBuffer::new(buffer_addr, len))
        } else {
            None
        };

        let mut channel = Channel::<I, D, T>::new(
            new_index as usize,
            buffer_in,
            buffer_out,
            endpoint.max_packet_size,
            endpoint.max_packet_size,
        );

        channel.retarget_channel(addr, endpoint, pre)?;
        Ok(channel)
    }

    // fn alloc_channel_in(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelIn, ()> {
    //     let index = (desc.endpoint_address - 0x80) as usize;

    //     if index == 0 {
    //         return Err(());
    //     }
    //     if index > EP_COUNT - 1 {
    //         return Err(());
    //     }
    //     let max_packet_size = desc.max_packet_size;
    //     let ep_type = desc.ep_type();
    //     debug!(
    //         "alloc_channel_in: index = {}, max_packet_size = {}, type = {:?}",
    //         index, max_packet_size, ep_type
    //     );

    //     // read current device address from channel 0
    //     let epr_reg = I::regs().epr(0);
    //     let addr = epr_reg.read().devaddr();

    //     self.claim_channel_in(index, max_packet_size, convert_type(ep_type), addr)
    // }

    // fn alloc_channel_out(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelOut, ()> {
    //     let index = desc.endpoint_address as usize;
    //     if index == 0 {
    //         return Err(());
    //     }
    //     if index > EP_COUNT - 1 {
    //         return Err(());
    //     }
    //     let max_packet_size = desc.max_packet_size;
    //     let ep_type = desc.ep_type();

    //     // read current device address from channel 0
    //     let epr_reg = I::regs().epr(0);
    //     let addr = epr_reg.read().devaddr();

    //     self.claim_channel_out(index, max_packet_size, convert_type(ep_type), addr)
    // }

    // fn reconfigure_channel0(&mut self, max_packet_size: u16, dev_addr: u8) -> Result<(), ()> {
    //     // Clear all buffer memory
    //     self.reset_alloc();

    //     self.control_channel_in = self.claim_channel_in(0, max_packet_size, EpType::CONTROL, dev_addr)?;
    //     self.control_channel_out = self.claim_channel_out(0, max_packet_size, EpType::CONTROL, dev_addr)?;

    //     Ok(())
    // }

    async fn bus_reset(&self) {
        let regs = I::regs();

        trace!("Bus reset");
        // Set bus in reset state
        regs.cntr().modify(|w| {
            w.set_fres(true);
        });

        // USB Spec says wait 50ms
        Timer::after_millis(50).await;

        // Clear reset state; device will be in default state
        regs.cntr().modify(|w| {
            w.set_fres(false);
        });
    }

    // async fn wait_for_device_connect(&mut self) {
    //     poll_fn(|cx| {
    //         let istr = I::regs().istr().read();

    //         BUS_WAKER.register(cx.waker());

    //         if istr.dcon_stat() {
    //             // device has been detected
    //             Poll::Ready(())
    //         } else {
    //             Poll::Pending
    //         }
    //     })
    //     .await;
    // }

    // async fn wait_for_device_disconnect(&mut self) {
    //     poll_fn(|cx| {
    //         let istr = I::regs().istr().read();

    //         BUS_WAKER.register(cx.waker());

    //         if !istr.dcon_stat() {
    //             // device has dosconnected
    //             Poll::Ready(())
    //         } else {
    //             Poll::Pending
    //         }
    //     })
    //     .await;
    // }

    async fn wait_for_device_event(&self) -> embassy_usb_driver::host::DeviceEvent {
        todo!()
    }
}
