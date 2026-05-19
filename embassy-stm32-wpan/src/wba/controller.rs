//! High-level BLE API for STM32WBA
//!
//! This module provides the main `Ble` struct that manages the BLE stack lifecycle
//! and provides access to GAP functionality including connection management.

#[cfg(feature = "bt-hci")]
use core::cell::RefCell;
use core::ops::Deref;
#[cfg(feature = "bt-hci")]
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_stm32::interrupt;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use stm32_bindings::ble::{BLE_SLEEPMODE_RUNNING, BleStack_Process, BleStack_Request};

use crate::Runtime;
use crate::platform::Platform;
use crate::wba::host_if::{MAX_BLE_PKT_SIZE, TASK_BLE_HOST_MASK, TASK_LINK_LAYER_MASK, TASK_PRIO_BLE_HOST};
use crate::wba::linklayer_plat::{EVENT_CHANNEL, PLATFORM, run_radio_high_isr, run_radio_sw_low_isr};
use crate::wba::ll_sys::init_ble_stack;
use crate::wba::util_seq;

/// High interrupt handler.
pub struct HighInterruptHandler {}

impl interrupt::typelevel::Handler<interrupt::typelevel::RADIO> for HighInterruptHandler {
    unsafe fn on_interrupt() {
        run_radio_high_isr();
    }
}

/// Low interrupt handler.
pub struct LowInterruptHandler {}

impl interrupt::typelevel::Handler<interrupt::typelevel::HASH> for LowInterruptHandler {
    unsafe fn on_interrupt() {
        run_radio_sw_low_isr();
    }
}

/// BLE stack background processing task, registered as a sequencer task.
///
/// Matches ST's BleStack_Process_BG:
///   - Calls BleStack_Process() once
///   - If it returns 0 (more work pending), re-queues via BleStackCB_Process
///   - If non-zero (idle/can sleep), does NOT re-queue
///
/// IMPORTANT: This runs on the sequencer's stack context, matching the
/// C reference implementation where BleStack_Process is a UTIL_SEQ task.
unsafe extern "C" fn ble_stack_process_bg() {
    let result = BleStack_Process();

    trace!("BleStack_Process called, result={}", result);

    if result == BLE_SLEEPMODE_RUNNING as u8 {
        // More work to do - re-queue
        util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);
    }
}

#[derive(Clone, Copy)]
pub struct ChannelPacket(pub [u8; MAX_BLE_PKT_SIZE], pub usize);

impl ChannelPacket {
    pub fn copy_from(&mut self, data: &[u8], ext_data: &[u8]) {
        self.0[..data.len()].copy_from_slice(data);
        self.0[data.len()..][..ext_data.len()].copy_from_slice(ext_data);
        self.1 = data.len() + ext_data.len();
    }
}

impl Deref for ChannelPacket {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0[..self.1]
    }
}

impl Default for ChannelPacket {
    fn default() -> Self {
        Self([0u8; MAX_BLE_PKT_SIZE], 0)
    }
}

pub struct Controller<'d, T: Runtime> {
    _runtime: &'d mut T,
    receiver: zerocopy_channel::Receiver<'static, CriticalSectionRawMutex, ChannelPacket>,
    cmd_buf: ([u8; 255], usize),
}

impl<'d, T: Runtime> Controller<'d, T> {
    /// Create a new BLE instance
    ///
    /// Requires hardware peripheral instances for RNG, AES, and PKA.
    /// These are stored in statics so the BLE stack's `extern "C"` callbacks can access them.
    pub async fn new(
        platform: &'static Platform,
        runtime: &'d mut T,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, ()> {
        // SAFETY: Safe IFF we have a runtime token
        let receiver = unsafe {
            let (sender, receiver) = platform.get_channel().split();

            EVENT_CHANNEL.replace(sender);
            PLATFORM.replace(platform);

            receiver
        };

        trace!("Waiting for rng to fill...");
        // Wait for the rng buffer to fill
        platform.wait_rng_ready().await;

        trace!("Waiting for rng to fill...done!");

        // Set-up sequencer stack
        util_seq::seq_resume();

        // 0. Initialize the BLE stack using BleStack_Init
        // This properly initializes the BLE host stack including memory management,
        // which is required before ll_intf_init can work properly.
        init_ble_stack().map_err(|status| {
            error!("BLE stack initialization failed: 0x{:02X}", status);
            ()
        })?;

        util_seq::UTIL_SEQ_RegTask(TASK_BLE_HOST_MASK, 0, Some(ble_stack_process_bg));

        trace!(
            "Registered BleStack_Process_BG as sequencer task (mask=0x{:08X})",
            TASK_BLE_HOST_MASK
        );

        // Schedule the initial tasks and kick the BLE stack.
        // BLE init and GAP setup happened before the runner started, so there may be
        // pending HCI commands that need BleStack_Process to deliver them to the LL.
        util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);
        util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, 0);
        util_seq::seq_resume();

        // Flush pending HCI commands through BleStack_Process.
        // This delivers scan enable, connection parameters, etc. to the LL.
        util_seq::UTIL_SEQ_ResumeTask(TASK_BLE_HOST_MASK);
        util_seq::seq_resume();

        // Run the sequencer once more to process any LL events from the enable
        util_seq::UTIL_SEQ_ResumeTask(TASK_LINK_LAYER_MASK);
        util_seq::seq_resume();

        // Wake the runner
        platform.start_run_ble();

        Ok(Self {
            receiver,
            cmd_buf: ([0u8; 255], 0),
            _runtime: runtime,
        })
    }

    fn exec<R>(&mut self, f: impl FnOnce(&mut [u8; 255]) -> R) -> R {
        let ret = f(&mut self.cmd_buf.0);
        self.cmd_buf.1 = unsafe { BleStack_Request(&mut self.cmd_buf.0 as *mut u8) }.into();

        ret
    }

    fn pop_buf(&mut self) -> Option<&[u8]> {
        match self.cmd_buf.1 {
            0 => None,
            len => {
                self.cmd_buf.1 = 0;

                Some(&self.cmd_buf.0[..len])
            }
        }
    }

    #[cfg(feature = "wb-hci")]
    pub async fn read_event(&mut self) -> Result<stm32wb_hci::Event, stm32wb_hci::event::Error> {
        use stm32wb_hci::Event;
        use stm32wb_hci::event::Packet;

        if let Some(buf) = self.pop_buf() {
            Event::new(Packet(&buf[1..]))
        } else {
            let slot = self.receiver.receive().await;
            // Parse and queue the event for processing.
            // Skip byte 0 (0x04 HCI Event packet indicator) — the parser expects
            // data starting at the event code byte.
            let parse_data = if *&slot.len() >= 2 && *&slot[0] == 0x04 {
                &slot[1..]
            } else {
                &slot
            };

            let event = Event::new(Packet(parse_data));

            slot.receive_done();

            event
        }
    }
}

#[cfg(feature = "wb-hci")]
impl<'d, T: Runtime> stm32wb_hci::Controller for Controller<'d, T> {
    async fn controller_read_into(&mut self, _buf: &mut [u8]) {
        panic!("use `read_event` to read events")
    }

    async fn controller_write(&mut self, opcode: stm32wb_hci::Opcode, payload: &[u8]) {
        use stm32wb_hci::host::HciHeader;
        use stm32wb_hci::vendor::CommandHeader;

        self.exec(|buf| {
            let (header, pkt) = buf.split_at_mut(CommandHeader::HEADER_LENGTH);

            CommandHeader::new(opcode, payload.len()).copy_into_slice(header);
            pkt[..payload.len()].copy_from_slice(payload);
        });
    }
}

#[cfg(feature = "bt-hci")]
const ERR: bt_hci::cmd::Error<embedded_io::ErrorKind> = bt_hci::cmd::Error::Io(embedded_io::ErrorKind::InvalidData);

#[cfg(feature = "bt-hci")]
pub struct ControllerAdapter<'d, T: Runtime> {
    controller: NoopMutex<RefCell<Controller<'d, T>>>,
    pending_evt: AtomicBool,
}

#[cfg(feature = "bt-hci")]
impl<'d, T: Runtime> ControllerAdapter<'d, T> {
    pub const fn new(controller: Controller<'d, T>) -> Self {
        Self {
            controller: NoopMutex::const_new(NoopRawMutex::new(), RefCell::new(controller)),
            pending_evt: AtomicBool::new(false),
        }
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, T: Runtime> embedded_io::ErrorType for ControllerAdapter<'d, T> {
    type Error = embedded_io::ErrorKind;
}

#[cfg(feature = "bt-hci")]
impl<'d, T: Runtime> bt_hci::controller::Controller for ControllerAdapter<'d, T> {
    async fn write_acl_data(&self, packet: &bt_hci::data::AclPacket<'_>) -> Result<(), Self::Error> {
        use bt_hci::WriteHci;
        use bt_hci::transport::WithIndicator;

        let mut controller = self.controller.borrow().borrow_mut();

        controller.exec(|buf| {
            WithIndicator::new(packet)
                .write_hci(&mut buf[..])
                .map_err(|_| embedded_io::ErrorKind::InvalidData)
        })
    }

    async fn write_iso_data(&self, _packet: &bt_hci::data::IsoPacket<'_>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn write_sync_data(&self, packet: &bt_hci::data::SyncPacket<'_>) -> Result<(), Self::Error> {
        use bt_hci::WriteHci;
        use bt_hci::transport::WithIndicator;

        let mut controller = self.controller.borrow().borrow_mut();

        controller.exec(|buf| {
            WithIndicator::new(packet)
                .write_hci(&mut buf[..])
                .map_err(|_| embedded_io::ErrorKind::InvalidData)
        })
    }

    async fn read<'a>(&self, _buf: &'a mut [u8]) -> Result<bt_hci::ControllerToHostPacket<'a>, Self::Error> {
        use core::future::poll_fn;
        use core::task::Poll;

        use bt_hci::{ControllerToHostPacket, FromHciBytes};

        let buf = poll_fn(|cx| {
            let mut controller = self.controller.borrow().borrow_mut();

            if self.pending_evt.swap(false, Ordering::AcqRel) {
                // Advance the channel
                controller.receiver.try_receive().unwrap().receive_done();
            }

            let Poll::Ready(slot) = controller.receiver.poll_receive(cx) else {
                return Poll::Pending;
            };

            self.pending_evt.store(true, Ordering::Release);

            // Optimization depends on the assumption that the event is dropped before read is called again
            Poll::Ready(unsafe { core::slice::from_raw_parts(&slot.0 as *const _ as *const u8, slot.1) })
        })
        .await;

        ControllerToHostPacket::from_hci_bytes_complete(&buf).map_err(|_| embedded_io::ErrorKind::InvalidData)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, T: Runtime, C> bt_hci::controller::ControllerCmdSync<C> for ControllerAdapter<'d, T>
where
    C: bt_hci::cmd::SyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<C::Return, bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::transport::WithIndicator;
        use bt_hci::{WriteHci, cmd};

        use crate::util::make_cc_with_cs;

        let mut controller = self.controller.borrow().borrow_mut();

        debug!("Executing command with opcode {}", C::OPCODE.0);
        controller.exec(|buf| WithIndicator::new(cmd).write_hci(&mut buf[..]).map_err(|_| ERR))?;

        let buf = controller.pop_buf().ok_or(ERR)?;
        let e = make_cc_with_cs(buf)?;

        let r = e.to_result::<C>().map_err(cmd::Error::Hci)?;
        debug!("Done executing command with opcode {}", C::OPCODE.0);
        Ok(r)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, T: Runtime, C> bt_hci::controller::ControllerCmdAsync<C> for ControllerAdapter<'d, T>
where
    C: bt_hci::cmd::AsyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<(), bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::WriteHci;
        use bt_hci::transport::WithIndicator;

        use crate::util::make_cc_with_cs;

        let mut controller = self.controller.borrow().borrow_mut();

        debug!("Executing command with opcode {}", C::OPCODE.0);
        controller.exec(|buf| WithIndicator::new(cmd).write_hci(&mut buf[..]).map_err(|_| ERR))?;

        let buf = controller.pop_buf().ok_or(ERR)?;
        let e = make_cc_with_cs(buf)?;

        e.status.to_result()?;

        debug!("Done executing command with opcode {}", C::OPCODE.0);
        Ok(())
    }
}

impl<'d, T: Runtime> Drop for Controller<'d, T> {
    fn drop(&mut self) {
        // Zero host stack buffers and reset the one-time LL init guard so
        // init_ble_stack() → BleStack_Init() can run cleanly on next Ble::new().
        crate::wba::ll_sys::reset_ble_stack();
    }
}

/// Version information from the BLE controller
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VersionInfo {
    pub hci_version: u8,
    pub hci_revision: u16,
    pub lmp_version: u8,
    pub manufacturer_name: u16,
    pub lmp_subversion: u16,
}
