use crate::interrupt::{self, OwnedInterrupt};

use bbqueue::{consts::U514, BBBuffer, ConstBBBuffer};
use bluetooth_hci::host::uart::Error;
use bluetooth_hci::{
    event::command::{CommandComplete, ReturnParameters},
    host::uart::{Hci as UartHci, Packet},
    Event,
};
use embassy::util::Signal;
use stm32wb55::event::{FirmwareKind, Stm32Wb5xError};
use stm32wb55::{event::Stm32Wb5xEvent, RadioCoprocessor};
use stm32wb_hal::{ipcc::Ipcc, tl_mbox::{TlMbox, shci::ShciBleInitCmdParam}};

type BufSize = U514;
static BB: BBBuffer<BufSize> = BBBuffer(ConstBBBuffer::new());
pub type Rc = RadioCoprocessor<'static, BufSize>;

pub struct Ble {
    _rx_int: interrupt::IPCC_C1_RX_ITInterrupt,
    _tx_int: interrupt::IPCC_C1_TX_ITInterrupt,
}

struct State {
    tx_int: Signal<()>,
    rx_int: Signal<()>,
}

static STATE: State = State {
    tx_int: Signal::new(),
    rx_int: Signal::new(),
};

static mut RADIO_COPROSESSOR: *mut Rc = core::ptr::null_mut();

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

impl From<nb::Error<bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>>>
    for BleError<bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>>
{
    fn from(e: nb::Error<bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>>) -> Self {
        BleError::NbError(e)
    }
}

impl Ble {
    pub async fn init(
        rx_int: interrupt::IPCC_C1_RX_ITInterrupt,
        tx_int: interrupt::IPCC_C1_TX_ITInterrupt,
        ble_config: ShciBleInitCmdParam,
        mbox: TlMbox,
        ipcc: Ipcc,
    ) -> Result<Self, BleError<Error<(), Stm32Wb5xError>>> {
        STATE.tx_int.reset();
        STATE.rx_int.reset();

        // Register ISRs
        tx_int.set_handler(Self::on_tx_irq, core::ptr::null_mut());
        rx_int.set_handler(Self::on_rx_irq, core::ptr::null_mut());

        let (producer, consumer) = BB.try_split().unwrap();
        let mut rc = RadioCoprocessor::new(producer, consumer, mbox, ipcc, ble_config);
        unsafe {
            RADIO_COPROSESSOR = &mut rc;
        }

        // Boot coprocessor
        stm32wb_hal::pwr::set_cpu2(true);

        match Self::receive_event_helper(&mut rc).await {
            Ok(Packet::Event(Event::Vendor(Stm32Wb5xEvent::CoprocessorReady(
                FirmwareKind::Wireless,
            )))) => Ok(Self {
                _rx_int: rx_int,
                _tx_int: tx_int,
            }),
            Err(e) => Err(BleError::NbError(e)),
            _ => Err(BleError::UnexpectedEvent),
        }
    }

    pub async fn perform_command(
        &mut self,
        command: impl Fn(&mut Rc) -> nb::Result<(), ()>,
    ) -> Result<ReturnParameters<Stm32Wb5xEvent>, BleError<Error<(), Stm32Wb5xError>>> {
        let rc = unsafe { RADIO_COPROSESSOR.as_mut() };
        if let Some(rc) = rc {
            cortex_m::interrupt::free(|_| command(rc))?;
            let response = Self::receive_event_helper(rc).await?;
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
    pub async fn receive_event(&self)
        -> Result<Packet<Stm32Wb5xEvent>, BleError<Error<(), Stm32Wb5xError>>>{
        let rc = unsafe { RADIO_COPROSESSOR.as_mut() };
        if let Some(rc) = rc {
            Ok(Self::receive_event_helper(rc).await?)
        } else {
            Err(BleError::NotInitialized)
        }
    }

    async fn receive_event_helper(
        rc: &mut Rc,
    ) -> nb::Result<Packet<Stm32Wb5xEvent>, Error<(), Stm32Wb5xError>> {
        loop {
            let event = cortex_m::interrupt::free(|_| {
                rc.process_events();
                rc.read().ok()
            });

            if let Some(e) = event {
                return Ok(e);
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
