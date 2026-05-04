//! High-level BLE API for STM32WBA
//!
//! This module provides the main `Ble` struct that manages the BLE stack lifecycle
//! and provides access to GAP functionality including connection management.

use core::cell::RefCell;
use core::ops::Deref;
use core::sync::atomic::Ordering;

use embassy_stm32::aes::Aes;
use embassy_stm32::interrupt;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::Pka;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::Mutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use stm32_bindings::ble::{BleStack_Process, BleStack_Request};
use stm32wb_hci::event::Packet;
use stm32wb_hci::host::HciHeader;
use stm32wb_hci::vendor::CommandHeader;
use stm32wb_hci::{Event, event};

use crate::bluetooth::error::BleError;
use crate::host_if::{MAX_BLE_PKT_SIZE, TASK_BLE_HOST_MASK, TASK_LINK_LAYER_MASK, TASK_PRIO_BLE_HOST};
use crate::linklayer_plat::{
    EVENT_CHANNEL, HARDWARE_AES, HARDWARE_PKA, HARDWARE_RNG, run_radio_high_isr, run_radio_sw_low_isr,
};
use crate::runner::{BLE_INIT, BLE_INIT_WAKER, BLE_SLEEPMODE_RUNNING};
use crate::util_seq;
use crate::wba::ll_sys::init_ble_stack;

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

    if result == BLE_SLEEPMODE_RUNNING {
        // More work to do - re-queue
        util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);
    }
}

#[derive(Clone, Copy)]
pub struct ChannelPacket(pub [u8; MAX_BLE_PKT_SIZE], pub usize);

impl ChannelPacket {
    pub fn copy_from(&mut self, data: &[u8]) {
        self.0[..data.len()].copy_from_slice(data);
        self.1 = data.len();
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

pub struct ControllerState {
    channel: zerocopy_channel::Channel<'static, CriticalSectionRawMutex, ChannelPacket>,
}

impl ControllerState {
    pub fn new<const N: usize>(buf: &'static mut [ChannelPacket; N]) -> Self {
        Self {
            channel: zerocopy_channel::Channel::new(buf),
        }
    }
}

#[macro_export]
macro_rules! declare_controller_state {
    ($buf:ident, $state:ident, $size:expr) => {
        static $buf: ::static_cell::StaticCell<[::embassy_stm32_wpan::ChannelPacket; $size]> =
            ::static_cell::StaticCell::new();
        static $state: ::static_cell::StaticCell<::embassy_stm32_wpan::ControllerState> =
            ::static_cell::StaticCell::new();
    };
}

#[macro_export]
macro_rules! use_controller_state {
    ($buf:ident, $state:ident, $size:expr) => {{
        $state.init(::embassy_stm32_wpan::ControllerState::new(
            $buf.init([::embassy_stm32_wpan::ChannelPacket::default(); $size]),
        ))
    }};
}

#[macro_export]
macro_rules! new_controller_state {
    ($size:expr) => {{
        ::embassy_stm32_wpan::declare_controller_state!(EVENT_BUFFER, EVENT_STATE, $size);
        ::embassy_stm32_wpan::use_controller_state!(EVENT_BUFFER, EVENT_STATE, $size)
    }};
}

pub struct Controller {
    state_ptr: *mut ControllerState,
    receiver: zerocopy_channel::Receiver<'static, CriticalSectionRawMutex, ChannelPacket>,
    cmd_buf: ([u8; 255], usize),
}

impl Controller {
    /// Create a new BLE instance
    ///
    /// Requires hardware peripheral instances for RNG, AES, and PKA.
    /// These are stored in statics so the BLE stack's `extern "C"` callbacks can access them.
    pub async fn new(
        state: &'static mut ControllerState,
        rng: &'static Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>,
        aes: Option<&'static Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AesPeriph, Blocking>>>>,
        pka: Option<&'static Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PkaPeriph>>>>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, BleError> {
        let state_ptr = state as *mut ControllerState;
        let (sender, receiver) = state.channel.split();
        unsafe {
            EVENT_CHANNEL.replace(sender);
            HARDWARE_RNG.replace(rng);
            aes.map(|aes| HARDWARE_AES.replace(aes));
            pka.map(|pka| HARDWARE_PKA.replace(pka));
        }

        // Set-up sequencer stack
        util_seq::seq_resume();

        // 0. Initialize the BLE stack using BleStack_Init
        // This properly initializes the BLE host stack including memory management,
        // which is required before ll_intf_init can work properly.
        init_ble_stack().map_err(|status| {
            error!("BLE stack initialization failed: 0x{:02X}", status);
            BleError::InitializationFailed
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
        util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, 0);
        util_seq::seq_resume();

        // Wake the runner
        BLE_INIT.store(true, Ordering::Release);
        BLE_INIT_WAKER.wake();

        Ok(Self {
            state_ptr,
            receiver,
            cmd_buf: ([0u8; 255], 0),
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

    /// Consume the controller and return the controller state so it can be
    /// passed to the next `HCI::new()` or `HCI::new_dtm()` call.
    pub(crate) fn release_state(self) -> &'static mut ControllerState {
        let ptr = self.state_ptr;
        // Drop self first — runs Drop (reset_ble_stack) and drops receiver —
        // so no references into *ptr exist before we reconstruct the &'static mut.
        drop(self);
        unsafe { &mut *ptr }
    }

    pub async fn read_event(&mut self) -> Result<Event, event::Error> {
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

impl stm32wb_hci::Controller for Controller {
    async fn controller_read_into(&mut self, _buf: &mut [u8]) {
        panic!("use `read_event` to read events")
    }

    async fn controller_write(&mut self, opcode: stm32wb_hci::Opcode, payload: &[u8]) {
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
pub struct AtomicController {
    controller: NoopMutex<RefCell<Controller>>,
}

#[cfg(feature = "bt-hci")]
impl AtomicController {
    pub const fn new(controller: Controller) -> Self {
        Self {
            controller: Mutex::const_new(NoopRawMutex::new(), RefCell::new(controller)),
        }
    }
}

#[cfg(feature = "bt-hci")]
impl embedded_io::ErrorType for AtomicController {
    type Error = embedded_io::ErrorKind;
}

#[cfg(feature = "bt-hci")]
impl bt_hci::controller::Controller for AtomicController {
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

    async fn read<'a>(&self, buf: &'a mut [u8]) -> Result<bt_hci::ControllerToHostPacket<'a>, Self::Error> {
        use core::future::poll_fn;
        use core::task::Poll;

        use bt_hci::{ControllerToHostPacket, FromHciBytes};

        // TODO: install buffer so that Linklayer_Plat copies directly into buffer rather than going through the channel

        let len = poll_fn(|cx| {
            let mut controller = self.controller.borrow().borrow_mut();

            let Poll::Ready(slot) = controller.receiver.poll_receive(cx) else {
                return Poll::Pending;
            };

            // Must copy to extend the lifetime
            buf[..*&slot.len()].copy_from_slice(&slot);

            Poll::Ready(*&slot.len())
        })
        .await;

        ControllerToHostPacket::from_hci_bytes_complete(&buf[..len]).map_err(|_| embedded_io::ErrorKind::InvalidData)
    }
}

#[cfg(feature = "bt-hci")]
impl<C> bt_hci::controller::ControllerCmdSync<C> for AtomicController
where
    C: bt_hci::cmd::SyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<C::Return, bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::event::{CommandComplete, CommandCompleteWithStatus, EventKind};
        use bt_hci::transport::WithIndicator;
        use bt_hci::{ControllerToHostPacket, FromHciBytes, WriteHci, cmd};

        let mut controller = self.controller.borrow().borrow_mut();

        //info!("Executing command with opcode {}", C::OPCODE);
        controller.exec(|buf| WithIndicator::new(cmd).write_hci(&mut buf[..]).map_err(|_| ERR))?;

        let buf = controller.pop_buf().ok_or(ERR)?;
        let pkt = ControllerToHostPacket::from_hci_bytes_complete(&buf[1..]).map_err(|_| ERR)?;

        let ControllerToHostPacket::Event(ref event) = pkt else {
            return Err(ERR);
        };

        if event.kind != EventKind::CommandComplete {
            return Err(ERR);
        }

        let e = CommandComplete::from_hci_bytes_complete(event.data).map_err(|_| ERR)?;
        let e: CommandCompleteWithStatus = e.try_into().map_err(|_| ERR)?;

        let r = e.to_result::<C>().map_err(cmd::Error::Hci)?;
        // info!("Done executing command with opcode {}", C::OPCODE);
        Ok(r)
    }
}

#[cfg(feature = "bt-hci")]
impl<C> bt_hci::controller::ControllerCmdAsync<C> for AtomicController
where
    C: bt_hci::cmd::AsyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<(), bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::event::{CommandStatus, EventKind};
        use bt_hci::transport::WithIndicator;
        use bt_hci::{ControllerToHostPacket, FromHciBytes, WriteHci};

        let mut controller = self.controller.borrow().borrow_mut();

        //info!("Executing command with opcode {}", C::OPCODE);
        controller.exec(|buf| WithIndicator::new(cmd).write_hci(&mut buf[..]).map_err(|_| ERR))?;

        let buf = controller.pop_buf().ok_or(ERR)?;
        let pkt = ControllerToHostPacket::from_hci_bytes_complete(&buf[1..]).map_err(|_| ERR)?;

        let ControllerToHostPacket::Event(ref event) = pkt else {
            return Err(ERR);
        };

        if event.kind != EventKind::CommandStatus {
            return Err(ERR);
        }

        let e = CommandStatus::from_hci_bytes_complete(event.data).map_err(|_| ERR)?;

        e.status.to_result()?;

        // info!("Done executing command with opcode {}", C::OPCODE);
        Ok(())
    }
}

impl Drop for Controller {
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
