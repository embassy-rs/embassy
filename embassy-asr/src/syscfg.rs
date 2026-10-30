//! System configuration controller.
//!
//! The vendor SDK uses this block for DMA request routing, a software boot-mode
//! flag, and I2S master word-select generation. The PAC currently exposes the
//! SYSCFG registers as raw words, so this module only touches bits exercised by
//! the SDK and preserves every other bit.

use crate::{Peri, pac, peripherals};

const DMA_REQUEST_MASK: u32 = 0x3f;

const BOOT_MODE_FLAG: u32 = 1 << 29;

const I2S_MASTER_ENABLE: u32 = 1 << 14;
// All divisors emitted by the vendor SDK need six bits. Bit 22 is the next
// documented field, but bit 21 is otherwise undocumented and is left alone.
const I2S_WORD_SELECT_DIVIDER_MASK: u32 = 0x3f << 15;
const I2S_WORD_SELECT_ENABLE: u32 = 1 << 22;

/// DMA controller whose request input is being routed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DmaController {
    /// DMA controller 0, routed through SYSCFG CR0.
    Dma0,
    /// DMA controller 1, routed through SYSCFG CR1.
    Dma1,
}

/// Channel within a DMA controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DmaChannel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

impl DmaChannel {
    const fn request_shift(self) -> u32 {
        // The SDK assigns one byte per channel in descending channel order:
        // channel 0 occupies bits 29:24 and channel 3 occupies bits 5:0.
        match self {
            Self::Channel0 => 24,
            Self::Channel1 => 16,
            Self::Channel2 => 8,
            Self::Channel3 => 0,
        }
    }
}

/// Peripheral request routed to a DMA channel.
///
/// Values match `dma_hand_shake_t` in the ASR6601 vendor SDK. The gaps in that
/// table are intentionally not representable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DmaRequest {
    LoracTx = 4,
    LoracRx = 5,
    Dac = 6,
    Adc = 7,
    Scc = 9,
    I2c2Tx = 10,
    I2c2Rx = 11,
    I2c1Tx = 12,
    I2c1Rx = 13,
    I2c0Tx = 14,
    I2c0Rx = 15,
    Ssp2Tx = 16,
    Ssp2Rx = 17,
    Ssp1Tx = 18,
    Ssp1Rx = 19,
    Ssp0Tx = 20,
    Ssp0Rx = 21,
    LpuartTx = 22,
    LpuartRx = 23,
    Uart3Tx = 24,
    Uart3Rx = 25,
    Uart2Tx = 26,
    Uart2Rx = 27,
    Uart1Tx = 28,
    Uart1Rx = 29,
    Uart0Tx = 30,
    Uart0Rx = 31,
    Timer0Channel3 = 32,
    Timer0Channel2 = 33,
    Timer0Channel1 = 34,
    Timer0Channel0 = 35,
    Timer0Trigger = 36,
    Timer0Update = 37,
    Timer1Channel3 = 38,
    Timer1Channel2 = 39,
    Timer1Channel1 = 40,
    Timer1Channel0 = 41,
    Timer1Trigger = 42,
    Timer1Update = 43,
    Timer2Channel1 = 44,
    Timer2Channel0 = 45,
    Timer2Trigger = 46,
    Timer2Update = 47,
    Timer3Channel1 = 48,
    Timer3Channel0 = 49,
    Timer3Trigger = 50,
    Timer3Update = 51,
    BasicTimer1Update = 52,
    BasicTimer0Update = 53,
}

/// I2S sample word size used to derive the vendor word-select divisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2sWordSize {
    /// The DesignWare I2S "don't care" encoding.
    DontCare,
    Bits12,
    Bits16,
    Bits20,
    Bits24,
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
        regs().cr4().read().bits() & BOOT_MODE_FLAG != 0
    }

    /// Set or clear the SDK software boot-mode flag.
    pub fn set_boot_mode_flag(&mut self, set: bool) {
        enable_clock();
        modify_cr4(BOOT_MODE_FLAG, if set { BOOT_MODE_FLAG } else { 0 });
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

fn modify_cr4(mask: u32, value: u32) {
    critical_section::with(|_| {
        regs()
            .cr4()
            .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value & mask)) });
    });
}

fn modify_cr10(mask: u32, value: u32) {
    critical_section::with(|_| {
        regs()
            .cr10()
            .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value & mask)) });
    });
}

pub(crate) fn configure_dma_request(controller: DmaController, channel: DmaChannel, request: DmaRequest) {
    enable_clock();

    let shift = channel.request_shift();
    let mask = DMA_REQUEST_MASK << shift;
    let value = (request as u32) << shift;

    critical_section::with(|_| match controller {
        DmaController::Dma0 => {
            regs()
                .cr0()
                .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | value) });
        }
        DmaController::Dma1 => {
            regs()
                .cr1()
                .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | value) });
        }
    });
}

pub(crate) fn configure_i2s_master(word_size: I2sWordSize) {
    enable_clock();

    let mask = I2S_MASTER_ENABLE | I2S_WORD_SELECT_DIVIDER_MASK;
    let value = I2S_MASTER_ENABLE | (word_size.word_select_divider() << 15);
    modify_cr10(mask, value);
}

pub(crate) fn set_i2s_word_select_output_enabled(enabled: bool) {
    enable_clock();
    modify_cr10(I2S_WORD_SELECT_ENABLE, if enabled { I2S_WORD_SELECT_ENABLE } else { 0 });
}
