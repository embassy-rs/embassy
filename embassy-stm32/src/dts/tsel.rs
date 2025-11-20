/// Trigger selection for H5
#[cfg(stm32h5)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    /// Software triggering. Performs continuous measurements.
    Software = 0,
    /// LPTIM1 CH1
    Lptim1 = 1,
    /// LPTIM2 CH1
    Lptim2 = 2,
    /// LPTIM3 CH1
    #[cfg(not(stm32h503))]
    Lptim3 = 3,
    /// EXTI13
    Exti13 = 4,
}

/// Trigger selection for H7, except for H7R and H7S
#[cfg(stm32h7)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    /// Software triggering. Performs continuous measurements.
    Software = 0,
    /// LPTIM1 OUT
    Lptim1 = 1,
    /// LPTIM2 OUT
    Lptim2 = 2,
    /// LPTIM3 OUT
    Lptim3 = 3,
    /// EXTI13
    Exti13 = 4,
}

/// Trigger selection for H7R and H7S
#[cfg(stm32h7rs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    /// Software triggering. Performs continuous measurements.
    Software = 0,
    /// LPTIM4 OUT
    Lptim4 = 1,
    /// LPTIM2 CH1
    Lptim2 = 2,
    /// LPTIM3 CH1
    Lptim3 = 3,
    /// EXTI13
    Exti13 = 4,
}

/// Trigger selection for N6
#[cfg(stm32n6)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSel {
    /// Software triggering. Performs continuous measurements.
    Software = 0,
    /// LPTIM4 OUT
    Lptim4 = 1,
    /// LPTIM2 CH1
    Lptim2 = 2,
    /// LPTIM3 CH1
    Lptim3 = 3,
    /// EXTI13
    Exti13 = 4,
}
