//! This example shows how to use an STM32 as both an SPI master and slave.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{assert_eq, error, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Executor;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts, dma, exti, gpio, interrupt, peripherals};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_STREAM2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

const READ: u8 = 0x01;
const SET: u8 = 0xC2;
const RESET: u8 = 0xC8;

#[embassy_executor::task]
async fn device_task(mut dev: Spi<'static, Async, spi::mode::Slave>) -> ! {
    info!("Device start");

    let mut state: u8 = 0;

    loop {
        let mut buf = [0u8; 2];
        let res = match dev.read(&mut buf).await {
            Ok(_size) => match buf[0] {
                READ => {
                    info!("Device received READ");
                    dev.write(&[state]).await.map(|size| {
                        assert_eq!(size, 1);
                    })
                }
                SET => {
                    info!("Device received SET with payload {}", buf[1]);
                    state = buf[1];
                    Ok(())
                }
                RESET => {
                    info!("Device received RESET");
                    state = 0;
                    Ok(())
                }
                x => {
                    error!("Device received invalid message {:02X}", x);
                    continue;
                }
            },
            Err(err) => {
                error!("Device error during receive: {}", err);
                continue;
            }
        };
        if let Err(err) = res {
            error!("Device error during respond: {}", err);
        }
    }
}

/// An SPI device on a single-device SPI bus.
struct SpiDevice {
    bus: Spi<'static, Async, spi::mode::Master>,
    nss: Output<'static>,
}

impl SpiDevice {
    async fn read(&mut self, buf: &mut [u8]) -> Result<(), spi::Error> {
        self.nss.set_low();
        let res = self.bus.read(buf).await;
        self.nss.set_high();
        res
    }

    async fn write(&mut self, buf: &[u8]) -> Result<(), spi::Error> {
        self.nss.set_low();
        let res = self.bus.write(buf).await;
        self.nss.set_high();
        res
    }
}

#[embassy_executor::task]
async fn controller_task(mut con: SpiDevice) {
    info!("Controller start");

    loop {
        let mut resp_buf = [0u8; 1];

        for i in 0x7F..0x8F {
            Timer::after_millis(100).await;

            match con.write(&[SET, i]).await {
                Ok(_) => {
                    info!("Controller set state to {}", i);
                }
                Err(err) => {
                    error!("Controller error during send: {}", err);
                    continue;
                }
            }

            Timer::after_millis(1).await;

            match con.write(&[READ, 0]).await {
                Ok(_) => {
                    info!("Controller sent read request");
                }
                Err(err) => {
                    error!("Controller error during send: {}", err);
                    continue;
                }
            }

            Timer::after_millis(100).await;

            match con.read(&mut resp_buf).await {
                Ok(_) => {
                    info!("Controller received state {}", resp_buf[0]);
                }
                Err(err) => {
                    error!("Controller error during receive: {}", err);
                    continue;
                }
            }

            assert_eq!(i, resp_buf[0]);
        }

        Timer::after_millis(100).await;

        match con.write(&[RESET, 0]).await {
            Ok(_) => {
                info!("Controller resetset state");
            }
            Err(err) => {
                error!("Controller error during send: {}", err);
                continue;
            }
        }

        Timer::after_millis(1).await;

        match con.write(&[READ, 0]).await {
            Ok(_) => {
                info!("Controller sent read request");
            }
            Err(err) => {
                error!("Controller error during send: {}", err);
                continue;
            }
        }

        Timer::after_millis(1).await;

        match con.read(&mut resp_buf).await {
            Ok(_) => {
                info!("Controller received state {}", resp_buf[0]);
            }
            Err(err) => {
                error!("Controller error during receive: {}", err);
                continue;
            }
        }

        assert_eq!(0, resp_buf[0]);
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8), // 100 MHz for SPI 1 and 3
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = mhz(1);

    let controller = SpiDevice {
        bus: Spi::new(p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH0, p.DMA1_CH1, Irqs, spi_cfg),
        nss: Output::new(p.PA4, gpio::Level::High, gpio::Speed::VeryHigh),
    };
    let device = Spi::new_slave(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        p.PA15,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        Some(p.EXTI15),
        spi_cfg,
    );

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(unwrap!(device_task(device)));
        spawner.spawn(unwrap!(controller_task(controller)));
    })
}
