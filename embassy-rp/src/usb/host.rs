
use core::{future::{poll_fn, Future}, marker::PhantomData, sync::atomic::{AtomicU16, AtomicUsize, Ordering}, task::Poll};

use embassy_hal_internal::Peripheral;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::host::{ChannelError, ChannelIn, ChannelOut, EndpointDescriptor, USBHostDriverTrait};
use embassy_usb_driver::{Direction, EndpointType};

use rp_pac::usb_dpram::vals::EpControlEndpointType;
use usbh::{bus::{AsyncHostBus, Error, Event, HostBus, InterruptPipe}, types::{ConnectionSpeed, TransferType}};
use crate::{interrupt::{self, typelevel::{Binding, Interrupt}}, usb::EP_MEMORY_SIZE};
use crate::RegExt;
use usb_device::UsbDirection;

use super::{Dir, EndpointBuffer, In, Instance, Out, SealedInstance, BUS_WAKER, DPRAM_DATA_OFFSET, EP_IN_WAKERS, EP_MEMORY, EP_OUT_WAKERS};

const CONTROL_BUFFER_SIZE: usize = 64;

const MAIN_BUFFER_SIZE: usize = 1024;

/// Accesses should be atomic
static mut PENDING_EVENT: Option<Event> = None;
/// FIXME: Critical sections?

/// Current channel with ongoing transfer
/// 
/// 0 means None 
static CURRENT_CHANNEL: AtomicUsize = AtomicUsize::new(0); 

/// RP2040 USB host driver handle.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    /// Bitset of allocated interrupt pipes
    allocated_pipes: AtomicU16,
    control_in: Channel<'d, T, In>,
    control_out: Channel<'d, T, Out>,
}

impl<'d, T: Instance> Driver<'d, T> {    
    /// Create a new USB driver.
    pub fn new(
        _usb: impl Peripheral<P = T> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>
    ) -> Self {
        let regs = T::regs();
        unsafe {
            // FIXME(magic):
            // zero fill regs
            let p = regs.as_ptr() as *mut u32;
            for i in 0..0x9c / 4 {
                p.add(i).write_volatile(0)
            }

            // zero fill epmem
            let p = EP_MEMORY as *mut u32;
            for i in 0..0x180 / 4 {
                p.add(i).write_volatile(0)
            }
        }

        regs.usb_muxing().modify(|w| {
            w.set_to_phy(true);
            w.set_softcon(true);
        });
        regs.usb_pwr().modify(|w| {
            w.set_vbus_detect(true);
            w.set_vbus_detect_override_en(true);
        });
        regs.main_ctrl().modify(|w| {
            w.set_controller_en(true);
            w.set_host_ndevice(true);
        });
        regs.sie_ctrl().modify(|w| {
            w.set_sof_en(true);
            w.set_keep_alive_en(true);
            w.set_pulldown_en(true);
        });
        
        regs.inte().write(|w| {
            w.set_buff_status(true);
            w.set_host_resume(true);
            w.set_stall(true);
            w.set_error_rx_timeout(true);
            w.set_error_data_seq(true);
            w.set_error_crc(true);
            w.set_error_bit_stuff(true);
            w.set_error_rx_overflow(true);
        });
        
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        
        // Initialize the bus so that it signals that power is available
        BUS_WAKER.wake();

        let desc = EndpointDescriptor {
            len: 64,
            descriptor_type: 0x05,
            endpoint_address: 0,
            attributes: EndpointType::Control as u8,
            max_packet_size: 8,
            interval: 10,
        };
        
        Self {
            phantom: PhantomData,
            allocated_pipes: AtomicU16::new(0),
            control_in: Channel::new(16, DPRAM_DATA_OFFSET, desc, 64, 0),
            control_out: Channel::new(17, DPRAM_DATA_OFFSET, desc, 64, 0), 
        }
    }

    fn claim_interrupt_channel(&self, desc: &EndpointDescriptor, dev_addr: u8) -> Option<Channel<'d, T, In>> {
        let alloc = self.allocated_pipes.load(Ordering::Acquire);
        let free_index = (1..16).find(|i| alloc & (1 << i) == 0)? as u8;
        
        self.allocated_pipes.store(alloc | 1 << free_index, Ordering::Release);
        // Use fixed layout
        let addr = DPRAM_DATA_OFFSET + MAIN_BUFFER_SIZE as u16 + free_index as u16 * 64;

        Some(Channel::new(free_index as _, addr, *desc, 64, dev_addr))
    }

    fn alloc_pipe(&self, size: u16) -> Option<InterruptPipe> {
        if size > 64 {
            return None
        }
        
        let alloc = self.allocated_pipes.load(Ordering::Acquire);
        let free_index = (0..15).find(|i| alloc & (1 << i) == 0)? as u8;

        // for simplicity, all pipes are considered to be 64 bytes long for now.
        // This is the maximum supported size for pipes other than Isochronous, 
        // which are not implemented yet.
       
        // Safety: this is the only place where offsets larger than 0x180+CONTROL_BUFFER_SIZE are used.
        // Since the highest index is 15, all offsets are below `0x180 + 16 * 64 = 1408`, which is below the 4096 bytes available in DPRAM.
        let ptr = unsafe {
            EP_MEMORY.offset(
                DPRAM_DATA_OFFSET as isize + CONTROL_BUFFER_SIZE as isize + free_index as isize * 64
            )
        };

        self.allocated_pipes.store(alloc | 1 << free_index, Ordering::Release);
        
        Some(InterruptPipe {
            bus_ref: free_index,
            ptr
        })
    }

    fn release_pipe(&self, pipe_ref: u8) {
        let alloc = self.allocated_pipes.load(Ordering::Acquire);
        self.allocated_pipes.store(alloc & !(1 << pipe_ref), Ordering::Release);
    }

    fn ints_to_event() -> Option<Event> {
        let regs = T::regs();
        let ints = regs.ints().read();
        
        if ints.host_conn_dis() {
            let event = match regs.sie_status().read().speed() {
                0b01 => Event::Attached(ConnectionSpeed::Low),
                0b10 => Event::Attached(ConnectionSpeed::Full),
                _ => Event::Detached,
            };
            regs.sie_status().modify(|w| {
                // FIXME(magic):
                w.set_speed(0b11); 
            });
            return Some(event);
        }
        if ints.host_resume() {
            regs.sie_status().write_clear(|w| w.set_resume(true));
            return Some(Event::Resume);
        }
        if ints.stall() {
            regs.sie_status().write_clear(|w| w.set_stall_rec(true));
            return Some(Event::Stall);
        }
        if ints.error_crc() {
            regs.sie_status().write_clear(|w| w.set_crc_error(true));
            return Some(Event::Error(Error::Crc));
        }
        if ints.error_bit_stuff() {
            regs.sie_status().write_clear(|w| w.set_bit_stuff_error(true));
            return Some(Event::Error(Error::BitStuffing));
        }
        if ints.error_rx_overflow() {
            regs.sie_status().write_clear(|w| w.set_rx_overflow(true));
            return Some(Event::Error(Error::RxOverflow));
        }
        if ints.error_rx_timeout() {
            regs.sie_status().write_clear(|w| w.set_rx_timeout(true));
            return Some(Event::Error(Error::RxTimeout));
        }
        if ints.error_data_seq() {
            regs.sie_status().write_clear(|w| w.set_data_seq_error(true));
            return Some(Event::Error(Error::DataSequence));
        }
        if ints.buff_status() {
            let status = regs.buff_status().read().0;
            // TODO: handle buffer updates more gracefully. Currently we always wait for TransComplete,
            //   which only works for transfers that fit into a single buffer.

            for i in 0..32 {
                // ith bit set
                if (status >> i) & 1 == 1 {
                    // clear bit 
                    regs.buff_status().write_clear(|w| w.0 = 1 << i );
                    // control transfers (buffer 0)
                    if i != 0 {
                        let idx = (i / 2) - 1;
                        return Some(Event::InterruptPipe(idx));
                    }
                }
            }
        }
        if ints.trans_complete() {
            regs.sie_status().write_clear(|w| w.set_trans_complete(true));
            return Some(Event::TransComplete);
        }
        if ints.host_sof() {
            return Some(Event::Sof);
        }
        None
    }

    fn control_buffer() -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(EP_MEMORY.offset(DPRAM_DATA_OFFSET as _), CONTROL_BUFFER_SIZE)
        }
    }
}

impl<'d, T: Instance> HostBus for Driver<'d, T> {
    fn enable_sof(&mut self) {
        T::regs().sie_ctrl().modify(|w| {
            w.set_sof_en(true);
            w.set_keep_alive_en(true);
            w.set_pulldown_en(true); 
        });
    }

    fn sof_enabled(&self) -> bool {
        let sie = T::regs().sie_ctrl().read();
        sie.sof_en() && sie.keep_alive_en()
    }

    fn set_recipient(
        &mut self,
        dev_addr: Option<usbh::types::DeviceAddress>,
        endpoint: u8,
        transfer_type: TransferType,
    ) {
        let regs = T::regs();
        regs.addr_endp().write(|w| {
            w.set_address(dev_addr.map(u8::from).unwrap_or(0));
            w.set_endpoint(endpoint);
        });

        T::dpram_epx_control().modify(|w| {
            w.set_enable(true);
            w.set_interrupt_per_buff(true);
            // Use control buffer
            w.set_buffer_address(DPRAM_DATA_OFFSET);

            let epty = match transfer_type {
                TransferType::Control => EpControlEndpointType::CONTROL,
                TransferType::Isochronous => EpControlEndpointType::ISOCHRONOUS,
                TransferType::Bulk => EpControlEndpointType::BULK,
                TransferType::Interrupt => EpControlEndpointType::INTERRUPT,
            };

            w.set_endpoint_type(epty);
        });
    }

    fn ls_preamble(&mut self, enabled: bool) {
        T::regs().sie_ctrl().modify(|w| w.set_preamble_en(enabled));
    }

    fn stop_transaction(&mut self) {
        T::regs().sie_ctrl().modify(|w| w.set_stop_trans(true));
    }

    fn write_setup(&mut self, setup: usbh::types::SetupPacket) {
        let dpram = T::dpram();
        dpram.setup_packet_low().write(|w| {
            w.set_bmrequesttype(setup.request_type.into()); 
            w.set_brequest(setup.request.into());
            w.set_wvalue(setup.value);
        });
        dpram.setup_packet_high().write(|w| {
            w.set_windex(setup.index);
            w.set_wlength(setup.length); 
        });
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_receive_data(false);
            w.set_send_setup(true);
            w.set_start_trans(true);
        });
    }

    fn write_data_in(&mut self, length: u16, pid: bool) {
        // FIXME(magic):
        T::dpram().ep_in_buffer_control(0).write(|w| {
            w.set_available(0, true); 
            w.set_pid(0, pid);
            w.set_full(0, false);
            w.set_length(0, length);
            w.set_last(0, true);
            w.set_reset(true);
        });
        
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_send_setup(false);
            w.set_receive_data(true);
            w.set_start_trans(true);
        });
    }

    fn prepare_data_out(&mut self, data: &[u8]) {
        Self::control_buffer()[..data.len()].copy_from_slice(data);

        T::dpram().ep_in_buffer_control(0).write(|w| {
            w.set_available(0, true);
            w.set_pid(0, true);
            w.set_full(0, true);
            w.set_length(0, data.len() as _);
            w.set_last(0, true);
            w.set_reset(true);
        });
    }

    fn write_data_out_prepared(&mut self) {
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_setup(false);
            w.set_receive_data(false);
            w.set_send_data(true);
            w.set_start_trans(true);
        });
    }

    fn poll(&mut self) -> Option<usbh::bus::Event> {
        unsafe { PENDING_EVENT.take() }
    }

    fn received_data(&self, length: usize) -> &[u8] {
        &Self::control_buffer()[..length]
    }

    fn create_interrupt_pipe(
        &mut self,
        device_address: usbh::types::DeviceAddress,
        endpoint_number: u8,
        direction: UsbDirection,
        size: u16,
        interval: u8,
    ) -> Option<InterruptPipe> {
        let pipe = self.alloc_pipe(size)?;
        let idx = pipe.bus_ref as usize;
        
        let regs = T::regs();
        let dpram = T::dpram();

        dpram.ep_in_control(idx).write(|w| {
            w.set_endpoint_type(EpControlEndpointType::INTERRUPT);
            w.set_interrupt_per_buff(true);
            // FIXME: host_poll_interval (bits 16:25)
            let interval = interval as u32 - 1;
            w.0 |= interval << 16;
            // FIXME: Index offset?
            w.set_buffer_address(
                DPRAM_DATA_OFFSET + CONTROL_BUFFER_SIZE as u16 + (idx * CONTROL_BUFFER_SIZE) as u16
            );
            w.set_enable(true);
        });

        regs.sie_ctrl().modify(|w| { w.set_sof_sync(true) });
        
        // FIXME(magic):
        dpram.ep_in_buffer_control(idx + 1).write(|w| {
            w.set_last(0, true);
            w.set_pid(0, false);
            w.set_full(0, false);
            w.set_reset(true);
            w.set_length(0, size);
        });

        // FIXME(delay):
        cortex_m::asm::delay(12);
        
        dpram.ep_in_buffer_control(idx + 1).modify(|w| w.set_available(0, true));        
        regs.addr_endp_x(idx).write(|w| { 
            w.set_address(device_address.into());
            w.set_endpoint(endpoint_number);
            w.set_intep_dir(direction == UsbDirection::Out);
        });

        // FIXME(delay):
        cortex_m::asm::delay(12);
        
        regs.int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() | 1 << idx);
        });

        Some(pipe)
    }

    fn release_interrupt_pipe(&mut self, pipe_ref: u8) {
        assert!(pipe_ref <= 15);
        let dpram = T::dpram();
        let idx = pipe_ref as usize;

        // Disable interrupt polling
        T::regs().int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() & !(1 << idx))
        });

        // FIXME: bits(0)?
        dpram.ep_in_control(idx).write(|_| {});
        dpram.ep_in_buffer_control(idx + 1).write(|_| {});

        T::regs().addr_endp_x(idx).write(|_| {});

        // Mark as released
        self.release_pipe(idx as u8);
    }

    fn pipe_continue(&mut self, pipe_ref: u8) {
        assert!(pipe_ref <= 15);
        let idx = pipe_ref as usize;

        // EP1..=EP15 IN
        // FIXME(index):
        let control = T::dpram().ep_in_buffer_control(idx + 1);
        control.modify(|w| {
            w.set_last(0, true);
            w.set_pid(0, !w.pid(0));
            w.set_full(0, false);
            w.set_reset(true);
        });

        // FIXME(delay):
        cortex_m::asm::delay(12);

        control.modify(|w| w.set_available(0, true))
    }

    fn interrupt_on_sof(&mut self, enable: bool) {
        T::regs().inte().modify(|w| w.set_host_sof(enable));
    }

    fn reset_controller(&mut self) {
        todo!()
    }

    fn reset_bus(&mut self) {
        todo!()
    }
}

/// USB endpoint.
pub struct Channel<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    index: usize,
    buf: EndpointBuffer<T>,
    desc: EndpointDescriptor,
    dev_addr: u8,

    /// DATA0-DATA1 state
    pid: bool,
}

impl<'d, T: Instance, D: Dir> Channel<'d, T, D> {
    /// [EP_MEMORY]-relative address
    fn new(
        index: usize, 
        addr: u16, 
        desc: EndpointDescriptor,
        len: u16,
        dev_addr: u8,
    ) -> Self {
        // TODO: assert only in debug?
        assert!(addr + len <= EP_MEMORY_SIZE as u16);
        assert!(desc.max_packet_size <= EP_MEMORY_SIZE as u16);

        // TODO: Support isochronous, bulk, and interrupt OUT
        assert!(desc.ep_type() != EndpointType::Isochronous);
        assert!(desc.ep_type() != EndpointType::Bulk);
        assert!(!(desc.ep_type() == EndpointType::Interrupt && D::dir() == Direction::Out));
        
        if desc.ep_type() == EndpointType::Interrupt {
            assert!(index > 0 && index < 16);
        } else {
            assert!(index >= 16);
        }
        
        Self {
            _phantom: PhantomData,
            index,
            desc,
            dev_addr,
            buf: EndpointBuffer {
                addr,
                len,
                _phantom: PhantomData,
            },
            pid: false,
        }
    }
}

type BufferControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpBufferControl, rp_pac::common::RW>;
type EpControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpControl, rp_pac::common::RW>;
type AddrControlReg = rp_pac::common::Reg<rp_pac::usb::regs::AddrEndpX, rp_pac::common::RW>;
impl<'d, T: Instance, IO: Dir> Channel<'d, T, IO> {
    /// Get channel waker
    fn waker(&self) -> &AtomicWaker {
        if self.is_interrupt_in() { 
            &EP_IN_WAKERS[self.index]
        } else { 
            &EP_IN_WAKERS[0] 
        }
    }

    /// Get buffer control register
    fn buffer_control(&self) -> BufferControlReg {
        let index = if self.is_interrupt_in() {
            // Validated 1-15
            self.index
        } else {
            0
        };
        T::dpram().ep_in_buffer_control(index)        
    }

    /// Get endpoint control register
    fn ep_control(&self) -> EpControlReg {
        if self.is_interrupt_in() {
            T::dpram().ep_in_control(self.index - 1)        
        } else {
            T::dpram_epx_control()
        }
    }
    
    /// Get interrupt endpoint address control
    fn addr_endp_host(&self) -> AddrControlReg {
        assert!(self.is_interrupt_in());
        T::regs().addr_endp_x(self.index - 1)
    }

    fn is_interrupt_in(&self) -> bool {
        self.desc.ep_type() == EndpointType::Interrupt && IO::dir() == Direction::In
    }
    
    /// Wait for buffer to be available
    /// Returns stall status
    async fn wait_available(&self) -> bool {
        trace!("CHANNEL {} WAIT AVAILABLE", self.index);
        poll_fn(|cx| {
            // Both IN and OUT endpoints use IN registers on rp2040 in host mode
            self.waker().register(cx.waker());

            let reg = self.buffer_control().read();

            // If waiting on current tx, clear interrupts
            if self.is_ready_for_transaction() {
                self.clear_sie_status();
            }
            
            // FIXME: Stall derived from other place
            match reg.available(0) {
                true => Poll::Pending,
                false => Poll::Ready(false),
            }
        }).await
    }

    /// Is hardware configured to perform transaction with this buffer
    /// Always true for INTERRUPT channel
    fn is_ready_for_transaction(&self) -> bool {
        if self.is_interrupt_in() {
            true
        } else {
            let sel = CURRENT_CHANNEL.load(Ordering::Relaxed);
            sel == self.index || sel == 0
        }
    }

    async fn wait_ready_for_transaction(&self) {
        // Wait transfer buffer to be free
        self.wait_available().await;
        
        trace!("CHANNEL {} WAIT READY", self.index);
        // Wait for other transaction end
        poll_fn(|cx| {
           self.waker().register(cx.waker());

            // Other transaction in progress
            if !self.is_ready_for_transaction() {
                return Poll::Pending
            }
            
            Poll::Ready(())
        }).await;
    }
    
    /// Wait transaction to be complete
    async fn wait_trans_complete(&self) {
        trace!("CHANNEL {} WAIT TRANS COMPLETE", self.index);
        let regs = T::regs();
        regs.inte().modify(|w| w.set_trans_complete(true));
        poll_fn(|cx| {
            self.waker().register(cx.waker());

            // Other transaction in progress
            if !regs.sie_status().read().trans_complete() {
                return Poll::Pending
            }
            
            Poll::Ready(())
        }).await;
        regs.sie_status().write_clear(|w| w.set_trans_complete(true));
    }

    /// Mark this channel as currently used and configure endpoint type
    /// 
    /// Call once on creation for interrupt pipe
    fn set_current(&self) {
        let regs = T::regs();
        let dpram = T::dpram();
        if self.is_interrupt_in() {
            trace!("INTERRUPT CHANNEL {} :: {}", self.index, self.desc);
            self.ep_control().write(|w| {
                w.set_endpoint_type(EpControlEndpointType::INTERRUPT);
                w.set_interrupt_per_buff(true);
                 
                // FIXME: host_poll_interval (bits 16:25)
                let interval = self.desc.interval as u32 - 1;
                w.0 |= interval << 16;
                
                w.set_buffer_address(self.buf.addr);
                w.set_enable(true);
            });

            // FIXME: What is this for?
            regs.sie_ctrl().modify(|w| { w.set_sof_sync(true) });
            
            self.addr_endp_host().write(|w| { 
                w.set_address(self.dev_addr);
                w.set_endpoint(self.desc.endpoint_address);
                // FIXME: INTERRUPT OUT?
                w.set_intep_dir(IO::dir() == Direction::Out);
            });
        } else {
            CURRENT_CHANNEL.store(self.index, Ordering::Relaxed);
            
            T::regs().addr_endp().write(|w| {
                w.set_address(self.dev_addr);
                w.set_endpoint(self.desc.endpoint_address);
            });
            
            self.ep_control().modify(|w| {
                w.set_enable(true);
                w.set_interrupt_per_buff(true);
                w.set_buffer_address(self.buf.addr);

                let epty = match self.desc.ep_type() {
                    EndpointType::Control => EpControlEndpointType::CONTROL,
                    EndpointType::Isochronous => EpControlEndpointType::ISOCHRONOUS,
                    EndpointType::Bulk => EpControlEndpointType::BULK,
                    EndpointType::Interrupt => EpControlEndpointType::INTERRUPT,
                };

                w.set_endpoint_type(epty);
            });
        }
    }
    
    /// Clear current active channel
    fn clear_current(&self) {
        if !self.is_interrupt_in() {
            CURRENT_CHANNEL.store(0, Ordering::Relaxed);
        }
    }

    /// Copy setup packet to buffer and set SETUP transaction
    /// 
    /// Set PID = 1 for next transaction
    fn set_setup_packet(&mut self, setup: &[u8]) {
        assert!(self.desc.ep_type() == EndpointType::Control);
        // FIXME: Byteorder
        T::dpram().setup_packet_low().write(|w| {
            w.0 = u32::from_ne_bytes(setup[..4].try_into().unwrap())
        });
        T::dpram().setup_packet_high().write(|w| {
            w.0 = u32::from_ne_bytes(setup[4..8].try_into().unwrap())
        });
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_receive_data(false);
            w.set_send_setup(true);
        });
        
        self.pid = true;
    }

    /// Reload interrupt channel buffer register
    fn interrupt_reload(&mut self) {
        assert!(self.desc.ep_type() == EndpointType::Interrupt);
        let ctrl = self.buffer_control();
        ctrl.write(|w| {
            w.set_last(0, true);
            w.set_pid(0, self.pid);
            w.set_full(0, false);
            w.set_reset(true);
            w.set_length(0, self.desc.max_packet_size);
            w.set_available(0, true);
        });

        self.pid = !self.pid;
        // TODO: SOF?
        // T::regs().sie_ctrl().modify(|w| {
        //     w.set_sof_en(true);
        //     w.set_keep_alive_en(true);
        //     w.set_pulldown_en(true); 
        // });

        // FIXME: delay reason
        cortex_m::asm::delay(12);
        T::regs().int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() | 1 << (self.index - 1));
        });
    }
    
    /// Set DATA IN transaction
    /// 
    /// WARNING: This flips PID
    fn set_data_in(&mut self, len: u16) {
        assert!(self.desc.ep_type() != EndpointType::Interrupt);
        
        self.buffer_control().write(|w| {
            w.set_pid(0, self.pid);
            w.set_full(0, false);
            w.set_length(0, len);
            w.set_last(0, true);
            w.set_reset(true);
            w.set_available(0, true); 
        });
        
        self.pid = !self.pid;
        
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_send_setup(false);
            w.set_receive_data(true);
        });
    }

    /// Set DATA OUT transaction and copy data to buffer
    /// Returns count of copied bytes
    fn set_data_out(&mut self, data: &[u8]) -> usize {        
        assert!(self.desc.ep_type() != EndpointType::Interrupt);

        let chunk = if data.len() > 0 {
           data.chunks(self.desc.max_packet_size as _).next().unwrap() 
        } else {
            &[]
        };
        
        self.buf.write(&chunk);
        
        self.buffer_control().write(|w| {
            w.set_available(0, true);
            w.set_pid(0, self.pid);
            w.set_full(0, true);
            w.set_length(0, chunk.len() as _);
            w.set_last(0, true);
            w.set_reset(true);
        });

        self.pid = !self.pid;
        
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(true);
            w.set_send_setup(false);
            w.set_receive_data(false);
        });

        chunk.len()
    }

    /// Start transaction with pre-configured values
    fn start_transaction(&self) {
        if !self.is_interrupt_in() {            
            // This field should be modified separately after delay
            cortex_m::asm::delay(12);
            T::regs().sie_ctrl().modify(|w| {
                w.set_start_trans(true);
            });
        }
    }

    /// Clear buffer interrupt bit
    fn clear_sie_status(&self) {
        if self.is_interrupt_in() {
            T::regs().buff_status().write_clear(|w| w.0 = 0b11 << self.index * 2);
        } else {
            T::regs().buff_status().write_clear(|w| w.0 = 0b11);
        }
    }

    /// Send SETUP packet
    /// 
    /// WARNING: This flips PID
    async fn send_setup(&mut self, setup: &[u8]) {
        // Wait transfer buffer to be free
        self.wait_ready_for_transaction().await;
        
        // Set this channel for transaction
        self.set_current();
        
        trace!("SEND SETUP");
        // Prepare HW
        self.set_setup_packet(setup);
        self.start_transaction();
        
        // Wait for SETUP end
        self.wait_trans_complete().await;

        self.clear_current();
    }

    /// Send status packet
    async fn control_status(&mut self) {
        // Wait transfer buffer to be free
        self.wait_ready_for_transaction().await;
        
        // Set this channel for transaction
        self.set_current();
        
        // Status packet always have DATA1
        self.pid = true;
        if IO::dir() == Direction::Out {
            self.set_data_in(0);
        } else {
            self.set_data_out(&[]);
        }
        
        self.start_transaction();
        self.wait_trans_complete().await;

        self.clear_current();
    }
}

impl<'d, T: Instance> ChannelIn for Channel<'d, T, In> {
    /// CONTROL: Data stage
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError> {
        // Wait transfer buffer to be free
        self.wait_ready_for_transaction().await;
        
        // Set this channel for transaction
        self.set_current();
        
        let mut count: usize = 0;

        // FIXME: Errors
        loop {
            if self.is_interrupt_in() {
                trace!("CHANNEL {} WAIT FOR INTERRUPT", self.index);
                self.interrupt_reload();
                self.wait_available().await;
            } else {
                trace!("CHANNEL {} START READ, len = {}", self.index, buf.len());
                self.set_data_in(buf[count..].len() as _,);
                self.start_transaction();
                self.wait_trans_complete().await;
            }
            
            let free = &mut buf[count..];
            let rx_len = self.buffer_control().read().length(0) as usize;
            trace!("CHANNEL {} READ DONE, rx_len = {}", self.index, rx_len);

            if rx_len > free.len() {
                return Err(ChannelError::BufferOverflow);
            }
            
            self.buf.read(&mut free[..rx_len]);
            count += rx_len;

            // If transfer is smaller than max_packet_size, we are done
            // If we have read buf.len() bytes, we are done
            if count == buf.len() || rx_len < self.desc.max_packet_size as usize {
                break;
            }
        }
        
        self.clear_current();
        
        Ok(count)
    }
}

impl<'d, T: Instance> ChannelOut for Channel<'d, T, Out> {
    /// CONTROL: Data stage
    async fn write(&mut self, buf: &[u8]) -> Result<(), ChannelError> {
        // Wait transfer buffer to be free
        self.wait_ready_for_transaction().await;
        
        let regs = T::regs();
        
        // Set this channel for transaction
        self.set_current();

        let mut count = 0;

        // FIXME: Errors
        loop {
            trace!("CHANNEL {} START WRITE", self.index);
            let packet = self.set_data_out(buf);
            self.start_transaction();
            self.wait_available().await;
            trace!("WRITE DONE, tx_len = {}", packet);

            count += packet;
            
            if count == buf.len() {
                break;
            }
        }

        self.clear_current();
               
        Ok(())
    }
}

impl<'d, T: Instance> USBHostDriverTrait for Driver<'d, T> {
    type ChannelIn = Channel<'d, T, In>;
    type ChannelOut = Channel<'d, T, Out>;

    /// FIXME(async): Await
    async fn bus_reset(&mut self) {
        T::regs().sie_ctrl().modify(|w| {
            w.set_reset_bus(true);
        });

        embassy_time::Timer::after_millis(50).await;
    }

    async fn wait_for_device_connect(&mut self) {
        // Enable conn/dis irq
        T::regs().inte().modify(|w| { w.set_host_conn_dis(true); });
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            
            match T::regs().sie_status().read().speed() {
                0b01 | 0b10 => Poll::Ready(()),
                _ => Poll::Pending
            }        
        }).await;
        T::regs().sie_status().write_clear(|w| { w.set_speed(0b11); });
    }

    async fn wait_for_device_disconnect(&mut self) {
        // Enable conn/dis irq
        T::regs().inte().modify(|w| { w.set_host_conn_dis(true); });
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            
            match T::regs().sie_status().read().speed() {
                0b01 | 0b10 => Poll::Pending,
                _ => Poll::Ready(())
            }        
        }).await;
        T::regs().sie_status().write_clear(|w| { w.set_speed(0b11); });
    }

    async fn control_request_out(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.control_out.send_setup(bytes).await;
        // TODO: Data stage

        self.control_out.control_status().await;

        Ok(())
    }

    async fn control_request_in(&mut self, bytes: &[u8], dest: &mut [u8]) -> Result<usize, ()> {
        self.control_in.send_setup(bytes).await;

        let read = self.control_in.read(dest).await.map_err(|_| ())?;

        self.control_in.control_status().await;

        Ok(read)
    }

    fn reconfigure_channel0(&mut self, max_packet_size: u16, dev_addr: u8) -> Result<(), ()> {
        self.control_in.dev_addr = dev_addr;
        self.control_in.desc.max_packet_size = max_packet_size;
        self.control_out.dev_addr = dev_addr;
        self.control_out.desc.max_packet_size = max_packet_size;
        Ok(())
    }

    fn alloc_channel_in(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelIn, ()> {
        // FIXME: dev addr
        self.claim_interrupt_channel(desc, 1).ok_or(())
    }

    fn alloc_channel_out(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelOut, ()> {
        todo!()
    }
}

/// USB interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _usb: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let ints = regs.ints().read();
        
        let ev = {    
            if ints.host_conn_dis() {
                regs.inte().write_clear(|w| w.set_host_conn_dis(true));
                match regs.sie_status().read().speed() {
                    0b01 => "attached low speed",
                    0b10 => "attached full speed",
                    _ => "detached",
                }
            }
            else if ints.host_resume() {
                regs.sie_status().write_clear(|w| w.set_resume(true));
                "resume"
            }
            else if ints.stall() {
                regs.sie_status().write_clear(|w| w.set_stall_rec(true));
                "stall"
            }
            else if ints.error_crc() {
                regs.sie_status().write_clear(|w| w.set_crc_error(true));
                "crc error"
            }
            else if ints.error_bit_stuff() {
                regs.sie_status().write_clear(|w| w.set_bit_stuff_error(true));
                "bit stuff error"
            }
            else if ints.error_rx_overflow() {
                regs.sie_status().write_clear(|w| w.set_rx_overflow(true));
                "rx overflow"
            }
            else if ints.error_rx_timeout() {
                regs.sie_status().write_clear(|w| w.set_rx_timeout(true));
                "rx timeout"
            }
            else if ints.error_data_seq() {
                regs.sie_status().write_clear(|w| w.set_data_seq_error(true));
                "data sequence error"
            }
            else if ints.buff_status() {
                let status = regs.buff_status().read().0;
                for i in 0..32 {
                    // ith bit set
                    if (status >> i) & 1 == 1 {
                        regs.buff_status().write_clear(|w| w.0 = 1 << i );
                        // control transfers (buffer 0)
                        if i != 0 {
                            let idx = i / 2;
                            // T::regs().int_ep_ctrl().modify(|w| {
                            //     w.set_int_ep_active(w.int_ep_active() | 1 << idx);
                            // });
                            trace!("USB IRQ: Interrupt EP {}", idx);
                            EP_IN_WAKERS[idx].wake();
                        } else {
                            trace!("USB IRQ: EPx");
                            EP_IN_WAKERS[0].wake();
                        }
                        break
                    }
                }
                "^^^"
            }
            else if ints.trans_complete() {
                regs.inte().write_clear(|w| w.set_trans_complete(true));
                EP_IN_WAKERS[0].wake();
                "transaction complete"
            }
            else if ints.host_sof() {
                // Prevent nonstop SOF interrupt
                T::regs().inte().write_clear(|w| w.set_host_sof(true));
                "sof"
            } else {
                "???"
            }
        };
        
        trace!("USB IRQ: {:08x} :: {}", ints.0, ev);
        
        BUS_WAKER.wake();
    }
}
