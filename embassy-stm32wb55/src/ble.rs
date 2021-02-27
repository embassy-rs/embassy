use crate::interrupt::{self, Interrupt};

use bbqueue::{
    consts::{U32, U514},
    BBBuffer, ConstBBBuffer,
};
use bluetooth_hci::host::uart::Error;
use bluetooth_hci::{
    event::command::{CommandComplete, ReturnParameters},
    host::uart::{Hci as UartHci, Packet},
    Event,
};
use embassy::util::Signal;
use stm32wb55::event::{FirmwareKind, Stm32Wb5xError};
use stm32wb55::{event::Stm32Wb5xEvent, RadioCoprocessor};
use stm32wb_hal::{
    ipcc::Ipcc,
    tl_mbox::{shci::ShciBleInitCmdParam, TlMbox},
};

type BufSize = U514;
static BB: BBBuffer<BufSize> = BBBuffer(ConstBBBuffer::new());

type EvtQueueSize = U32;
type HeaplessEvtQueue =
    heapless::spsc::Queue<Packet<Stm32Wb5xEvent>, EvtQueueSize, u8, heapless::spsc::SingleCore>;

/// Reexport of the BLE stack type with data buffer.
pub type Rc = RadioCoprocessor<'static, BufSize>;

struct State {
    tx_int: Signal<()>,
    rx_int: Signal<()>,
}

static STATE: State = State {
    tx_int: Signal::new(),
    rx_int: Signal::new(),
};

static mut RADIO_COPROSESSOR: *mut Rc = core::ptr::null_mut();

/// Type alias for the BLE stack's transport layer errors.
pub type BleTransportLayerError = bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>;

/// BLE stack or system errors.
#[derive(Debug)]
pub enum BleError<E: core::fmt::Debug> {
    NbError(nb::Error<E>),
    EmptyError,
    UnexpectedEvent,
    NotInitialized,
}

impl<E: core::fmt::Debug> From<nb::Error<()>> for BleError<E> {
    fn from(_: nb::Error<()>) -> Self {
        BleError::EmptyError
    }
}

impl From<nb::Error<BleTransportLayerError>> for BleError<BleTransportLayerError> {
    fn from(e: nb::Error<bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>>) -> Self {
        BleError::NbError(e)
    }
}

/// Defines BLE stack interactions. It should be instantiated only once.
pub struct Ble {
    rx_int: interrupt::IPCC_C1_RX_IT,
    tx_int: interrupt::IPCC_C1_TX_IT,
    deferred_events: HeaplessEvtQueue,
}

impl Ble {
    /// Initializes the BLE stack and returns a status response from the BLE stack.
    pub async fn init(
        rx_int: interrupt::IPCC_C1_RX_IT,
        tx_int: interrupt::IPCC_C1_TX_IT,
        ble_config: ShciBleInitCmdParam,
        mbox: TlMbox,
        ipcc: Ipcc,
    ) -> Result<Self, BleError<Error<(), Stm32Wb5xError>>> {
        STATE.tx_int.reset();
        STATE.rx_int.reset();

        // Register ISRs
        tx_int.set_handler(Self::on_tx_irq);
        rx_int.set_handler_context(core::ptr::null_mut());
        rx_int.set_handler(Self::on_rx_irq);
        rx_int.set_handler_context(core::ptr::null_mut());

        let (producer, consumer) = BB.try_split().unwrap();
        let mut rc = RadioCoprocessor::new(producer, consumer, mbox, ipcc, ble_config);
        unsafe {
            RADIO_COPROSESSOR = &mut rc;
        }

        // Boot coprocessor
        stm32wb_hal::pwr::set_cpu2(true);

        let mut evt_queue = unsafe { heapless::spsc::Queue::u8_sc() };
        match Self::receive_event_helper(&mut evt_queue, &mut rc, false).await {
            Ok(Packet::Event(Event::Vendor(Stm32Wb5xEvent::CoprocessorReady(
                FirmwareKind::Wireless,
            )))) => Ok(Self {
                rx_int,
                tx_int,
                deferred_events: evt_queue,
            }),
            Err(e) => Err(BleError::NbError(e)),
            _ => Err(BleError::UnexpectedEvent),
        }
    }

    /// Sends an HCI BLE command and awaits for a response from the BLE stack.
    pub async fn perform_command(
        &mut self,
        command: impl Fn(&mut Rc) -> nb::Result<(), ()>,
    ) -> Result<ReturnParameters<Stm32Wb5xEvent>, BleError<Error<(), Stm32Wb5xError>>> {
        let rc = unsafe { RADIO_COPROSESSOR.as_mut() };
        if let Some(rc) = rc {
            cortex_m::interrupt::free(|_| command(rc))?;
            let response = Self::receive_event_helper(&mut self.deferred_events, rc, true).await?;
            if let Packet::Event(Event::CommandComplete(CommandComplete {
                return_params,
                num_hci_command_packets: _,
            })) = response
            {
                Ok(return_params)
            } else {
                Err(BleError::UnexpectedEvent)
            }
        } else {
            Err(BleError::NotInitialized)
        }
    }

    /// Awaits for a BLE event from the BLE stack.
    pub async fn receive_event(
        &mut self,
    ) -> Result<Packet<Stm32Wb5xEvent>, BleError<Error<(), Stm32Wb5xError>>> {
        let rc = unsafe { RADIO_COPROSESSOR.as_mut() };
        if let Some(rc) = rc {
            Ok(Self::receive_event_helper(&mut self.deferred_events, rc, false).await?)
        } else {
            Err(BleError::NotInitialized)
        }
    }

    /// Returns `true` if there are some event(s) to be received.
    pub fn has_events(&self) -> bool {
        STATE.rx_int.signaled() || self.deferred_events.peek().is_some()
    }

    async fn receive_event_helper(
        queue: &mut HeaplessEvtQueue,
        rc: &mut Rc,
        need_cmd_response: bool,
    ) -> nb::Result<Packet<Stm32Wb5xEvent>, Error<(), Stm32Wb5xError>> {
        loop {
            let event = cortex_m::interrupt::free(|_| {
                rc.process_events();
                rc.read().ok()
            });

            // If the receiver is only interested in command response events,
            // it will be an error to return the first event from the event queue since
            // it is not guaranteed that no events occurred in between of command execution and
            // response.
            // Thus we defer all of the non-command-response events into the temporary queue before
            // we get a command response event that will be returned.
            if need_cmd_response {
                if let Some(event) = event {
                    if let Packet::Event(Event::CommandComplete(_)) = event {
                        return Ok(event);
                    } else {
                        // Defer the currently received event into temporary queue
                        // for it to be processed later
                        queue.enqueue(event).unwrap();
                    }
                }
            } else {
                let event = queue.dequeue().or(event);
                if let Some(event) = event {
                    return Ok(event);
                }
            }

            STATE.rx_int.wait().await;
        }
    }

    unsafe fn on_tx_irq(_ctx: *mut ()) {
        if let Some(rc) = RADIO_COPROSESSOR.as_mut() {
            rc.handle_ipcc_tx();
        }

        STATE.tx_int.signal(());
    }

    unsafe fn on_rx_irq(_ctx: *mut ()) {
        if let Some(rc) = RADIO_COPROSESSOR.as_mut() {
            rc.handle_ipcc_rx();
        }

        STATE.rx_int.signal(());
    }
}

impl Drop for Ble {
    fn drop(&mut self) {
        self.rx_int.disable();
        self.rx_int.remove_handler();

        self.tx_int.disable();
        self.tx_int.remove_handler();

        STATE.rx_int.reset();
        STATE.tx_int.reset();
    }
}
