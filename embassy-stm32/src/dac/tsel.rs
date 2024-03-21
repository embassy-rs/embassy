#![allow(missing_docs)]

/// Trigger selection for STM32F0.
#[cfg(stm32f0)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    Tim3 = 1,
    Tim7 = 2,
    Tim15 = 3,
    Tim2 = 4,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F1.
#[cfg(stm32f1)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    #[cfg(any(stm32f100, stm32f105, stm32f107))]
    Tim3 = 1,
    #[cfg(any(stm32f101, stm32f103))]
    Tim8 = 1,
    Tim7 = 2,
    #[cfg(any(stm32f101, stm32f103, stm32f105, stm32f107))]
    Tim5 = 3,
    #[cfg(all(stm32f100, any(flashsize_4, flashsize_6, flashsize_8, flashsize_b)))]
    Tim15 = 3,
    #[cfg(all(stm32f100, any(flashsize_c, flashsize_d, flashsize_e)))]
    /// Can be remapped to TIM15 with MISC_REMAP in AFIO_MAPR2.
    Tim5Or15 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F2/F4/F7/L4, except F410 or L4+.
#[cfg(all(any(stm32f2, stm32f4, stm32f7, stm32l4_nonplus), not(stm32f410)))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    Tim8 = 1,
    #[cfg(not(any(stm32l45x, stm32l46x)))]
    Tim7 = 2,
    Tim5 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F410.
#[cfg(stm32f410)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim5 = 3,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F301/2 and 318.
#[cfg(any(stm32f301, stm32f302, stm32f318))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    #[cfg(stm32f302)]
    /// Requires DAC_TRIG_RMP set in SYSCFG_CFGR1.
    Tim3 = 1,
    Tim15 = 3,
    Tim2 = 4,
    #[cfg(all(stm32f302, any(flashsize_6, flashsize_8)))]
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F303/3x8 (excluding 318 which is like 301, and 378 which is 37x).
#[cfg(any(stm32f303, stm32f328, stm32f358, stm32f398))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    /// * DAC1: defaults to TIM8 but can be remapped to TIM3 with DAC_TRIG_RMP in SYSCFG_CFGR1
    /// * DAC2: always TIM3
    Tim8Or3 = 1,
    Tim7 = 2,
    Tim15 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F37x.
#[cfg(any(stm32f373, stm32f378))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    Tim3 = 1,
    Tim7 = 2,
    /// TIM5 on DAC1, TIM18 on DAC2
    Dac1Tim5Dac2Tim18 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32F334.
#[cfg(stm32f334)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    /// Requires DAC_TRIG_RMP set in SYSCFG_CFGR1.
    Tim3 = 1,
    Tim7 = 2,
    /// Can be remapped to HRTIM_DACTRG1 using DAC1_TRIG3_RMP in SYSCFG_CFGR3.
    Tim15OrHrtimDacTrg1 = 3,
    Tim2 = 4,
    /// Requires DAC_TRIG5_RMP set in SYSCFG_CFGR3.
    HrtimDacTrg2 = 5,
}

/// Trigger selection for STM32L0.
#[cfg(stm32l0)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    Tim3 = 1,
    Tim3Ch3 = 2,
    Tim21 = 3,
    Tim2 = 4,
    Tim7 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for STM32L1.
#[cfg(stm32l1)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Tim6 = 0,
    Tim7 = 2,
    Tim9 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Software = 7,
}

/// Trigger selection for L4+, L5, U5, H7.
#[cfg(any(stm32l4_plus, stm32l5, stm32u5, stm32h7))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Software = 0,
    Tim1 = 1,
    Tim2 = 2,
    Tim4 = 3,
    Tim5 = 4,
    Tim6 = 5,
    Tim7 = 6,
    Tim8 = 7,
    Tim15 = 8,
    #[cfg(all(stm32h7, hrtim))]
    Hrtim1DacTrg1 = 9,
    #[cfg(all(stm32h7, hrtim))]
    Hrtim1DacTrg2 = 10,
    Lptim1 = 11,
    #[cfg(not(stm32u5))]
    Lptim2 = 12,
    #[cfg(stm32u5)]
    Lptim3 = 12,
    Exti9 = 13,
    #[cfg(any(stm32h7ax, stm32h7bx))]
    /// RM0455 suggests this might be LPTIM2 on DAC1 and LPTIM3 on DAC2,
    /// but it's probably wrong. Please let us know if you find out.
    Lptim3 = 14,
    #[cfg(any(stm32h72x, stm32h73x))]
    Tim23 = 14,
    #[cfg(any(stm32h72x, stm32h73x))]
    Tim24 = 15,
}

/// Trigger selection for H5.
#[cfg(stm32h5)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Software = 0,
    Tim1 = 1,
    Tim2 = 2,
    #[cfg(any(stm32h56x, stm32h57x))]
    Tim4 = 3,
    #[cfg(stm32h503)]
    Tim3 = 3,
    #[cfg(any(stm32h56x, stm32h57x))]
    Tim5 = 4,
    Tim6 = 5,
    Tim7 = 6,
    #[cfg(any(stm32h56x, stm32h57x))]
    Tim8 = 7,
    #[cfg(any(stm32h56x, stm32h57x))]
    Tim15 = 8,
    Lptim1 = 11,
    Lptim2 = 12,
    Exti9 = 13,
}

/// Trigger selection for G0.
#[cfg(stm32g0)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Software = 0,
    Tim1 = 1,
    Tim2 = 2,
    Tim3 = 3,
    Tim6 = 5,
    Tim7 = 6,
    Tim15 = 8,
    Lptim1 = 11,
    Lptim2 = 12,
    Exti9 = 13,
}

/// Trigger selection for G4.
#[cfg(stm32g4)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Software = 0,
    /// * DAC1, DAC2, DAC4: TIM8
    /// * DAC3: TIM1
    Dac124Tim8Dac3Tim1 = 1,
    Tim7 = 2,
    Tim15 = 3,
    Tim2 = 4,
    Tim4 = 5,
    Exti9 = 6,
    Tim6 = 7,
    Tim3 = 8,
    HrtimDacRstTrg1 = 9,
    HrtimDacRstTrg2 = 10,
    HrtimDacRstTrg3 = 11,
    HrtimDacRstTrg4 = 12,
    HrtimDacRstTrg5 = 13,
    HrtimDacRstTrg6 = 14,
    /// * DAC1, DAC4: HRTIM_DAC_TRG1
    /// * DAC2: HRTIM_DAC_TRG2
    /// * DAC3: HRTIM_DAC_TRG3
    HrtimDacTrg123 = 15,
}

/// Trigger selection for WL.
#[cfg(stm32wl)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    Software = 0,
    Tim1 = 1,
    Tim2 = 2,
    Lptim1 = 11,
    Lptim2 = 12,
    Lptim3 = 13,
    Exti9 = 14,
}

impl TriggerSel {
    pub fn tsel(&self) -> u8 {
        *self as u8
    }
}
