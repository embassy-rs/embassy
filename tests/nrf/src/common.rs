#![macro_use]

use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "nrf52832")]
teleprobe_meta::target!(b"nrf52832-dk");
#[cfg(feature = "nrf52840")]
teleprobe_meta::target!(b"nrf52840-dk");
#[cfg(feature = "nrf52833")]
teleprobe_meta::target!(b"nrf52833-dk");
#[cfg(feature = "nrf5340")]
teleprobe_meta::target!(b"nrf5340-dk");
#[cfg(feature = "nrf9160")]
teleprobe_meta::target!(b"nrf9160-dk");
#[cfg(feature = "nrf51422")]
teleprobe_meta::target!(b"nrf51-dk");

macro_rules! define_peris {
    ($($name:ident = $peri:ident,)* $(@irq $irq_name:ident = $irq_code:tt,)*) => {
        #[allow(unused_macros)]
        macro_rules! peri {
            $(
                ($p:expr, $name) => {
                    $p.$peri
                };
            )*
        }
        #[allow(unused_macros)]
        macro_rules! irqs {
            $(
                ($irq_name) => {{
                    embassy_nrf::bind_interrupts!(struct Irqs $irq_code);
                    Irqs
                }};
            )*
            ( @ dummy ) => {};
        }

        #[allow(unused)]
        #[allow(non_camel_case_types)]
        pub mod peris {
            $(
                pub type $name = embassy_nrf::peripherals::$peri;
            )*
        }
    };
}

#[cfg(feature = "nrf51422")]
define_peris!(PIN_A = P0_13, PIN_B = P0_14,);

#[cfg(feature = "nrf52832")]
define_peris!(PIN_A = P0_11, PIN_B = P0_12,);

#[cfg(feature = "nrf52833")]
define_peris!(PIN_A = P1_01, PIN_B = P1_02,);

#[cfg(feature = "nrf52840")]
define_peris!(PIN_A = P1_02, PIN_B = P1_03,);

#[cfg(feature = "nrf5340")]
define_peris!(PIN_A = P1_00, PIN_B = P1_01,);

#[cfg(feature = "nrf9160")]
define_peris!(PIN_A = P0_00, PIN_B = P0_01,);
