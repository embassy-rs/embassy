#[cfg(feature = "stm32wl")]
use embassy_stm32::interrupt;
#[cfg(feature = "stm32wl")]
use embassy_stm32::interrupt::InterruptExt;
#[cfg(feature = "stm32wl")]
use embassy_stm32::pac;
#[cfg(feature = "stm32wl")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "stm32wl")]
use embassy_sync::signal::Signal;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::digital::Wait;
use lora_phy::mod_params::RadioError::*;
use lora_phy::mod_params::{BoardType, RadioError};
use lora_phy::mod_traits::InterfaceVariant;

/// Interrupt handler.
#[cfg(feature = "stm32wl")]
pub struct InterruptHandler {}

#[cfg(feature = "stm32wl")]
impl interrupt::typelevel::Handler<interrupt::typelevel::SUBGHZ_RADIO> for InterruptHandler {
    unsafe fn on_interrupt() {
        interrupt::SUBGHZ_RADIO.disable();
        IRQ_SIGNAL.signal(());
    }
}

#[cfg(feature = "stm32wl")]
static IRQ_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[cfg(feature = "stm32wl")]
/// Base for the InterfaceVariant implementation for an stm32wl/sx1262 combination
pub struct Stm32wlInterfaceVariant<CTRL> {
    board_type: BoardType,
    rf_switch_rx: Option<CTRL>,
    rf_switch_tx: Option<CTRL>,
}

#[cfg(feature = "stm32wl")]
impl<'a, CTRL> Stm32wlInterfaceVariant<CTRL>
where
    CTRL: OutputPin,
{
    /// Create an InterfaceVariant instance for an stm32wl/sx1262 combination
    pub fn new(
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::SUBGHZ_RADIO, InterruptHandler>,
        rf_switch_rx: Option<CTRL>,
        rf_switch_tx: Option<CTRL>,
    ) -> Result<Self, RadioError> {
        interrupt::SUBGHZ_RADIO.disable();
        Ok(Self {
            board_type: BoardType::Stm32wlSx1262, // updated when associated with a specific LoRa board
            rf_switch_rx,
            rf_switch_tx,
        })
    }
}

#[cfg(feature = "stm32wl")]
impl<CTRL> InterfaceVariant for Stm32wlInterfaceVariant<CTRL>
where
    CTRL: OutputPin,
{
    fn set_board_type(&mut self, board_type: BoardType) {
        self.board_type = board_type;
    }
    async fn set_nss_low(&mut self) -> Result<(), RadioError> {
        let pwr = pac::PWR;
        pwr.subghzspicr().modify(|w| w.set_nss(pac::pwr::vals::Nss::LOW));
        Ok(())
    }
    async fn set_nss_high(&mut self) -> Result<(), RadioError> {
        let pwr = pac::PWR;
        pwr.subghzspicr().modify(|w| w.set_nss(pac::pwr::vals::Nss::HIGH));
        Ok(())
    }
    async fn reset(&mut self, _delay: &mut impl DelayUs) -> Result<(), RadioError> {
        let rcc = pac::RCC;
        rcc.csr().modify(|w| w.set_rfrst(true));
        rcc.csr().modify(|w| w.set_rfrst(false));
        Ok(())
    }
    async fn wait_on_busy(&mut self) -> Result<(), RadioError> {
        let pwr = pac::PWR;
        while pwr.sr2().read().rfbusys() == pac::pwr::vals::Rfbusys::BUSY {}
        Ok(())
    }

    async fn await_irq(&mut self) -> Result<(), RadioError> {
        unsafe { interrupt::SUBGHZ_RADIO.enable() };
        IRQ_SIGNAL.wait().await;
        Ok(())
    }

    async fn enable_rf_switch_rx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx)?,
            None => (),
        };
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchRx),
            None => Ok(()),
        }
    }
    async fn enable_rf_switch_tx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
    async fn disable_rf_switch(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
}

/// Base for the InterfaceVariant implementation for an stm32l0/sx1276 combination
pub struct Stm32l0InterfaceVariant<CTRL, WAIT> {
    board_type: BoardType,
    nss: CTRL,
    reset: CTRL,
    irq: WAIT,
    rf_switch_rx: Option<CTRL>,
    rf_switch_tx: Option<CTRL>,
}

impl<CTRL, WAIT> Stm32l0InterfaceVariant<CTRL, WAIT>
where
    CTRL: OutputPin,
    WAIT: Wait,
{
    /// Create an InterfaceVariant instance for an stm32l0/sx1276 combination
    pub fn new(
        nss: CTRL,
        reset: CTRL,
        irq: WAIT,
        rf_switch_rx: Option<CTRL>,
        rf_switch_tx: Option<CTRL>,
    ) -> Result<Self, RadioError> {
        Ok(Self {
            board_type: BoardType::Stm32l0Sx1276, // updated when associated with a specific LoRa board
            nss,
            reset,
            irq,
            rf_switch_rx,
            rf_switch_tx,
        })
    }
}

impl<CTRL, WAIT> InterfaceVariant for Stm32l0InterfaceVariant<CTRL, WAIT>
where
    CTRL: OutputPin,
    WAIT: Wait,
{
    fn set_board_type(&mut self, board_type: BoardType) {
        self.board_type = board_type;
    }
    async fn set_nss_low(&mut self) -> Result<(), RadioError> {
        self.nss.set_low().map_err(|_| NSS)
    }
    async fn set_nss_high(&mut self) -> Result<(), RadioError> {
        self.nss.set_high().map_err(|_| NSS)
    }
    async fn reset(&mut self, delay: &mut impl DelayUs) -> Result<(), RadioError> {
        delay.delay_ms(10).await;
        self.reset.set_low().map_err(|_| Reset)?;
        delay.delay_ms(10).await;
        self.reset.set_high().map_err(|_| Reset)?;
        delay.delay_ms(10).await;
        Ok(())
    }
    async fn wait_on_busy(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    async fn await_irq(&mut self) -> Result<(), RadioError> {
        self.irq.wait_for_high().await.map_err(|_| Irq)
    }

    async fn enable_rf_switch_rx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx)?,
            None => (),
        };
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchRx),
            None => Ok(()),
        }
    }
    async fn enable_rf_switch_tx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
    async fn disable_rf_switch(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
}

/// Base for the InterfaceVariant implementation for a generic Sx126x LoRa board
pub struct GenericSx126xInterfaceVariant<CTRL, WAIT> {
    board_type: BoardType,
    nss: CTRL,
    reset: CTRL,
    dio1: WAIT,
    busy: WAIT,
    rf_switch_rx: Option<CTRL>,
    rf_switch_tx: Option<CTRL>,
}

impl<CTRL, WAIT> GenericSx126xInterfaceVariant<CTRL, WAIT>
where
    CTRL: OutputPin,
    WAIT: Wait,
{
    /// Create an InterfaceVariant instance for an nrf52840/sx1262 combination
    pub fn new(
        nss: CTRL,
        reset: CTRL,
        dio1: WAIT,
        busy: WAIT,
        rf_switch_rx: Option<CTRL>,
        rf_switch_tx: Option<CTRL>,
    ) -> Result<Self, RadioError> {
        Ok(Self {
            board_type: BoardType::Rak4631Sx1262, // updated when associated with a specific LoRa board
            nss,
            reset,
            dio1,
            busy,
            rf_switch_rx,
            rf_switch_tx,
        })
    }
}

impl<CTRL, WAIT> InterfaceVariant for GenericSx126xInterfaceVariant<CTRL, WAIT>
where
    CTRL: OutputPin,
    WAIT: Wait,
{
    fn set_board_type(&mut self, board_type: BoardType) {
        self.board_type = board_type;
    }
    async fn set_nss_low(&mut self) -> Result<(), RadioError> {
        self.nss.set_low().map_err(|_| NSS)
    }
    async fn set_nss_high(&mut self) -> Result<(), RadioError> {
        self.nss.set_high().map_err(|_| NSS)
    }
    async fn reset(&mut self, delay: &mut impl DelayUs) -> Result<(), RadioError> {
        delay.delay_ms(10).await;
        self.reset.set_low().map_err(|_| Reset)?;
        delay.delay_ms(20).await;
        self.reset.set_high().map_err(|_| Reset)?;
        delay.delay_ms(10).await;
        Ok(())
    }
    async fn wait_on_busy(&mut self) -> Result<(), RadioError> {
        self.busy.wait_for_low().await.map_err(|_| Busy)
    }
    async fn await_irq(&mut self) -> Result<(), RadioError> {
        if self.board_type != BoardType::RpPicoWaveshareSx1262 {
            self.dio1.wait_for_high().await.map_err(|_| DIO1)?;
        } else {
            self.dio1.wait_for_rising_edge().await.map_err(|_| DIO1)?;
        }
        Ok(())
    }

    async fn enable_rf_switch_rx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx)?,
            None => (),
        };
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchRx),
            None => Ok(()),
        }
    }
    async fn enable_rf_switch_tx(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_high().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
    async fn disable_rf_switch(&mut self) -> Result<(), RadioError> {
        match &mut self.rf_switch_rx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchRx)?,
            None => (),
        };
        match &mut self.rf_switch_tx {
            Some(pin) => pin.set_low().map_err(|_| RfSwitchTx),
            None => Ok(()),
        }
    }
}
