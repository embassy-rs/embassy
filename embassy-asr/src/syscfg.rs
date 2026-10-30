//! System configuration controller.
//!
//! The vendor SDK uses this block for DMA request routing, a software boot-mode
//! flag, and I2S master word-select generation. DMA routing, the CR4 boot-mode
//! flag, and I2S go through PAC field accessors.

use crate::{Peri, pac, peripherals};

// All divisors emitted by the vendor SDK need six bits. Bit 22 is the next
// documented field, but bit 21 is otherwise undocumented and is left alone.

/// DMA controller whose request input is being routed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DmaController {
    /// DMA controller 0, routed through SYSCFG CR0.
    Dma0,
    /// DMA controller 1, routed through SYSCFG CR1.
    Dma1,
}

/// Channel within a DMA controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DmaChannel {
    /// DMA channel 0.
    Channel0,
    /// DMA channel 1.
    Channel1,
    /// DMA channel 2.
    Channel2,
    /// DMA channel 3.
    Channel3,
}

/// Peripheral request routed to a DMA channel.
///
/// Values match `dma_hand_shake_t` in the ASR6601 vendor SDK. The gaps in that
/// table are intentionally not representable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[non_exhaustive]
pub enum DmaRequest {
    /// LoRa controller transmit DMA request.
    LoracTx = 4,
    /// LoRa controller receive DMA request.
    LoracRx = 5,
    /// DAC DMA request.
    Dac = 6,
    /// ADC DMA request.
    Adc = 7,
    /// Security coprocessor DMA request.
    Scc = 9,
    /// I2C2 transmit DMA request.
    I2c2Tx = 10,
    /// I2C2 receive DMA request.
    I2c2Rx = 11,
    /// I2C1 transmit DMA request.
    I2c1Tx = 12,
    /// I2C1 receive DMA request.
    I2c1Rx = 13,
    /// I2C0 transmit DMA request.
    I2c0Tx = 14,
    /// I2C0 receive DMA request.
    I2c0Rx = 15,
    /// SSP2 transmit DMA request.
    Ssp2Tx = 16,
    /// SSP2 receive DMA request.
    Ssp2Rx = 17,
    /// SSP1 transmit DMA request.
    Ssp1Tx = 18,
    /// SSP1 receive DMA request.
    Ssp1Rx = 19,
    /// SSP0 transmit DMA request.
    Ssp0Tx = 20,
    /// SSP0 receive DMA request.
    Ssp0Rx = 21,
    /// LPUART transmit DMA request.
    LpuartTx = 22,
    /// LPUART receive DMA request.
    LpuartRx = 23,
    /// UART3 transmit DMA request.
    Uart3Tx = 24,
    /// UART3 receive DMA request.
    Uart3Rx = 25,
    /// UART2 transmit DMA request.
    Uart2Tx = 26,
    /// UART2 receive DMA request.
    Uart2Rx = 27,
    /// UART1 transmit DMA request.
    Uart1Tx = 28,
    /// UART1 receive DMA request.
    Uart1Rx = 29,
    /// UART0 transmit DMA request.
    Uart0Tx = 30,
    /// UART0 receive DMA request.
    Uart0Rx = 31,
    /// Timer0 channel 3 DMA request.
    Timer0Channel3 = 32,
    /// Timer0 channel 2 DMA request.
    Timer0Channel2 = 33,
    /// Timer0 channel 1 DMA request.
    Timer0Channel1 = 34,
    /// Timer0 channel 0 DMA request.
    Timer0Channel0 = 35,
    /// Timer0 trigger DMA request.
    Timer0Trigger = 36,
    /// Timer0 update DMA request.
    Timer0Update = 37,
    /// Timer1 channel 3 DMA request.
    Timer1Channel3 = 38,
    /// Timer1 channel 2 DMA request.
    Timer1Channel2 = 39,
    /// Timer1 channel 1 DMA request.
    Timer1Channel1 = 40,
    /// Timer1 channel 0 DMA request.
    Timer1Channel0 = 41,
    /// Timer1 trigger DMA request.
    Timer1Trigger = 42,
    /// Timer1 update DMA request.
    Timer1Update = 43,
    /// Timer2 channel 1 DMA request.
    Timer2Channel1 = 44,
    /// Timer2 channel 0 DMA request.
    Timer2Channel0 = 45,
    /// Timer2 trigger DMA request.
    Timer2Trigger = 46,
    /// Timer2 update DMA request.
    Timer2Update = 47,
    /// Timer3 channel 1 DMA request.
    Timer3Channel1 = 48,
    /// Timer3 channel 0 DMA request.
    Timer3Channel0 = 49,
    /// Timer3 trigger DMA request.
    Timer3Trigger = 50,
    /// Timer3 update DMA request.
    Timer3Update = 51,
    /// basic timer 1 update DMA request.
    BasicTimer1Update = 52,
    /// basic timer 0 update DMA request.
    BasicTimer0Update = 53,
}

/// I2S sample word size used to derive the vendor word-select divisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum I2sWordSize {
    /// The DesignWare I2S "don't care" encoding.
    DontCare,
    /// 12-bit samples.
    Bits12,
    /// 16-bit samples.
    Bits16,
    /// 20-bit samples.
    Bits20,
    /// 24-bit samples.
    Bits24,
    /// 32-bit samples.
    Bits32,
}

impl I2sWordSize {
    const fn word_select_divider(self) -> u32 {
        // These are the exact results of the SDK's i2s_calculate_devision().
        match self {
            Self::DontCare | Self::Bits20 => 21,
            Self::Bits12 => 13,
            Self::Bits16 => 17,
            Self::Bits24 => 25,
            Self::Bits32 => 33,
        }
    }
}

/// Owned access to the system configuration controller.
///
/// Dropping this value does not gate the SYSCFG clock because DMA and I2S
/// drivers may still depend on routes configured here.
pub struct Syscfg<'d> {
    _peri: Peri<'d, peripherals::SYSCFG>,
}

impl<'d> Syscfg<'d> {
    /// Create an owned SYSCFG controller and enable its peripheral clock.
    pub fn new(peri: Peri<'d, peripherals::SYSCFG>) -> Self {
        enable_clock();
        Self { _peri: peri }
    }

    /// Route a peripheral request to one DMA controller channel.
    ///
    /// This replaces only the selected channel's six-bit request field.
    pub fn route_dma_request(&mut self, controller: DmaController, channel: DmaChannel, request: DmaRequest) {
        configure_dma_request(controller, channel, request);
    }

    /// Return the SDK software boot-mode flag.
    ///
    /// The vendor OTA bootloader uses CR4 bit 29 as a software marker. No
    /// reset-retention behavior is implied by this API.
    pub fn boot_mode_flag(&self) -> bool {
        enable_clock();
        regs().cr4().read().boot_mode().bit_is_set()
    }

    /// Set or clear the SDK software boot-mode flag.
    pub fn set_boot_mode_flag(&mut self, set: bool) {
        enable_clock();
        critical_section::with(|_| regs().cr4().modify(|_, w| w.boot_mode().bit(set)));
    }

    /// Configure the I2S master word-select divisor for a sample word size.
    ///
    /// This enables I2S master generation but does not enable the word-select
    /// output. The I2S driver can enable that output after its data block is
    /// ready, matching the vendor initialization order.
    pub fn configure_i2s_master(&mut self, word_size: I2sWordSize) {
        configure_i2s_master(word_size);
    }

    /// Enable or disable the I2S master word-select output.
    pub fn set_i2s_word_select_output_enabled(&mut self, enabled: bool) {
        set_i2s_word_select_output_enabled(enabled);
    }
}

fn regs() -> pac::Syscfg {
    // Register access is kept inside this module. All read-modify-write
    // sequences are serialized below, and the HAL ownership token remains the
    // public way to perform stand-alone SYSCFG configuration.
    unsafe { pac::Syscfg::steal() }
}

fn enable_clock() {
    critical_section::with(|_| {
        // The vendor's rcc_enable_peripheral_clk(SYSCFG, true) operation only
        // sets this gate and requires no synchronization wait.
        unsafe { pac::Rcc::steal() }
            .cgr0()
            .modify(|_, w| w.syscfg_clk_en().set_bit());
    });
}

pub(crate) fn configure_dma_request(controller: DmaController, channel: DmaChannel, request: DmaRequest) {
    enable_clock();

    // Request numbers are the `dma_hand_shake_t` values (no PAC enum);
    // field positions come from the generated handshake selectors.
    let value = request as u8;
    critical_section::with(|_| unsafe {
        match (controller, channel) {
            (DmaController::Dma0, DmaChannel::Channel0) => {
                regs().cr0().modify(|_, w| w.dmac0_handshake0_sel().bits(value))
            }
            (DmaController::Dma0, DmaChannel::Channel1) => {
                regs().cr0().modify(|_, w| w.dmac0_handshake1_sel().bits(value))
            }
            (DmaController::Dma0, DmaChannel::Channel2) => {
                regs().cr0().modify(|_, w| w.dmac0_handshake2_sel().bits(value))
            }
            (DmaController::Dma0, DmaChannel::Channel3) => {
                regs().cr0().modify(|_, w| w.dmac0_handshake3_sel().bits(value))
            }
            (DmaController::Dma1, DmaChannel::Channel0) => {
                regs().cr1().modify(|_, w| w.dmac1_handshake0_sel().bits(value))
            }
            (DmaController::Dma1, DmaChannel::Channel1) => {
                regs().cr1().modify(|_, w| w.dmac1_handshake1_sel().bits(value))
            }
            (DmaController::Dma1, DmaChannel::Channel2) => {
                regs().cr1().modify(|_, w| w.dmac1_handshake2_sel().bits(value))
            }
            (DmaController::Dma1, DmaChannel::Channel3) => {
                regs().cr1().modify(|_, w| w.dmac1_handshake3_sel().bits(value))
            }
        }
    });
}

pub(crate) fn configure_i2s_master(word_size: I2sWordSize) {
    enable_clock();

    // Master mode plus the word-select divider derived from the sample word
    // size (exact SDK `i2s_calculate_devision()` results). Preserve unrelated
    // CR10 fields (QSPI remap, WS select).
    critical_section::with(|_| {
        regs().cr10().modify(|_, w| {
            w.i2s_mode_sel().master();
            unsafe { w.i2s_ws_len().bits(word_size.word_select_divider() as u8) }
        });
    });
}

pub(crate) fn set_i2s_word_select_output_enabled(enabled: bool) {
    enable_clock();
    critical_section::with(|_| {
        regs().cr10().modify(|_, w| w.i2s_ws_en().bit(enabled));
    });
}
