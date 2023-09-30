#![macro_use]

pub use defmt::*;
#[allow(unused)]
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "stm32f103c8")]
teleprobe_meta::target!(b"bluepill-stm32f103c8");
#[cfg(feature = "stm32g491re")]
teleprobe_meta::target!(b"nucleo-stm32g491re");
#[cfg(feature = "stm32g071rb")]
teleprobe_meta::target!(b"nucleo-stm32g071rb");
#[cfg(feature = "stm32f429zi")]
teleprobe_meta::target!(b"nucleo-stm32f429zi");
#[cfg(feature = "stm32wb55rg")]
teleprobe_meta::target!(b"nucleo-stm32wb55rg");
#[cfg(feature = "stm32h755zi")]
teleprobe_meta::target!(b"nucleo-stm32h755zi");
#[cfg(feature = "stm32u585ai")]
teleprobe_meta::target!(b"iot-stm32u585ai");
#[cfg(feature = "stm32h563zi")]
teleprobe_meta::target!(b"nucleo-stm32h563zi");
#[cfg(feature = "stm32c031c6")]
teleprobe_meta::target!(b"nucleo-stm32c031c6");
#[cfg(feature = "stm32l073rz")]
teleprobe_meta::target!(b"nucleo-stm32l073rz");
#[cfg(feature = "stm32l152re")]
teleprobe_meta::target!(b"nucleo-stm32l152re");
#[cfg(feature = "stm32l4a6zg")]
teleprobe_meta::target!(b"nucleo-stm32l4a6zg");
#[cfg(feature = "stm32l4r5zi")]
teleprobe_meta::target!(b"nucleo-stm32l4r5zi");
#[cfg(feature = "stm32l552ze")]
teleprobe_meta::target!(b"nucleo-stm32l552ze");

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
                    embassy_stm32::bind_interrupts!(struct Irqs $irq_code);
                    Irqs
                }};
            )*
        }

        #[allow(unused)]
        #[allow(non_camel_case_types)]
        pub mod peris {
            $(
                pub type $name = embassy_stm32::peripherals::$peri;
            )*
        }
    };
}

#[cfg(feature = "stm32f103c8")]
define_peris!(
    UART = USART1, UART_TX = PA9, UART_RX = PA10, UART_TX_DMA = DMA1_CH4, UART_RX_DMA = DMA1_CH5,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH3, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;},
);
#[cfg(feature = "stm32g491re")]
define_peris!(
    UART = USART1, UART_TX = PC4, UART_RX = PC5, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;},
);
#[cfg(feature = "stm32g071rb")]
define_peris!(
    UART = USART1, UART_TX = PC4, UART_RX = PC5, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;},
);
#[cfg(feature = "stm32f429zi")]
define_peris!(
    UART = USART6, UART_TX = PG14, UART_RX = PG9, UART_TX_DMA = DMA2_CH6, UART_RX_DMA = DMA2_CH1,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA2_CH3, SPI_RX_DMA = DMA2_CH2,
    @irq UART = {USART6 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART6>;},
);
#[cfg(feature = "stm32wb55rg")]
define_peris!(
    UART = LPUART1, UART_TX = PA2, UART_RX = PA3, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {LPUART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::LPUART1>;},
);
#[cfg(feature = "stm32h755zi")]
define_peris!(
    UART = USART1, UART_TX = PB6, UART_RX = PB7, UART_TX_DMA = DMA1_CH0, UART_RX_DMA = DMA1_CH1,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PB5, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH0, SPI_RX_DMA = DMA1_CH1,
    @irq UART = {USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;},
);
#[cfg(feature = "stm32u585ai")]
define_peris!(
    UART = USART3, UART_TX = PD8, UART_RX = PD9, UART_TX_DMA = GPDMA1_CH0, UART_RX_DMA = GPDMA1_CH1,
    SPI = SPI1, SPI_SCK = PE13, SPI_MOSI = PE15, SPI_MISO = PE14, SPI_TX_DMA = GPDMA1_CH0, SPI_RX_DMA = GPDMA1_CH1,
    @irq UART = {USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;},
);
#[cfg(feature = "stm32h563zi")]
define_peris!(
    UART = LPUART1, UART_TX = PB6, UART_RX = PB7, UART_TX_DMA = GPDMA1_CH0, UART_RX_DMA = GPDMA1_CH1,
    SPI = SPI4, SPI_SCK = PE12, SPI_MOSI = PE14, SPI_MISO = PE13, SPI_TX_DMA = GPDMA1_CH0, SPI_RX_DMA = GPDMA1_CH1,
    @irq UART = {LPUART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::LPUART1>;},
);
#[cfg(feature = "stm32c031c6")]
define_peris!(
    UART = USART1, UART_TX = PB6, UART_RX = PB7, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;},
);
#[cfg(feature = "stm32l4a6zg")]
define_peris!(
    UART = USART3, UART_TX = PD8, UART_RX = PD9, UART_TX_DMA = DMA1_CH2, UART_RX_DMA = DMA1_CH3,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH3, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;},
);
#[cfg(feature = "stm32l4r5zi")]
define_peris!(
    UART = USART3, UART_TX = PD8, UART_RX = PD9, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;},
);
#[cfg(feature = "stm32l073rz")]
define_peris!(
    UART = USART4, UART_TX = PA0, UART_RX = PA1, UART_TX_DMA = DMA1_CH3, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH3, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART4_5 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART4>;},
);
#[cfg(feature = "stm32l152re")]
define_peris!(
    UART = USART3, UART_TX = PB10, UART_RX = PB11, UART_TX_DMA = DMA1_CH2, UART_RX_DMA = DMA1_CH3,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH3, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;},
);
#[cfg(feature = "stm32l552ze")]
define_peris!(
    UART = USART3, UART_TX = PD8, UART_RX = PD9, UART_TX_DMA = DMA1_CH1, UART_RX_DMA = DMA1_CH2,
    SPI = SPI1, SPI_SCK = PA5, SPI_MOSI = PA7, SPI_MISO = PA6, SPI_TX_DMA = DMA1_CH1, SPI_RX_DMA = DMA1_CH2,
    @irq UART = {USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;},
);

pub fn config() -> Config {
    #[allow(unused_mut)]
    let mut config = Config::default();

    #[cfg(feature = "stm32h755zi")]
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(Hsi::Mhz64);
        config.rcc.csi = true;
        config.rcc.pll_src = PllSource::Hsi;
        config.rcc.pll1 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(2),
            divq: Some(8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.adc_clock_source = AdcClockSource::PLL2_P;
    }

    #[cfg(any(feature = "stm32l4a6zg", feature = "stm32l4r5zi"))]
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::PLL(
            // 72Mhz clock (16 / 1 * 18 / 4)
            PLLSource::HSI16,
            PLLClkDiv::Div4,
            PLLSrcDiv::Div1,
            PLLMul::Mul18,
            Some(PLLClkDiv::Div6), // 48Mhz (16 / 1 * 18 / 6)
        );
    }

    #[cfg(any(feature = "stm32l552ze"))]
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::PLL(
            // 110Mhz clock (16 / 4 * 55 / 2)
            PLLSource::HSI16,
            PLLClkDiv::Div2,
            PLLSrcDiv::Div4,
            PLLMul::Mul55,
            None,
        );
    }

    #[cfg(feature = "stm32u585ai")]
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::MSI(MSIRange::Range48mhz);
    }

    #[cfg(feature = "stm32l073rz")]
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::PLL(
            // 32Mhz clock (16 * 4 / 2)
            PLLSource::HSI16,
            PLLMul::Mul4,
            PLLDiv::Div2,
        );
    }

    #[cfg(any(feature = "stm32l152re"))]
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::PLL(
            // 32Mhz clock (16 * 4 / 2)
            PLLSource::HSI,
            PLLMul::Mul4,
            PLLDiv::Div2,
        );
    }

    config
}
