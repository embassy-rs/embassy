// required-features: afio
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfioRemap, OutputType, Pull};
use embassy_stm32::pac::AFIO;
use embassy_stm32::time::khz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::pwm_input::PwmInput;
use embassy_stm32::timer::qei::Qei;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Ch1, Ch2};
use embassy_stm32::usart::{Uart, UartRx, UartTx};
use embassy_stm32::{Peripherals, bind_interrupts};

#[cfg(not(feature = "afio-connectivity-line"))]
bind_interrupts!(struct Irqs {
    USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
    TIM1_CC => embassy_stm32::timer::CaptureCompareInterruptHandler<embassy_stm32::peripherals::TIM1>;

    DMA1_CHANNEL2 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>;
    DMA1_CHANNEL3 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH3>;
});

#[cfg(feature = "afio-connectivity-line")]
bind_interrupts!(struct Irqs {
    CAN1_RX0 => embassy_stm32::can::Rx0InterruptHandler<embassy_stm32::peripherals::CAN1>;
    CAN1_RX1 => embassy_stm32::can::Rx1InterruptHandler<embassy_stm32::peripherals::CAN1>;
    CAN1_SCE => embassy_stm32::can::SceInterruptHandler<embassy_stm32::peripherals::CAN1>;
    CAN1_TX => embassy_stm32::can::TxInterruptHandler<embassy_stm32::peripherals::CAN1>;

    CAN2_RX0 => embassy_stm32::can::Rx0InterruptHandler<embassy_stm32::peripherals::CAN2>;
    CAN2_RX1 => embassy_stm32::can::Rx1InterruptHandler<embassy_stm32::peripherals::CAN2>;
    CAN2_SCE => embassy_stm32::can::SceInterruptHandler<embassy_stm32::peripherals::CAN2>;
    CAN2_TX => embassy_stm32::can::TxInterruptHandler<embassy_stm32::peripherals::CAN2>;

    ETH => embassy_stm32::eth::InterruptHandler;
    USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
    TIM1_CC => embassy_stm32::timer::CaptureCompareInterruptHandler<embassy_stm32::peripherals::TIM1>;

    DMA1_CHANNEL2 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>;
    DMA1_CHANNEL3 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH3>;
    DMA2_CHANNEL1 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA2_CH1>;
    DMA2_CHANNEL2 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA2_CH2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = init();
    info!("Hello World!");

    // USART3
    {
        // no remap RX/TX/RTS/CTS
        afio_registers_set_remap();
        Uart::new_blocking_with_rtscts(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.PB10.reborrow(),
            p.PB14.reborrow(),
            p.PB13.reborrow(),
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap RX/TX
        afio_registers_set_remap();
        Uart::new_blocking(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.PB10.reborrow(),
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap TX
        afio_registers_set_remap();
        Uart::new_blocking_half_duplex(
            p.USART3.reborrow(),
            p.PB10.reborrow(),
            Default::default(),
            embassy_stm32::usart::HalfDuplexReadback::NoReadback,
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap TX async
        afio_registers_set_remap();
        UartTx::new(
            p.USART3.reborrow(),
            p.PB10.reborrow(),
            p.DMA1_CH2.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap TX/CTS async
        afio_registers_set_remap();
        UartTx::new_with_cts(
            p.USART3.reborrow(),
            p.PB10.reborrow(),
            p.PB13.reborrow(),
            p.DMA1_CH2.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap RX async
        afio_registers_set_remap();
        UartRx::new(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.DMA1_CH3.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap RX async
        afio_registers_set_remap();
        UartRx::new_with_rts(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.PB14.reborrow(),
            p.DMA1_CH3.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap RX/TX async
        afio_registers_set_remap();
        Uart::new(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.PB10.reborrow(),
            p.DMA1_CH2.reborrow(),
            p.DMA1_CH3.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }
    {
        // no remap RX/TX/RTS/CTS async
        afio_registers_set_remap();
        Uart::new_with_rtscts(
            p.USART3.reborrow(),
            p.PB11.reborrow(),
            p.PB10.reborrow(),
            p.PB14.reborrow(),
            p.PB13.reborrow(),
            p.DMA1_CH2.reborrow(),
            p.DMA1_CH3.reborrow(),
            Irqs,
            Default::default(),
        )
        .unwrap();
        defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 0);
    }

    // TIM1
    {
        // no remap
        afio_registers_set_remap();
        SimplePwm::new::<AfioRemap<0>>(
            p.TIM1.reborrow(),
            Some(PwmPin::new(p.PA8.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA9.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA10.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA11.reborrow(), OutputType::PushPull)),
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 0);
    }
    {
        // no remap
        afio_registers_set_remap();
        SimplePwm::new::<AfioRemap<0>>(
            p.TIM1.reborrow(),
            Some(PwmPin::new(p.PA8.reborrow(), OutputType::PushPull)),
            None,
            None,
            None,
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 0);
    }
    {
        // partial remap
        reset_afio_registers();
        ComplementaryPwm::new::<AfioRemap<1>>(
            p.TIM1.reborrow(),
            Some(PwmPin::new(p.PA8.reborrow(), OutputType::PushPull)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 1);
    }
    {
        // partial remap
        reset_afio_registers();
        ComplementaryPwm::new::<AfioRemap<1>>(
            p.TIM1.reborrow(),
            Some(PwmPin::new(p.PA8.reborrow(), OutputType::PushPull)),
            Some(ComplementaryPwmPin::new(p.PA7.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA9.reborrow(), OutputType::PushPull)),
            Some(ComplementaryPwmPin::new(p.PB0.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA10.reborrow(), OutputType::PushPull)),
            None, // pin does not exist on medium-density devices
            Some(PwmPin::new(p.PA11.reborrow(), OutputType::PushPull)),
            None, // signal does not exist
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 1);
    }
    {
        // partial remap
        reset_afio_registers();
        InputCapture::new::<AfioRemap<1>>(
            p.TIM1.reborrow(),
            Some(CapturePin::new(p.PA8.reborrow(), Pull::Down)),
            None,
            None,
            None,
            Irqs,
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 1);
    }
    {
        // partial remap
        reset_afio_registers();
        PwmInput::new_ch1::<AfioRemap<1>>(p.TIM1.reborrow(), p.PA8.reborrow(), Irqs, Pull::Down, khz(10));
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 1);
    }
    {
        // partial remap
        reset_afio_registers();
        Qei::new::<Ch1, Ch2, AfioRemap<1>>(
            p.TIM1.reborrow(),
            p.PA8.reborrow(),
            p.PA9.reborrow(),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 1);
    }

    // TIM2
    {
        // no remap
        afio_registers_set_remap();
        SimplePwm::new::<AfioRemap<0>>(
            p.TIM2.reborrow(),
            Some(PwmPin::new(p.PA0.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA1.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA2.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA3.reborrow(), OutputType::PushPull)),
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim2_remap(), 0);
    }
    {
        // partial remap 1
        reset_afio_registers();
        SimplePwm::new::<AfioRemap<1>>(
            p.TIM2.reborrow(),
            Some(PwmPin::new(p.PA15.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB3.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA2.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA3.reborrow(), OutputType::PushPull)),
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim2_remap(), 1);
    }
    {
        // partial remap 2
        reset_afio_registers();
        SimplePwm::new::<AfioRemap<2>>(
            p.TIM2.reborrow(),
            Some(PwmPin::new(p.PA0.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PA1.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB10.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB11.reborrow(), OutputType::PushPull)),
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim2_remap(), 2);
    }
    {
        // full remap
        reset_afio_registers();
        SimplePwm::new::<AfioRemap<3>>(
            p.TIM2.reborrow(),
            Some(PwmPin::new(p.PA15.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB3.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB10.reborrow(), OutputType::PushPull)),
            Some(PwmPin::new(p.PB11.reborrow(), OutputType::PushPull)),
            khz(10),
            Default::default(),
        );
        defmt::assert_eq!(AFIO.mapr().read().tim2_remap(), 3);
    }

    connectivity_line::run(&mut p);
    value_line::run(&mut p);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[cfg(feature = "afio-connectivity-line")]
mod connectivity_line {
    use embassy_stm32::can::Can;
    use embassy_stm32::eth::{Ethernet, PacketQueue};
    use embassy_stm32::i2s::I2S;
    use embassy_stm32::spi::Spi;

    use super::*;

    pub fn run(p: &mut Peripherals) {
        // USART3
        {
            // partial remap RX/TX/RTS/CTS
            reset_afio_registers();
            Uart::new_blocking_with_rtscts(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.PC10.reborrow(),
                p.PB14.reborrow(),
                p.PB13.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap RX/TX
            reset_afio_registers();
            Uart::new_blocking(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.PC10.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap TX
            reset_afio_registers();
            Uart::new_blocking_half_duplex(
                p.USART3.reborrow(),
                p.PC10.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap TX async
            reset_afio_registers();
            UartTx::new(
                p.USART3.reborrow(),
                p.PC10.reborrow(),
                p.DMA1_CH2.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap TX/CTS async
            reset_afio_registers();
            UartTx::new_with_cts(
                p.USART3.reborrow(),
                p.PC10.reborrow(),
                p.PB13.reborrow(),
                p.DMA1_CH2.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap RX async
            reset_afio_registers();
            UartRx::new(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap RX async
            reset_afio_registers();
            UartRx::new_with_rts(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.PB14.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap RX/TX async
            reset_afio_registers();
            Uart::new(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.PC10.reborrow(),
                p.DMA1_CH2.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // partial remap RX/TX/RTS/CTS async
            reset_afio_registers();
            Uart::new_with_rtscts(
                p.USART3.reborrow(),
                p.PC11.reborrow(),
                p.PC10.reborrow(),
                p.PB14.reborrow(),
                p.PB13.reborrow(),
                p.DMA1_CH2.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 1);
        }
        {
            // full remap RX/TX/RTS/CTS
            reset_afio_registers();
            Uart::new_blocking_with_rtscts(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.PD8.reborrow(),
                p.PD12.reborrow(),
                p.PD11.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap RX/TX
            reset_afio_registers();
            Uart::new_blocking(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.PD8.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap TX
            reset_afio_registers();
            Uart::new_blocking_half_duplex(
                p.USART3.reborrow(),
                p.PD8.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap TX async
            reset_afio_registers();
            UartTx::new(
                p.USART3.reborrow(),
                p.PD8.reborrow(),
                p.DMA1_CH2.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap TX/CTS async
            reset_afio_registers();
            UartTx::new_with_cts(
                p.USART3.reborrow(),
                p.PD8.reborrow(),
                p.PD11.reborrow(),
                p.DMA1_CH2.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap RX async
            reset_afio_registers();
            UartRx::new(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap RX async
            reset_afio_registers();
            UartRx::new_with_rts(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.PD12.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap RX/TX async
            reset_afio_registers();
            Uart::new(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.PD8.reborrow(),
                p.DMA1_CH2.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }
        {
            // full remap RX/TX/RTS/CTS async
            reset_afio_registers();
            Uart::new_with_rtscts(
                p.USART3.reborrow(),
                p.PD9.reborrow(),
                p.PD8.reborrow(),
                p.PD12.reborrow(),
                p.PD11.reborrow(),
                p.DMA1_CH2.reborrow(),
                p.DMA1_CH3.reborrow(),
                Irqs,
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart3_remap(), 3);
        }

        // SPI3
        {
            // no remap SCK/MISO/MOSI
            afio_registers_set_remap();
            Spi::new_blocking(
                p.SPI3.reborrow(),
                p.PB3.reborrow(),
                p.PB5.reborrow(),
                p.PB4.reborrow(),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // no remap SCK/MOSI
            afio_registers_set_remap();
            Spi::new_blocking_txonly(
                p.SPI3.reborrow(),
                p.PB3.reborrow(),
                p.PB5.reborrow(),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // no remap MOSI
            afio_registers_set_remap();
            Spi::new_blocking_txonly_nosck(p.SPI3.reborrow(), p.PB5.reborrow(), Default::default());
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // no remap SCK/MISO
            afio_registers_set_remap();
            Spi::new_blocking_rxonly(
                p.SPI3.reborrow(),
                p.PB3.reborrow(),
                p.PB4.reborrow(),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // remap SCK/MISO/MOSI
            reset_afio_registers();
            Spi::new_blocking(
                p.SPI3.reborrow(),
                p.PC10.reborrow(),
                p.PC12.reborrow(),
                p.PC11.reborrow(),
                Default::default(),
            );

            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap SCK/MOSI
            reset_afio_registers();
            Spi::new_blocking_txonly(
                p.SPI3.reborrow(),
                p.PC10.reborrow(),
                p.PC12.reborrow(),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap MOSI
            reset_afio_registers();
            Spi::new_blocking_txonly_nosck(p.SPI3.reborrow(), p.PB5.reborrow(), Default::default());
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap SCK/MISO
            reset_afio_registers();
            Spi::new_blocking_rxonly(
                p.SPI3.reborrow(),
                p.PC10.reborrow(),
                p.PC11.reborrow(),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }

        // I2S3
        {
            // no remap SD/WS/CK/MCK
            afio_registers_set_remap();
            I2S::new_txonly(
                p.SPI3.reborrow(),
                p.PB5.reborrow(),
                p.PA15.reborrow(),
                p.PB3.reborrow(),
                p.PC7.reborrow(),
                p.DMA2_CH2.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // no remap SD/WS/CK
            afio_registers_set_remap();
            I2S::new_txonly_nomck(
                p.SPI3.reborrow(),
                p.PB5.reborrow(),
                p.PA15.reborrow(),
                p.PB3.reborrow(),
                p.DMA2_CH2.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), false);
        }
        {
            // no remap SD/WS/CK/MCK
            afio_registers_set_remap();
            I2S::new_rxonly(
                p.SPI3.reborrow(),
                p.PB4.reborrow(),
                p.PA15.reborrow(),
                p.PB3.reborrow(),
                p.PC7.reborrow(),
                p.DMA2_CH1.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap SD/WS/CK/MCK
            reset_afio_registers();
            I2S::new_txonly(
                p.SPI3.reborrow(),
                p.PC12.reborrow(),
                p.PA4.reborrow(),
                p.PC10.reborrow(),
                p.PC7.reborrow(),
                p.DMA2_CH2.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap SD/WS/CK
            reset_afio_registers();
            I2S::new_txonly_nomck(
                p.SPI3.reborrow(),
                p.PC12.reborrow(),
                p.PA4.reborrow(),
                p.PC10.reborrow(),
                p.DMA2_CH2.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }
        {
            // remap SD/WS/CK/MCK
            reset_afio_registers();
            I2S::new_rxonly(
                p.SPI3.reborrow(),
                p.PC11.reborrow(),
                p.PA4.reborrow(),
                p.PC10.reborrow(),
                p.PC7.reborrow(),
                p.DMA2_CH1.reborrow(),
                &mut [0u16; 0],
                Irqs,
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().spi3_remap(), true);
        }

        // CAN2
        {
            // no remap
            afio_registers_set_remap();
            Can::new(p.CAN2.reborrow(), p.PB12.reborrow(), p.PB13.reborrow(), Irqs);
            defmt::assert_eq!(AFIO.mapr().read().can2_remap(), false);
        }
        {
            // remap
            reset_afio_registers();
            Can::new(p.CAN2.reborrow(), p.PB5.reborrow(), p.PB6.reborrow(), Irqs);
            defmt::assert_eq!(AFIO.mapr().read().can2_remap(), true);
        }

        // Ethernet
        {
            // no remap RMII
            afio_registers_set_remap();
            Ethernet::new(
                &mut PacketQueue::<1, 1>::new(),
                p.ETH.reborrow(),
                Irqs,
                p.PA1.reborrow(),
                p.PA7.reborrow(),
                p.PC4.reborrow(),
                p.PC5.reborrow(),
                p.PB12.reborrow(),
                p.PB13.reborrow(),
                p.PB11.reborrow(),
                [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF],
                p.ETH_SMA.reborrow(),
                p.PA2.reborrow(),
                p.PC1.reborrow(),
            );
            defmt::assert_eq!(AFIO.mapr().read().eth_remap(), false);
        }
        {
            // no remap MII
            afio_registers_set_remap();
            Ethernet::new_mii(
                &mut PacketQueue::<1, 1>::new(),
                p.ETH.reborrow(),
                Irqs,
                p.PA1.reborrow(),
                p.PC3.reborrow(),
                p.PA7.reborrow(),
                p.PC4.reborrow(),
                p.PC5.reborrow(),
                p.PB0.reborrow(),
                p.PB1.reborrow(),
                p.PB12.reborrow(),
                p.PB13.reborrow(),
                p.PC2.reborrow(),
                p.PB8.reborrow(),
                p.PB11.reborrow(),
                [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF],
                p.ETH_SMA.reborrow(),
                p.PA2.reborrow(),
                p.PC1.reborrow(),
            );
            defmt::assert_eq!(AFIO.mapr().read().eth_remap(), false);
        }
        {
            // remap RMII
            reset_afio_registers();
            Ethernet::new(
                &mut PacketQueue::<1, 1>::new(),
                p.ETH.reborrow(),
                Irqs,
                p.PA1.reborrow(),
                p.PD8.reborrow(),
                p.PD9.reborrow(),
                p.PD10.reborrow(),
                p.PB12.reborrow(),
                p.PB13.reborrow(),
                p.PB11.reborrow(),
                [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF],
                p.ETH_SMA.reborrow(),
                p.PA2.reborrow(),
                p.PC1.reborrow(),
            );
            defmt::assert_eq!(AFIO.mapr().read().eth_remap(), true);
        }
        {
            // remap MII
            reset_afio_registers();
            Ethernet::new_mii(
                &mut PacketQueue::<1, 1>::new(),
                p.ETH.reborrow(),
                Irqs,
                p.PA1.reborrow(),
                p.PC3.reborrow(),
                p.PD8.reborrow(),
                p.PD9.reborrow(),
                p.PD10.reborrow(),
                p.PD11.reborrow(),
                p.PD12.reborrow(),
                p.PB12.reborrow(),
                p.PB13.reborrow(),
                p.PC2.reborrow(),
                p.PB8.reborrow(),
                p.PB11.reborrow(),
                [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF],
                p.ETH_SMA.reborrow(),
                p.PA2.reborrow(),
                p.PC1.reborrow(),
            );
            defmt::assert_eq!(AFIO.mapr().read().eth_remap(), true);
        }

        // CAN1
        {
            // no remap
            afio_registers_set_remap();
            Can::new(p.CAN1.reborrow(), p.PA11.reborrow(), p.PA12.reborrow(), Irqs);
            defmt::assert_eq!(AFIO.mapr().read().can1_remap(), 0);
        }
        {
            // partial remap
            reset_afio_registers();
            Can::new(p.CAN1.reborrow(), p.PB8.reborrow(), p.PB9.reborrow(), Irqs);
            defmt::assert_eq!(AFIO.mapr().read().can1_remap(), 2);
        }
        {
            // full remap
            reset_afio_registers();
            Can::new(p.CAN1.reborrow(), p.PD0.reborrow(), p.PD1.reborrow(), Irqs);
            defmt::assert_eq!(AFIO.mapr().read().can1_remap(), 3);
        }

        // USART2
        {
            // no remap RX/TX/RTS/CTS
            afio_registers_set_remap();
            Uart::new_blocking_with_rtscts(
                p.USART2.reborrow(),
                p.PA3.reborrow(),
                p.PA2.reborrow(),
                p.PA1.reborrow(),
                p.PA0.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), false);
        }
        {
            // no remap RX/TX
            afio_registers_set_remap();
            Uart::new_blocking(
                p.USART2.reborrow(),
                p.PA3.reborrow(),
                p.PA2.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), false);
        }
        {
            // no remap TX
            afio_registers_set_remap();
            Uart::new_blocking_half_duplex(
                p.USART2.reborrow(),
                p.PA2.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), false);
        }
        {
            // full remap RX/TX/RTS/CTS
            reset_afio_registers();
            Uart::new_blocking_with_rtscts(
                p.USART2.reborrow(),
                p.PD6.reborrow(),
                p.PD5.reborrow(),
                p.PD4.reborrow(),
                p.PD3.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), false);
        }
        {
            // full remap RX/TX
            reset_afio_registers();
            Uart::new_blocking(
                p.USART2.reborrow(),
                p.PD6.reborrow(),
                p.PD5.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), false);
        }
        {
            // full remap TX
            reset_afio_registers();
            Uart::new_blocking_half_duplex(
                p.USART2.reborrow(),
                p.PD5.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart2_remap(), true);
        }

        // USART1
        {
            // no remap RX/TX/RTS/CTS
            afio_registers_set_remap();
            Uart::new_blocking_with_rtscts(
                p.USART1.reborrow(),
                p.PA10.reborrow(),
                p.PA9.reborrow(),
                p.PA12.reborrow(),
                p.PA11.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), false);
        }
        {
            // no remap RX/TX
            afio_registers_set_remap();
            Uart::new_blocking(
                p.USART1.reborrow(),
                p.PA10.reborrow(),
                p.PA9.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), false);
        }
        {
            // no remap TX
            afio_registers_set_remap();
            Uart::new_blocking_half_duplex(
                p.USART1.reborrow(),
                p.PA9.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), false);
        }
        {
            // remap RX/TX/RTS/CTS
            reset_afio_registers();
            Uart::new_blocking_with_rtscts(
                p.USART1.reborrow(),
                p.PB7.reborrow(),
                p.PB6.reborrow(),
                p.PA12.reborrow(),
                p.PA11.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), true);
        }
        {
            // remap RX/TX
            reset_afio_registers();
            Uart::new_blocking(
                p.USART1.reborrow(),
                p.PB7.reborrow(),
                p.PB6.reborrow(),
                Default::default(),
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), true);
        }
        {
            // remap TX
            reset_afio_registers();
            Uart::new_blocking_half_duplex(
                p.USART1.reborrow(),
                p.PB6.reborrow(),
                Default::default(),
                embassy_stm32::usart::HalfDuplexReadback::NoReadback,
            )
            .unwrap();
            defmt::assert_eq!(AFIO.mapr().read().usart1_remap(), true);
        }

        // TIM1
        {
            // full remap
            reset_afio_registers();
            SimplePwm::new(
                p.TIM1.reborrow(),
                Some(PwmPin::new(p.PE9.reborrow(), OutputType::PushPull)),
                Some(PwmPin::new(p.PE11.reborrow(), OutputType::PushPull)),
                None,
                None,
                khz(10),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr().read().tim1_remap(), 3);
        }
    }
}

#[cfg(feature = "afio-value-line")]
mod value_line {
    use super::*;

    pub fn run(p: &mut Peripherals) {
        // TIM13
        {
            // no remap
            reset_afio_registers();
            SimplePwm::new(
                p.TIM13.reborrow(),
                Some(PwmPin::new(p.PC8.reborrow(), OutputType::PushPull)),
                None,
                None,
                None,
                khz(10),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr2().read().tim13_remap(), false);
        }
        {
            // remap
            reset_afio_registers();
            SimplePwm::new(
                p.TIM13.reborrow(),
                Some(PwmPin::new(p.PB0.reborrow(), OutputType::PushPull)),
                None,
                None,
                None,
                khz(10),
                Default::default(),
            );
            defmt::assert_eq!(AFIO.mapr2().read().tim13_remap(), true);
        }
    }
}

#[cfg(not(feature = "afio-connectivity-line"))]
mod connectivity_line {
    use super::*;

    pub fn run(_: &mut Peripherals) {}
}

#[cfg(not(feature = "afio-value-line"))]
mod value_line {
    use super::*;

    pub fn run(_: &mut Peripherals) {}
}

fn reset_afio_registers() {
    set_afio_registers(false, 0);
}

fn afio_registers_set_remap() {
    set_afio_registers(true, 1);
}

fn set_afio_registers(bool_val: bool, num_val: u8) {
    AFIO.mapr().modify(|w| {
        w.set_swj_cfg(embassy_stm32::pac::afio::vals::SwjCfg::NO_OP);
        w.set_can1_remap(num_val);
        w.set_can2_remap(bool_val);
        w.set_eth_remap(bool_val);
        w.set_i2c1_remap(bool_val);
        w.set_spi1_remap(bool_val);
        w.set_spi3_remap(bool_val);
        w.set_tim1_remap(num_val);
        w.set_tim2_remap(num_val);
        w.set_tim3_remap(num_val);
        w.set_tim4_remap(bool_val);
        w.set_usart1_remap(bool_val);
        w.set_usart2_remap(bool_val);
        w.set_usart3_remap(num_val);
    });

    AFIO.mapr2().modify(|w| {
        w.set_cec_remap(bool_val);
        w.set_tim9_remap(bool_val);
        w.set_tim10_remap(bool_val);
        w.set_tim11_remap(bool_val);
        w.set_tim12_remap(bool_val);
        w.set_tim13_remap(bool_val);
        w.set_tim14_remap(bool_val);
        w.set_tim15_remap(bool_val);
        w.set_tim16_remap(bool_val);
        w.set_tim17_remap(bool_val);
    });
}
