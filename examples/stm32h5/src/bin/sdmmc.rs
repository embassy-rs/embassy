#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::sdmmc::sd::CmdBlock;
use embassy_stm32::sdmmc::sdio::SerialDataInterface;
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts, peripherals, sdmmc};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SdioBus<&'static mut SerialDataInterface<'static, 'static>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello world!");
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV4), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: None,
            divq: None,
            divr: Some(PllDiv::DIV4), // 100mhz
        });
        config.rcc.sys = Sysclk::PLL1_P; // 200 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV1; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::PLL2_R;
    }
    let p = embassy_stm32::init(config);

    let mut wl_reg = Output::new(p.PD0, Level::Low, Speed::High);
    let mut _bt_reg = Output::new(p.PG3, Level::Low, Speed::High);
    let mut _sdio_reset = Output::new(p.PD11, Level::Low, Speed::High);

    let sdio_clk = Input::new(unsafe { p.PC12.clone_unchecked() }, Pull::None);
    let sdio_cmd = Input::new(unsafe { p.PD2.clone_unchecked() }, Pull::None);
    let sdio_data0 = Input::new(unsafe { p.PC8.clone_unchecked() }, Pull::None);
    let sdio_data1 = Input::new(unsafe { p.PC9.clone_unchecked() }, Pull::None);
    let sdio_data2 = Input::new(unsafe { p.PC10.clone_unchecked() }, Pull::None);
    let sdio_data3 = Input::new(unsafe { p.PC11.clone_unchecked() }, Pull::None);

    let fw = include_bytes!("../../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");

    let sdmmc = Sdmmc::new_4bit(
        p.SDMMC1,
        Irqs,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    {
        // let _out1 = Output::new(p.PC12.reborrow(), Level::Low, Speed::High);
        // let _out2 = Output::new(p.PD2.reborrow(), Level::High, Speed::High);
        // let _out3 = Output::new(p.PC8.reborrow(), Level::High, Speed::High);
        // let _out4 = Output::new(p.PC9.reborrow(), Level::High, Speed::High);
        // let _out5 = Output::new(p.PC10.reborrow(), Level::High, Speed::High);
        // let _out6 = Output::new(p.PC11.reborrow(), Level::High, Speed::High);

        if sdio_clk.is_high() {
            trace!("sdio_clk is high");
        } else {
            trace!("sdio_clk is not high");
        }
        if sdio_cmd.is_high() {
            trace!("sdio_cmd is high");
        } else {
            trace!("sdio_cmd is not high");
        }

        if sdio_data0.is_high() {
            trace!("sdio_data0 is high");
        } else {
            trace!("sdio_data0 is not high");
        }
        if sdio_data1.is_high() {
            trace!("sdio_data1 is high");
        } else {
            trace!("sdio_data1 is not high");
        }

        if sdio_data2.is_high() {
            trace!("sdio_data2 is high");
        } else {
            trace!("sdio_data2 is not high");
        }

        if sdio_data3.is_high() {
            trace!("sdio_data3 is high");
        } else {
            trace!("sdio_data3 is not high");
        }

        trace!("WL_REG off/on");
        wl_reg.set_low();
        Timer::after_millis(250).await;
        wl_reg.set_high();
        Timer::after_millis(250).await;
    }

    static SDMMC: StaticCell<Sdmmc<'static>> = StaticCell::new();
    static SDIO: StaticCell<SerialDataInterface<'static, 'static>> = StaticCell::new();

    let sdmmc = SDMMC.init(sdmmc);

    let _cmd_block = CmdBlock::new();

    let sdio = SerialDataInterface::new(sdmmc, mhz(50)).await.unwrap();

    let sdio = SDIO.init(sdio);

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    info!("new sdio");

    let (_net_device, mut control, runner) = cyw43::new_sdio(state, sdio, fw).await;

    info!("spawn task");

    spawner.spawn(unwrap!(cyw43_task(runner)));

    info!("init control");

    control.init(clm).await;

    cortex_m::asm::bkpt();

    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let mut scanner = control.scan(Default::default()).await;
    while let Some(bss) = scanner.next().await {
        if let Ok(ssid_str) = str::from_utf8(&bss.ssid) {
            info!("scanned {} == {:x}", ssid_str, bss.bssid);
        }
    }

    cortex_m::asm::bkpt();
}
