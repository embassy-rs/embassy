/// Get the GPIO register block. This is used to configure all GPIO pins.
///
/// # Safety
/// Due to the type system of peripherals, access to the settings of a single pin is possible only
/// by a single thread at a time. Read/Write operations on a single registers are NOT atomic. You
/// must ensure that the GPIO registers are not accessed concurrently by multiple threads.
pub(crate) fn gpio_reg() -> &'static lpc55_pac::gpio::RegisterBlock {
    unsafe { &*lpc55_pac::GPIO::ptr() }
}

/// Get the IOCON register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn iocon_reg() -> &'static lpc55_pac::iocon::RegisterBlock {
    unsafe { &*lpc55_pac::IOCON::ptr() }
}

/// Get the INPUTMUX register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn inputmux_reg() -> &'static lpc55_pac::inputmux::RegisterBlock {
    unsafe { &*lpc55_pac::INPUTMUX::ptr() }
}

/// Get the SYSCON register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn syscon_reg() -> &'static lpc55_pac::syscon::RegisterBlock {
    unsafe { &*lpc55_pac::SYSCON::ptr() }
}

/// Get the PINT register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn pint_reg() -> &'static lpc55_pac::pint::RegisterBlock {
    unsafe { &*lpc55_pac::PINT::ptr() }
}

/// Match the pin bank and number of a pin to the corresponding IOCON register.
///
/// # Example
/// ```
/// use embassy_nxp::gpio::Bank;
/// use embassy_nxp::pac_utils::{iocon_reg, match_iocon};
///
/// // Make pin PIO1_6 digital and set it to pull-down mode.
/// match_iocon!(register, iocon_reg(), Bank::Bank1, 6, {
///     register.modify(|_, w| w.mode().pull_down().digimode().digital());
/// });
/// ```
macro_rules! match_iocon {
    ($register:ident, $iocon_register:expr, $pin_bank:expr, $pin_number:expr, $action:expr) => {
        match ($pin_bank, $pin_number) {
            (Bank::Bank0, 0) => {
                let $register = &($iocon_register).pio0_0;
                $action;
            }
            (Bank::Bank0, 1) => {
                let $register = &($iocon_register).pio0_1;
                $action;
            }
            (Bank::Bank0, 2) => {
                let $register = &($iocon_register).pio0_2;
                $action;
            }
            (Bank::Bank0, 3) => {
                let $register = &($iocon_register).pio0_3;
                $action;
            }
            (Bank::Bank0, 4) => {
                let $register = &($iocon_register).pio0_4;
                $action;
            }
            (Bank::Bank0, 5) => {
                let $register = &($iocon_register).pio0_5;
                $action;
            }
            (Bank::Bank0, 6) => {
                let $register = &($iocon_register).pio0_6;
                $action;
            }
            (Bank::Bank0, 7) => {
                let $register = &($iocon_register).pio0_7;
                $action;
            }
            (Bank::Bank0, 8) => {
                let $register = &($iocon_register).pio0_8;
                $action;
            }
            (Bank::Bank0, 9) => {
                let $register = &($iocon_register).pio0_9;
                $action;
            }
            (Bank::Bank0, 10) => {
                let $register = &($iocon_register).pio0_10;
                $action;
            }
            (Bank::Bank0, 11) => {
                let $register = &($iocon_register).pio0_11;
                $action;
            }
            (Bank::Bank0, 12) => {
                let $register = &($iocon_register).pio0_12;
                $action;
            }
            (Bank::Bank0, 13) => {
                let $register = &($iocon_register).pio0_13;
                $action;
            }
            (Bank::Bank0, 14) => {
                let $register = &($iocon_register).pio0_14;
                $action;
            }
            (Bank::Bank0, 15) => {
                let $register = &($iocon_register).pio0_15;
                $action;
            }
            (Bank::Bank0, 16) => {
                let $register = &($iocon_register).pio0_16;
                $action;
            }
            (Bank::Bank0, 17) => {
                let $register = &($iocon_register).pio0_17;
                $action;
            }
            (Bank::Bank0, 18) => {
                let $register = &($iocon_register).pio0_18;
                $action;
            }
            (Bank::Bank0, 19) => {
                let $register = &($iocon_register).pio0_19;
                $action;
            }
            (Bank::Bank0, 20) => {
                let $register = &($iocon_register).pio0_20;
                $action;
            }
            (Bank::Bank0, 21) => {
                let $register = &($iocon_register).pio0_21;
                $action;
            }
            (Bank::Bank0, 22) => {
                let $register = &($iocon_register).pio0_22;
                $action;
            }
            (Bank::Bank0, 23) => {
                let $register = &($iocon_register).pio0_23;
                $action;
            }
            (Bank::Bank0, 24) => {
                let $register = &($iocon_register).pio0_24;
                $action;
            }
            (Bank::Bank0, 25) => {
                let $register = &($iocon_register).pio0_25;
                $action;
            }
            (Bank::Bank0, 26) => {
                let $register = &($iocon_register).pio0_26;
                $action;
            }
            (Bank::Bank0, 27) => {
                let $register = &($iocon_register).pio0_27;
                $action;
            }
            (Bank::Bank0, 28) => {
                let $register = &($iocon_register).pio0_28;
                $action;
            }
            (Bank::Bank0, 29) => {
                let $register = &($iocon_register).pio0_29;
                $action;
            }
            (Bank::Bank0, 30) => {
                let $register = &($iocon_register).pio0_30;
                $action;
            }
            (Bank::Bank0, 31) => {
                let $register = &($iocon_register).pio0_31;
                $action;
            }
            (Bank::Bank1, 0) => {
                let $register = &($iocon_register).pio1_0;
                $action;
            }
            (Bank::Bank1, 1) => {
                let $register = &($iocon_register).pio1_1;
                $action;
            }
            (Bank::Bank1, 2) => {
                let $register = &($iocon_register).pio1_2;
                $action;
            }
            (Bank::Bank1, 3) => {
                let $register = &($iocon_register).pio1_3;
                $action;
            }
            (Bank::Bank1, 4) => {
                let $register = &($iocon_register).pio1_4;
                $action;
            }
            (Bank::Bank1, 5) => {
                let $register = &($iocon_register).pio1_5;
                $action;
            }
            (Bank::Bank1, 6) => {
                let $register = &($iocon_register).pio1_6;
                $action;
            }
            (Bank::Bank1, 7) => {
                let $register = &($iocon_register).pio1_7;
                $action;
            }
            (Bank::Bank1, 8) => {
                let $register = &($iocon_register).pio1_8;
                $action;
            }
            (Bank::Bank1, 9) => {
                let $register = &($iocon_register).pio1_9;
                $action;
            }
            (Bank::Bank1, 10) => {
                let $register = &($iocon_register).pio1_10;
                $action;
            }
            (Bank::Bank1, 11) => {
                let $register = &($iocon_register).pio1_11;
                $action;
            }
            (Bank::Bank1, 12) => {
                let $register = &($iocon_register).pio1_12;
                $action;
            }
            (Bank::Bank1, 13) => {
                let $register = &($iocon_register).pio1_13;
                $action;
            }
            (Bank::Bank1, 14) => {
                let $register = &($iocon_register).pio1_14;
                $action;
            }
            (Bank::Bank1, 15) => {
                let $register = &($iocon_register).pio1_15;
                $action;
            }
            (Bank::Bank1, 16) => {
                let $register = &($iocon_register).pio1_16;
                $action;
            }
            (Bank::Bank1, 17) => {
                let $register = &($iocon_register).pio1_17;
                $action;
            }
            (Bank::Bank1, 18) => {
                let $register = &($iocon_register).pio1_18;
                $action;
            }
            (Bank::Bank1, 19) => {
                let $register = &($iocon_register).pio1_19;
                $action;
            }
            (Bank::Bank1, 20) => {
                let $register = &($iocon_register).pio1_20;
                $action;
            }
            (Bank::Bank1, 21) => {
                let $register = &($iocon_register).pio1_21;
                $action;
            }
            (Bank::Bank1, 22) => {
                let $register = &($iocon_register).pio1_22;
                $action;
            }
            (Bank::Bank1, 23) => {
                let $register = &($iocon_register).pio1_23;
                $action;
            }
            (Bank::Bank1, 24) => {
                let $register = &($iocon_register).pio1_24;
                $action;
            }
            (Bank::Bank1, 25) => {
                let $register = &($iocon_register).pio1_25;
                $action;
            }
            (Bank::Bank1, 26) => {
                let $register = &($iocon_register).pio1_26;
                $action;
            }
            (Bank::Bank1, 27) => {
                let $register = &($iocon_register).pio1_27;
                $action;
            }
            (Bank::Bank1, 28) => {
                let $register = &($iocon_register).pio1_28;
                $action;
            }
            (Bank::Bank1, 29) => {
                let $register = &($iocon_register).pio1_29;
                $action;
            }
            (Bank::Bank1, 30) => {
                let $register = &($iocon_register).pio1_30;
                $action;
            }
            (Bank::Bank1, 31) => {
                let $register = &($iocon_register).pio1_31;
                $action;
            }
            _ => unreachable!(),
        }
    };
}

pub(crate) use match_iocon;
