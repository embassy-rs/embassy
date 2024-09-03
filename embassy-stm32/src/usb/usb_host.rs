#![macro_use]
#![allow(missing_docs)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Timer;
use embassy_usb_driver::host::{ChannelIn, ChannelOut, EndpointDescriptor, USBHostDriverTrait};
use embassy_usb_driver::{Direction, EndpointError, EndpointType};

use crate::pac::usb::regs;
use crate::pac::usb::vals::{EpType, Stat};
use crate::pac::USBRAM;
use crate::rcc::RccPeripheral;
use crate::{interrupt, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let x = regs.istr().read().0;
        trace!("USB IRQ: {:08x}", x);

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

            // debug!("INT EP {}", index);
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

    pub(super) fn write_in<T: Instance>(_index: usize, _addr: u16) {}

    // // Write to Transmit Buffer Descriptor for Channel/endpoint (USB_CHEP_TXRXBD_n)
    // // Device: IN endpoint
    // // Host: Out endpoint
    // // Address offset: n*8 [bytes] or n*2 in 32 bit words
    // pub(super) fn write_in_len<T: Instance>(index: usize, addr: u16, len: u16) {
    //     USBRAM.mem(index * 2).write_value((addr as u32) | ((len as u32) << 16));
    // }

    // TODO: Replaces write_in_len
    /// Writes to Transmit Buffer Descriptor for Channel/endpoint `index``
    /// For Device this is an IN endpoint for Host an OUT endpoint
    pub(super) fn write_transmit_buffer_descriptor<T: Instance>(index: usize, addr: u16, len: u16) {
        // Address offset: index*8 [bytes] thus index*2 in 32 bit words
        USBRAM.mem(index * 2).write_value((addr as u32) | ((len as u32) << 16));
    }

    // Replaces write_out
    /// Writes to Receive Buffer Descriptor for Channel/endpoint `index``
    /// For Device this is an OUT endpoint for Host an IN endpoint
    pub(super) fn write_receive_buffer_descriptor<T: Instance>(index: usize, addr: u16, max_len_bits: u16) {
        // Address offset: index*8 + 4 [bytes] thus index*2 + 1 in 32 bit words
        USBRAM
            .mem(index * 2 + 1)
            .write_value((addr as u32) | ((max_len_bits as u32) << 16));
    }

    pub(super) fn update_rx_packet_size<T: Instance>(index: usize, max_len_bits: u16) {
        // Address offset: index*8 + 4 [bytes] thus index*2 + 1 in 32 bit words
        let reg = USBRAM.mem(index * 2 + 1);
        let curr_val = reg.read();
        let new_value = (curr_val & 0x0000FFF) | ((max_len_bits as u32) << 16);
        reg.write_value(new_value);
    }

    pub(super) fn read_out_len<T: Instance>(index: usize) -> u16 {
        (USBRAM.mem(index * 2 + 1).read() >> 16) as u16
    }
}

// Maybe replace with struct that only knows its index
struct EndpointBuffer<T: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<T>,
}

impl<T: Instance> EndpointBuffer<T> {
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

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct EndpointData {
    ep_type: EndpointType, // only valid if used_in || used_out
    used_in: bool,
    used_out: bool,
}

/// USB host driver.
pub struct USBHostDriver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_mem_free: u16, // first free address in EP mem, in bytes.
    control_channel_in: Channel<'d, T, In>,
    control_channel_out: Channel<'d, T, Out>,
    channels_in_used: u8,
    channels_out_used: u8,
}

impl<'d, T: Instance> USBHostDriver<'d, T> {
    /// Create a new USB driver.
    pub fn new(
        _usb: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dp: impl Peripheral<P = impl DpPin<T>> + 'd,
        dm: impl Peripheral<P = impl DmPin<T>> + 'd,
    ) -> Self {
        into_ref!(dp, dm);

        super::common_init::<T>();

        let regs = T::regs();

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

        Self {
            phantom: PhantomData,
            ep_mem_free: EP_COUNT as u16 * 8, // for each EP, 4 regs, so 8 bytes
            control_channel_in: Channel::new(0, 0, 0, 0, EpType::CONTROL),
            control_channel_out: Channel::new(0, 0, 0, 0, EpType::CONTROL),
            channels_in_used: 0,
            channels_out_used: 0,
        }
    }

    /// Start the USB peripheral
    pub fn start(&mut self) {
        let _ = self.reconfigure_channel0(8, 0);

        let regs = T::regs();

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
        let regs = T::regs();

        let istr = regs.istr().read();

        istr.0
    }

    fn reset_alloc(&mut self) {
        // Reset alloc pointer.
        self.ep_mem_free = EP_COUNT as u16 * 8; // for each EP, 4 regs, so 8 bytes

        self.channels_in_used = 0;
        self.channels_out_used = 0;
    }

    fn alloc_channel_mem(&mut self, len: u16) -> Result<u16, ()> {
        assert!(len as usize % USBRAM_ALIGN == 0);
        let addr = self.ep_mem_free;
        if addr + len > USBRAM_SIZE as _ {
            // panic!("Endpoint memory full");
            error!("Endpoint memory full");
            return Err(());
        }
        self.ep_mem_free += len;
        Ok(addr)
    }

    fn claim_channel_in(
        &mut self,
        index: usize,
        max_packet_size: u16,
        ep_type: EpType,
        dev_addr: u8,
    ) -> Result<Channel<'d, T, In>, ()> {
        if self.channels_in_used & (1 << index) != 0 {
            error!("Channel {} In already in use", index);
            return Err(());
        }

        self.channels_in_used |= 1 << index;

        let (len, len_bits) = calc_receive_len_bits(max_packet_size);
        let Ok(addr) = self.alloc_channel_mem(len) else {
            return Err(());
        };

        btable::write_receive_buffer_descriptor::<T>(index, addr, len_bits);

        let in_channel: Channel<T, In> = Channel::new(index, addr, len, max_packet_size, ep_type);

        // configure channel register
        let epr_reg = T::regs().epr(index);
        let mut epr = invariant(epr_reg.read());
        epr.set_devaddr(dev_addr);
        epr.set_ep_type(ep_type);
        epr.set_ea(index as _);
        epr_reg.write_value(epr);

        Ok(in_channel)
    }

    fn claim_channel_out(
        &mut self,
        index: usize,
        max_packet_size: u16,
        ep_type: EpType,
        dev_addr: u8,
    ) -> Result<Channel<'d, T, Out>, ()> {
        if self.channels_out_used & (1 << index) != 0 {
            error!("Channel {} In already in use", index);
            return Err(());
        }
        self.channels_out_used |= 1 << index;

        let len = align_len_up(max_packet_size);
        let Ok(addr) = self.alloc_channel_mem(len) else {
            return Err(());
        };

        // ep_in_len is written when actually TXing packets.
        btable::write_in::<T>(index, addr);

        let out_channel: Channel<T, Out> = Channel::new(index, addr, len, max_packet_size, ep_type);

        // configure channel register
        let epr_reg = T::regs().epr(index);
        let mut epr = invariant(epr_reg.read());
        epr.set_devaddr(dev_addr);
        epr.set_ep_type(ep_type);
        epr.set_ea(index as _);
        epr_reg.write_value(epr);

        Ok(out_channel)
    }
}

trait Dir {
    fn dir() -> Direction;
}

/// Marker type for the "IN" direction.
pub enum In {}
impl Dir for In {
    fn dir() -> Direction {
        Direction::In
    }
}

/// Marker type for the "OUT" direction.
pub enum Out {}
impl Dir for Out {
    fn dir() -> Direction {
        Direction::Out
    }
}

/// USB endpoint.
pub struct Channel<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    index: usize,
    max_packet_size: u16,
    ep_type: EpType,
    buf: EndpointBuffer<T>,
}

impl<'d, T: Instance, D> Channel<'d, T, D> {
    fn new(index: usize, addr: u16, len: u16, max_packet_size: u16, ep_type: EpType) -> Self {
        Self {
            _phantom: PhantomData,
            index,
            max_packet_size,
            ep_type,
            buf: EndpointBuffer {
                addr,
                len,
                _phantom: PhantomData,
            },
        }
    }
}

impl<'d, T: Instance> Channel<'d, T, In> {
    fn read_data(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.index;
        let rx_len = btable::read_out_len::<T>(index) as usize & 0x3FF;
        trace!("READ DONE, rx_len = {}", rx_len);
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        self.buf.read(&mut buf[..rx_len]);
        Ok(rx_len)
    }
}

impl<'d, T: Instance> ChannelIn for Channel<'d, T, In> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.index;

        let regs = T::regs();
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
                    if count == buf.len() || n < self.max_packet_size as usize {
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
                    Poll::Ready(Err(EndpointError::Disabled))
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

impl<'d, T: Instance> Channel<'d, T, Out> {
    fn write_data(&mut self, buf: &[u8]) {
        let index = self.index;
        self.buf.write(buf);
        btable::write_transmit_buffer_descriptor::<T>(index, self.buf.addr, buf.len() as _);
    }
}

impl<'d, T: Instance> ChannelOut for Channel<'d, T, Out> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.write_data(buf);

        let index = self.index;

        let regs = T::regs();

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
            let regs = T::regs();
            let stat = regs.epr(index).read().stat_rx();
            if matches!(stat, Stat::STALL | Stat::DISABLED) {
                Poll::Ready(stat)
            } else {
                Poll::Pending
            }
        })
        .await;

        if stat == Stat::STALL {
            // TODO better error
            return Err(EndpointError::Disabled);
        }

        Ok(())
    }
}

impl<'d, T: Instance> USBHostDriverTrait for USBHostDriver<'d, T> {
    type ChannelIn = Channel<'d, T, In>;
    type ChannelOut = Channel<'d, T, Out>;

    fn alloc_channel_in(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelIn, ()> {
        let index = (desc.endpoint_address - 0x80) as usize;

        if index == 0 {
            return Err(());
        }
        if index > EP_COUNT - 1 {
            return Err(());
        }
        let max_packet_size = desc.max_packet_size;
        let ep_type = desc.ep_type();
        debug!(
            "alloc_channel_in: index = {}, max_packet_size = {}, type = {:?}",
            index, max_packet_size, ep_type
        );

        // read current device address from channel 0
        let epr_reg = T::regs().epr(0);
        let addr = epr_reg.read().devaddr();

        self.claim_channel_in(index, max_packet_size, convert_type(ep_type), addr)
    }

    fn alloc_channel_out(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelOut, ()> {
        let index = desc.endpoint_address as usize;
        if index == 0 {
            return Err(());
        }
        if index > EP_COUNT - 1 {
            return Err(());
        }
        let max_packet_size = desc.max_packet_size;
        let ep_type = desc.ep_type();

        // read current device address from channel 0
        let epr_reg = T::regs().epr(0);
        let addr = epr_reg.read().devaddr();

        self.claim_channel_out(index, max_packet_size, convert_type(ep_type), addr)
    }

    fn reconfigure_channel0(&mut self, max_packet_size: u16, dev_addr: u8) -> Result<(), ()> {
        // Clear all buffer memory
        self.reset_alloc();

        self.control_channel_in = self.claim_channel_in(0, max_packet_size, EpType::CONTROL, dev_addr)?;
        self.control_channel_out = self.claim_channel_out(0, max_packet_size, EpType::CONTROL, dev_addr)?;

        Ok(())
    }

    async fn bus_reset(&mut self) {
        let regs = T::regs();

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

    async fn wait_for_device_connect(&mut self) {
        poll_fn(|cx| {
            let istr = T::regs().istr().read();

            BUS_WAKER.register(cx.waker());

            if istr.dcon_stat() {
                // device has been detected
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    async fn wait_for_device_disconnect(&mut self) {
        poll_fn(|cx| {
            let istr = T::regs().istr().read();

            BUS_WAKER.register(cx.waker());

            if !istr.dcon_stat() {
                // device has dosconnected
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    async fn control_request_out(&mut self, bytes: &[u8]) -> Result<(), ()> {
        let epr0 = T::regs().epr(0);

        // setup stage
        let mut epr_val = invariant(epr0.read());
        epr_val.set_setup(true);
        epr0.write_value(epr_val);
        self.control_channel_out.write(bytes).await.map_err(|_| ())?;

        // TODO data stage
        // self.control_channel_out.write(bytes).await.map_err(|_| ())?;

        // Status stage
        let mut status = [0u8; 0];
        self.control_channel_in.read(&mut status).await.map_err(|_| ())?;

        Ok(())
    }

    async fn control_request_in(&mut self, bytes: &[u8], dest: &mut [u8]) -> Result<usize, ()> {
        let epr0 = T::regs().epr(0);

        // setup stage
        let mut epr_val = invariant(epr0.read());
        epr_val.set_setup(true);
        epr0.write_value(epr_val);

        self.control_channel_out.write(bytes).await.map_err(|_| ())?;

        // data stage

        let count = self.control_channel_in.read(dest).await.map_err(|_| ())?;

        // status stage

        // Send 0 bytes
        let zero = [0u8; 0];
        self.control_channel_out.write(&zero).await.map_err(|_| ())?;

        Ok(count)
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::usb::Usb;
}

/// USB instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// Interrupt for this USB instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Internal PHY pins
pin_trait!(DpPin, Instance);
pin_trait!(DmPin, Instance);

foreach_interrupt!(
    ($inst:ident, usb, $block:ident, LP, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::usb::Usb {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
