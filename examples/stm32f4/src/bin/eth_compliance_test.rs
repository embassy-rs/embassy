#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, StationManagement};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, eth, peripherals, rng};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL180,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 180 / 2 = 180Mhz.
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello Compliance World!");

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    const PHY_ADDR: u8 = 0;
    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let mut device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        p.PB13,
        p.PG11,
        GenericPhy::new(PHY_ADDR),
        mac_addr,
    );

    let sm = device.station_management();

    // Just an example. Exact register settings depend on the specific PHY and test.
    sm.smi_write(PHY_ADDR, 0, 0x2100);
    sm.smi_write(PHY_ADDR, 11, 0xA000);

    // NB: Remember to reset the PHY after testing before starting the networking stack

    loop {
        Timer::after_secs(1).await;
    }
}
