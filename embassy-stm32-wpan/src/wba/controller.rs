//! High-level BLE API for STM32WBA
//!
//! This module provides the main `Ble` struct that manages the BLE stack lifecycle
//! and provides access to GAP functionality including connection management.

use core::cell::RefCell;
use core::sync::atomic::Ordering;

use embassy_stm32::aes::Aes;
use embassy_stm32::interrupt;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::Pka;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use stm32_bindings::ble::BleStack_Process;
use stm32wb_hci::event::Packet;
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
    receiver: zerocopy_channel::Receiver<'static, CriticalSectionRawMutex, ChannelPacket>,
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
        loop {
            let result = unsafe { BleStack_Process() };
            if result != BLE_SLEEPMODE_RUNNING {
                break;
            }
        }

        // Run the sequencer once more to process any LL events from the enable
        util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, 0);
        util_seq::seq_resume();

        // Wake the runner
        BLE_INIT.store(true, Ordering::Release);
        BLE_INIT_WAKER.wake();

        Ok(Self { receiver })
    }

    pub async fn read_event(&mut self) -> Result<Event, event::Error> {
        let slot = self.receiver.receive().await;
        let event = Event::new(Packet(&slot.0[..slot.1]));

        slot.receive_done();

        event
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
