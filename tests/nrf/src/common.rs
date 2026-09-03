#![macro_use]

use defmt_rtt as _;
use panic_probe as _;

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
#[cfg(feature = "nrf54l15")]
teleprobe_meta::target!(b"nrf54l15-dk");

macro_rules! define_peris {
    ($($name:ident = $peri:ident,)* $(@pac $pac_name:ident = $pac_peri:ident,)* $(@irq $irq_name:ident = $irq_code:tt,)*) => {
        #[allow(unused_macros)]
        macro_rules! peri {
            $(
                ($p:expr, $name) => {
                    $p.$peri
                };
            )*
        }
        #[allow(unused_macros)]
        macro_rules! peri_pac {
            $(
                ($pac_name) => {
                    embassy_nrf::pac::$pac_peri
                };
            )*
            ( @ dummy ) => {};
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
define_peris!(
    PIN_A = P0_11, PIN_B = P0_12,
    PIN_X = P0_13,
    UART0 = UARTE0,
    SPIM0 = TWISPI0,
    @irq UART0 = {UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq UART0_BUFFERED = {UARTE0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq SPIM0 = {TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;},
);

#[cfg(feature = "nrf52833")]
define_peris!(
    PIN_A = P1_01, PIN_B = P1_02,
    PIN_X = P1_03,
    UART0 = UARTE0,
    UART1 = UARTE1,
    PPI_CH2 = PPI_CH2,
    SPIM0 = TWISPI0,
    @pac UART1 = UARTE1,
    @irq UART0 = {UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq UART1 = {UARTE1 => uarte::InterruptHandler<peripherals::UARTE1>;},
    @irq UART0_BUFFERED = {UARTE0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq UART1_BUFFERED = {UARTE1 => buffered_uarte::InterruptHandler<peripherals::UARTE1>;},
    @irq SPIM0 = {TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;},
);

#[cfg(feature = "nrf52840")]
define_peris!(
    RNG = CC_RNG,
    PIN_A = P1_02, PIN_B = P1_03,
    PIN_X = P1_04,
    UART0 = UARTE0,
    UART1 = UARTE1,
    PPI_CH2 = PPI_CH2,
    SPIM0 = TWISPI0,
    @pac UART1 = UARTE1,
    @irq UART0 = {UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq UART1 = {UARTE1 => uarte::InterruptHandler<peripherals::UARTE1>;},
    @irq UART0_BUFFERED = {UARTE0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;},
    @irq UART1_BUFFERED = {UARTE1 => buffered_uarte::InterruptHandler<peripherals::UARTE1>;},
    @irq SPIM0 = {TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;},
);

#[cfg(feature = "nrf5340")]
define_peris!(
    RNG = CC_RNG,
    PIN_A = P1_08, PIN_B = P1_09,
    PIN_X = P1_10,
    UART0 = SERIAL0,
    UART1 = SERIAL1,
    PPI_CH2 = PPI_CH2,
    SPIM0 = SERIAL0,
    @pac UART1 = UARTE1,
    @irq UART0 = {SERIAL0 => uarte::InterruptHandler<peripherals::SERIAL0>;},
    @irq UART1 = {SERIAL1 => uarte::InterruptHandler<peripherals::SERIAL1>;},
    @irq UART0_BUFFERED = {SERIAL0 => buffered_uarte::InterruptHandler<peripherals::SERIAL0>;},
    @irq UART1_BUFFERED = {SERIAL1 => buffered_uarte::InterruptHandler<peripherals::SERIAL1>;},
    @irq SPIM0 = {SERIAL0 => spim::InterruptHandler<peripherals::SERIAL0>;},
);

#[cfg(feature = "nrf9160")]
define_peris!(
    RNG = CC_RNG,
    PIN_A = P0_00, PIN_B = P0_01,
    PIN_X = P0_02,
    UART0 = SERIAL0,
    UART1 = SERIAL1,
    PPI_CH2 = PPI_CH2,
    SPIM0 = SERIAL0,
    @pac UART1 = UARTE1,
    @irq UART0 = {SERIAL0 => uarte::InterruptHandler<peripherals::SERIAL0>;},
    @irq UART1 = {SERIAL1 => uarte::InterruptHandler<peripherals::SERIAL1>;},
    @irq UART0_BUFFERED = {SERIAL0 => buffered_uarte::InterruptHandler<peripherals::SERIAL0>;},
    @irq UART1_BUFFERED = {SERIAL1 => buffered_uarte::InterruptHandler<peripherals::SERIAL1>;},
    @irq SPIM0 = {SERIAL0 => spim::InterruptHandler<peripherals::SERIAL0>;},
);

// PIN_A and PIN_B must be wired together on the board.
#[cfg(feature = "nrf54l15")]
define_peris!(
    RNG = CRACEN,
    PIN_A = P1_11, PIN_B = P1_12,
    PIN_X = P1_13,
    UART0 = SERIAL21,
    UART1 = SERIAL22,
    SPIM0 = SERIAL21,
    PPI_CH2 = PPI20_CH0,
    @pac UART1 = UARTE22,
    @irq UART0 = {SERIAL21 => uarte::InterruptHandler<peripherals::SERIAL21>;},
    @irq UART1 = {SERIAL22 => uarte::InterruptHandler<peripherals::SERIAL22>;},
    @irq UART0_BUFFERED = {SERIAL21 => buffered_uarte::InterruptHandler<peripherals::SERIAL21>;},
    @irq UART1_BUFFERED = {SERIAL22 => buffered_uarte::InterruptHandler<peripherals::SERIAL22>;},
    @irq SPIM0 = {SERIAL21 => spim::InterruptHandler<peripherals::SERIAL21>;},
);

// The nRF54L `BufferedUarte` doesn't need a timer and PPI channels to count received bytes,
// it uses the UARTE's own frame timeout instead.
#[cfg(not(feature = "nrf54l15"))]
#[allow(unused_macros)]
macro_rules! buffered_uarte_new {
    ($p:ident, $config:expr, $rx_buffer:expr, $tx_buffer:expr) => {
        BufferedUarte::new(
            peri!($p, UART0).reborrow(),
            $p.TIMER0.reborrow(),
            $p.PPI_CH0.reborrow(),
            $p.PPI_CH1.reborrow(),
            $p.PPI_GROUP0.reborrow(),
            peri!($p, PIN_A).reborrow(),
            peri!($p, PIN_B).reborrow(),
            irqs!(UART0_BUFFERED),
            $config,
            $rx_buffer,
            $tx_buffer,
        )
    };
}

#[cfg(feature = "nrf54l15")]
#[allow(unused_macros)]
macro_rules! buffered_uarte_new {
    ($p:ident, $config:expr, $rx_buffer:expr, $tx_buffer:expr) => {
        BufferedUarte::new(
            peri!($p, UART0).reborrow(),
            peri!($p, PIN_A).reborrow(),
            peri!($p, PIN_B).reborrow(),
            irqs!(UART0_BUFFERED),
            $config,
            $rx_buffer,
            $tx_buffer,
        )
    };
}

#[cfg(not(feature = "nrf54l15"))]
#[allow(unused_macros)]
macro_rules! buffered_uarte_rx_new {
    ($p:ident, $rxd:expr, $config:expr, $rx_buffer:expr) => {
        BufferedUarteRx::new(
            peri!($p, UART0).reborrow(),
            $p.TIMER0.reborrow(),
            $p.PPI_CH0.reborrow(),
            $p.PPI_CH1.reborrow(),
            $p.PPI_GROUP0.reborrow(),
            irqs!(UART0_BUFFERED),
            $rxd,
            $config,
            $rx_buffer,
        )
    };
}

#[cfg(feature = "nrf54l15")]
#[allow(unused_macros)]
macro_rules! buffered_uarte_rx_new {
    ($p:ident, $rxd:expr, $config:expr, $rx_buffer:expr) => {
        BufferedUarteRx::new(
            peri!($p, UART0).reborrow(),
            irqs!(UART0_BUFFERED),
            $rxd,
            $config,
            $rx_buffer,
        )
    };
}
