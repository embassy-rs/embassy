
use core::{future::poll_fn, marker::PhantomData, task::Poll};

use atomic_polyfill::{AtomicU16, AtomicUsize, Ordering};
use embassy_hal_internal::Peripheral;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::host::{channel, ChannelError, ChannelIn, ChannelOut, DeviceEvent, EndpointDescriptor, HostError, SetupPacket, UsbChannel, UsbHostDriver};
use embassy_usb_driver::EndpointType;

use rp_pac::usb_dpram::vals::EpControlEndpointType;
use crate::{interrupt::{self, typelevel::{Binding, Interrupt}}, usb::EP_MEMORY_SIZE};
use crate::RegExt;

use super::{EndpointBuffer, Instance, SealedInstance, BUS_WAKER, DPRAM_DATA_OFFSET, EP_IN_WAKERS, EP_MEMORY};

const MAIN_BUFFER_SIZE: usize = 1024;

/// Current channel with ongoing transfer
/// 
/// 0 means None 
static CURRENT_CHANNEL: AtomicUsize = AtomicUsize::new(0); 

/// RP2040 USB host driver handle.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    /// Bitset of allocated interrupt pipes
    allocated_pipes: AtomicU16,
    /// Index for next 'allocated' channel
    channel_index: AtomicUsize,
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
            // 1-15 are reserved for interrupt EPs
            channel_index: AtomicUsize::new(16),
        }
    }
}

// impl<'d, T: Instance> HostBus for Driver<'d, T> {
//     fn set_recipient(
//         &mut self,
//         dev_addr: Option<usbh::types::DeviceAddress>,
//         endpoint: u8,
//         transfer_type: TransferType,
//     ) {
//         let regs = T::regs();
//         regs.addr_endp().write(|w| {
//             w.set_address(dev_addr.map(u8::from).unwrap_or(0));
//             w.set_endpoint(endpoint);
//         });

//         T::dpram_epx_control().modify(|w| {
//             w.set_enable(true);
//             w.set_interrupt_per_buff(true);
//             // Use control buffer
//             w.set_buffer_address(DPRAM_DATA_OFFSET);

//             let epty = match transfer_type {
//                 TransferType::Control => EpControlEndpointType::CONTROL,
//                 TransferType::Isochronous => EpControlEndpointType::ISOCHRONOUS,
//                 TransferType::Bulk => EpControlEndpointType::BULK,
//                 TransferType::Interrupt => EpControlEndpointType::INTERRUPT,
//             };

//             w.set_endpoint_type(epty);
//         });
//     }

//     fn ls_preamble(&mut self, enabled: bool) {
//         T::regs().sie_ctrl().modify(|w| w.set_preamble_en(enabled));
//     }

//     fn stop_transaction(&mut self) {
//         T::regs().sie_ctrl().modify(|w| w.set_stop_trans(true));
//     }

//     fn write_setup(&mut self, setup: usbh::types::SetupPacket) {
//         let dpram = T::dpram();
//         dpram.setup_packet_low().write(|w| {
//             w.set_bmrequesttype(setup.request_type.into()); 
//             w.set_brequest(setup.request.into());
//             w.set_wvalue(setup.value);
//         });
//         dpram.setup_packet_high().write(|w| {
//             w.set_windex(setup.index);
//             w.set_wlength(setup.length); 
//         });
//         T::regs().sie_ctrl().modify(|w| {
//             w.set_send_data(false);
//             w.set_receive_data(false);
//             w.set_send_setup(true);
//             w.set_start_trans(true);
//         });
//     }

//     fn create_interrupt_pipe(
//         &mut self,
//         device_address: usbh::types::DeviceAddress,
//         endpoint_number: u8,
//         direction: UsbDirection,
//         size: u16,
//         interval: u8,
//     ) -> Option<InterruptPipe> {
//         let pipe = self.alloc_pipe(size)?;
//         let idx = pipe.bus_ref as usize;
        
//         let regs = T::regs();
//         let dpram = T::dpram();

//         dpram.ep_in_control(idx).write(|w| {
//             w.set_endpoint_type(EpControlEndpointType::INTERRUPT);
//             w.set_interrupt_per_buff(true);
//             // FIXME: host_poll_interval (bits 16:25)
//             let interval = interval as u32 - 1;
//             w.0 |= interval << 16;
//             // FIXME: Index offset?
//             w.set_buffer_address(
//                 DPRAM_DATA_OFFSET + CONTROL_BUFFER_SIZE as u16 + (idx * CONTROL_BUFFER_SIZE) as u16
//             );
//             w.set_enable(true);
//         });

//         regs.sie_ctrl().modify(|w| { w.set_sof_sync(true) });
        
//         // FIXME(magic):
//         dpram.ep_in_buffer_control(idx + 1).write(|w| {
//             w.set_last(0, true);
//             w.set_pid(0, false);
//             w.set_full(0, false);
//             w.set_reset(true);
//             w.set_length(0, size);
//         });

//         // FIXME(delay):
//         cortex_m::asm::delay(12);
        
//         dpram.ep_in_buffer_control(idx + 1).modify(|w| w.set_available(0, true));        
//         regs.addr_endp_x(idx).write(|w| { 
//             w.set_address(device_address.into());
//             w.set_endpoint(endpoint_number);
//             w.set_intep_dir(direction == UsbDirection::Out);
//         });

//         // FIXME(delay):
//         cortex_m::asm::delay(12);
        
//         regs.int_ep_ctrl().modify(|w| {
//             w.set_int_ep_active(w.int_ep_active() | 1 << idx);
//         });

//         Some(pipe)
//     }

//     fn release_interrupt_pipe(&mut self, pipe_ref: u8) {
//         assert!(pipe_ref <= 15);
//         let dpram = T::dpram();
//         let idx = pipe_ref as usize;

//         // Disable interrupt polling
//         T::regs().int_ep_ctrl().modify(|w| {
//             w.set_int_ep_active(w.int_ep_active() & !(1 << idx))
//         });

//         // FIXME: bits(0)?
//         dpram.ep_in_control(idx).write(|_| {});
//         dpram.ep_in_buffer_control(idx + 1).write(|_| {});

//         T::regs().addr_endp_x(idx).write(|_| {});

//         // Mark as released
//         self.release_pipe(idx as u8);
//     }

//     fn pipe_continue(&mut self, pipe_ref: u8) {
//         assert!(pipe_ref <= 15);
//         let idx = pipe_ref as usize;

//         // EP1..=EP15 IN
//         // FIXME(index):
//         let control = T::dpram().ep_in_buffer_control(idx + 1);
//         control.modify(|w| {
//             w.set_last(0, true);
//             w.set_pid(0, !w.pid(0));
//             w.set_full(0, false);
//             w.set_reset(true);
//         });

//         // FIXME(delay):
//         cortex_m::asm::delay(12);

//         control.modify(|w| w.set_available(0, true))
//     }
// }

/// USB endpoint.
pub struct Channel<'d, T: Instance, E, D> {
    _phantom: PhantomData<(&'d mut T, E, D)>,
    index: usize,
    buf: EndpointBuffer<T>,
    desc: EndpointDescriptor,
    dev_addr: u8,

    /// DATA0-DATA1 state
    pid: bool,
    /// Send PRE packet
    pre: bool,
}

impl<'d, T: Instance, E: channel::Type, D: channel::Direction> Channel<'d, T, E, D> {
    /// [EP_MEMORY]-relative address
    fn new(
        index: usize, 
        addr: u16, 
        desc: EndpointDescriptor,
        len: u16,
        dev_addr: u8,
        pre: bool,
    ) -> Self {
        // TODO: assert only in debug?
        assert!(addr + len <= EP_MEMORY_SIZE as u16);
        assert!(desc.max_packet_size <= EP_MEMORY_SIZE as u16);

        // TODO: Support isochronous, bulk, and interrupt OUT
        assert!(desc.ep_type() != EndpointType::Isochronous);
        assert!(desc.ep_type() != EndpointType::Bulk);
        assert!(!(desc.ep_type() == EndpointType::Interrupt && D::is_out()));
        
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
            pre,
        }
    }
}

type BufferControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpBufferControl, rp_pac::common::RW>;
type EpControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpControl, rp_pac::common::RW>;
type AddrControlReg = rp_pac::common::Reg<rp_pac::usb::regs::AddrEndpX, rp_pac::common::RW>;
impl<'d, T: Instance, E: channel::Type, D: channel::Direction> Channel<'d, T, E, D> {
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
        self.desc.ep_type() == EndpointType::Interrupt && D::is_in()
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
                w.set_intep_dir(D::is_out());
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
    fn set_setup_packet(&mut self, setup: &SetupPacket) {
        assert!(self.desc.ep_type() == EndpointType::Control);
        let dpram = T::dpram();
        dpram.setup_packet_low().write(|w| {
            w.set_bmrequesttype(setup.request_type.bits()); 
            w.set_brequest(setup.request);
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
    async fn send_setup(&mut self, setup: &SetupPacket) {
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
    async fn control_status(&mut self, active_direction_out: bool) {
        // Wait transfer buffer to be free
        self.wait_ready_for_transaction().await;
        
        // Set this channel for transaction
        self.set_current();
        
        // Status packet always have DATA1
        self.pid = true;
        if active_direction_out {
            self.set_data_in(0);
        } else {
            self.set_data_out(&[]);
        }
        
        self.start_transaction();
        self.wait_trans_complete().await;

        self.clear_current();
    }
}

impl<'d, T: Instance, E: channel::Type, D: channel::Direction> UsbChannel<E, D> for Channel<'d, T, E, D> {
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, HostError>
    where 
        E: channel::IsControl,
        D: channel::IsIn {
        // TODO: Pass directly
        // Setup stage
        self.send_setup(setup).await;

        // Data stage
        let read = self.request_in(buf).await?;

        // Status stage
        self.control_status(false).await;

        Ok(read)
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, HostError>
    where 
        E: channel::IsControl,
        D: channel::IsOut {
        // TODO: Pass directly
        // Setup stage
        self.send_setup(setup).await;

        // Data stage
        let written = self.request_out(buf).await?;

        // Status stage
        self.control_status(true).await;

        Ok(written)
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, HostError>
    where 
        D: channel::IsIn {
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
                return Err(HostError::BufferOverflow);
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

    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, HostError>
    where 
        D: channel::IsOut {
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
               
        Ok(count)
    }
}

impl<'d, T: Instance> UsbHostDriver for Driver<'d, T> {
    type Channel<E: channel::Type, D: channel::Direction> = Channel<'d, T, E, D>;

    async fn wait_for_device_event(&self) -> DeviceEvent {
        let is_connected = |status: u8| match status {
            0b01 | 0b10 => true,
            _ => false
        };
        
        // Read current state
        let was = is_connected(T::regs().sie_status().read().speed());
        // Enable conn/dis irq
        T::regs().inte().modify(|w| { w.set_host_conn_dis(true); });
        let ev = poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            
            let now = is_connected(T::regs().sie_status().read().speed());
            match (was, now) {
                (true, false) => Poll::Ready(DeviceEvent::Disconnected),
                (false, true) => Poll::Ready(DeviceEvent::Connected),
                _ => Poll::Pending
            }        
        }).await;
        // FIXME: ?
        // T::regs().sie_status().write_clear(|w| { w.set_speed(0b11); });
        ev
    }

    async fn bus_reset(&self) {
        T::regs().sie_ctrl().modify(|w| {
            w.set_reset_bus(true);
        });

        embassy_time::Timer::after_millis(50).await;
    }

    // FIXME: Max packet size
    fn retarget_channel<D: channel::Direction>(
        &self, 
        channel: &mut Self::Channel<channel::Control, D>,
        max_packet_size: u8,
        addr: u8,
        pre: bool,
    ) -> Result<(), HostError> {
        channel.pre = pre;
        channel.dev_addr = addr;
        channel.desc.max_packet_size = max_packet_size as u16;
        Ok(())
    }

    fn alloc_channel<E: channel::Type, D: channel::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointDescriptor,
        pre: bool,
    ) -> Result<Self::Channel<E, D>, HostError> {
        if endpoint.ep_type() != E::ep_type() {
            return Err(HostError::InvalidDescriptor)
        }
        if E::ep_type() == EndpointType::Interrupt {
            let alloc = self.allocated_pipes.load(Ordering::Acquire);
            let free_index = (1..16)
                .find(|i| alloc & (1 << i) == 0)
                .ok_or(HostError::OutOfChannels)? as u8;
        
            self.allocated_pipes.store(alloc | 1 << free_index, Ordering::Release);
            // Use fixed layout
            let addr = DPRAM_DATA_OFFSET + MAIN_BUFFER_SIZE as u16 + free_index as u16 * 64;

            Ok(Channel::new(free_index as _, addr, *endpoint, 64, addr as u8, pre))
        } else {
            let index = self.channel_index.fetch_add(1, Ordering::Relaxed);
            Ok(Channel::new(index, DPRAM_DATA_OFFSET, *endpoint, MAIN_BUFFER_SIZE as u16, addr, pre))
        }        
    }

    fn drop_channel<E: channel::Type, D: channel::Direction>(
        &self, 
        channel: &mut Self::Channel<E, D>
    ) {
        if E::ep_type() == EndpointType::Interrupt {
            // TODO: Disable interrupt?
            self.allocated_pipes.fetch_and(!(1 << channel.index), Ordering::Relaxed);
        }
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
