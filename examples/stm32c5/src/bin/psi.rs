#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{self, Psi, PsiFrequency, PsiKernelDivider, PsiSource, Sysclk, mux};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, peripherals};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hsi = true; // HSI / 18 supplies the 8 MHz PSI reference.
    config.rcc.hsi_div3 = false;
    config.rcc.psi = Some(Psi {
        source: PsiSource::Hsi,
        frequency: PsiFrequency::F144mHz,
        sys: true,
        div3: true,
        kernel_div: Some(PsiKernelDivider::Div15),
    });
    config.rcc.sys = Sysclk::Psi;
    config.rcc.mux.clk48sel = mux::Clksel::Psidiv3;
    config.rcc.mux.adcdacsel = mux::Adcdacsel::Psik;
    config.rcc.mux.usart2sel = mux::Usartsel::Psik;
    // PSIK = 96 MHz; ADC/DAC clock = 96 / 4 = 24 MHz.
    config.rcc.adcdac_pre = embassy_stm32::pac::rcc::vals::Adcdacpre::Div4;

    // For an HSE reference, configure config.rcc.hse, select PsiSource::Hse,
    // and disable config.rcc.hsi if it is not needed elsewhere.
    let p = embassy_stm32::init(config);
    let clocks = rcc::clocks(&p.RCC);
    assert!(clocks.sys.to_hertz() == Some(Hertz(144_000_000)));
    assert!(clocks.psi.to_hertz() == Some(Hertz(144_000_000)));
    assert!(clocks.psidiv3.to_hertz() == Some(Hertz(48_000_000)));
    assert!(clocks.psik.to_hertz() == Some(Hertz(96_000_000)));
    assert!(clocks.hsidiv3.to_hertz().is_none());
    assert!(rcc::frequency::<peripherals::USART2>() == Hertz(96_000_000));
    assert!(rcc::frequency::<peripherals::ADC1>() == Hertz(24_000_000));
    assert!(rcc::frequency::<peripherals::USB>() == Hertz(48_000_000));
    assert!(rcc::frequency::<peripherals::RNG>() == Hertz(48_000_000));

    loop {
        info!("PSI SYSCLK=144 MHz, PSIDIV3=48 MHz, PSIK=96 MHz");
        Timer::after_secs(1).await;
    }
}
