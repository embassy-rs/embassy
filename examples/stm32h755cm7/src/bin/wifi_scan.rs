#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::str;

use cyw43::{Cyw4373, aligned_bytes};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::{Config, SharedData, bind_interrupts, peripherals, sdmmc};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
});

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, cyw43::SdioBus<&'static mut Sdmmc<'static>>, Cyw4373>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello world!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default());
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8),
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }
    let mut p = embassy_stm32::init_primary(config, &SHARED_DATA);

    // STM32Cube / AIROC SDMMC1 bring-up from NUCLEO-H745ZI-Q (`main.c` `MX_GPIO_Init`):
    //   - **`PG8` = `CYBSP_BT_POWER`** — drive **LOW** for Wi‑Fi-only use. This is the
    //     combo-module **supply strap** from Cube; leaving it floating often yields
    //     SDIO F1 + FW load but **F2 / `IORDY` never ready** (WLAN never comes up).
    //   - **`PD0` = `WL_REG_ON`**
    // STM32H755 same package = same Nucleo144 pins (PC12/PD2/PC8–PC11).
    //
    // Wi‑Fi only: no BT stack / `cyw43` `bluetooth` feature — still drive `PG8` like Cube.
    let _bt_power = Output::new(p.PG8, Level::Low, Speed::Low);
    let mut wl_reg = Output::new(p.PD0, Level::Low, Speed::Low);

    // Debug timing: make power sequencing conservative so we can separate
    // power-on margin issues from firmware/SDIO issues.
    trace!("WL_REG_ON low, pre-power delay");
    Timer::after_millis(100).await;

    let fw = aligned_bytes!("../../../../cyw43-firmware/4373A0.bin");
    let clm = aligned_bytes!("../../../../cyw43-firmware/4373A0_clm.bin");
    let nvram = aligned_bytes!("../../../../cyw43-firmware/nvram_murata_2bc.bin");
    info!(
        "CYW43 debug pair: module=MURATA-2BC chip=CYW4373 fw=4373A0.bin({} B) clm=4373A0_clm.bin({} B) nvram=nvram_murata_2bc.bin({} B)",
        fw.len(),
        clm.len(),
        nvram.len(),
    );

    // toggle_sdmmc_data: Workaround for Nucleo144-M.2 Adapter (from official
    // console_task.c).  Temporarily drive the four SDMMC data lines LOW then
    // HIGH before handing them to the SDMMC peripheral as an alternate
    // function.  This clears any stuck state the CYW module may have left on
    // the lines from a prior run.  Must happen BEFORE Sdmmc::new_4bit takes
    // ownership of the pins.
    trace!("toggle_sdmmc_data");
    {
        // SAFETY: the Outputs are dropped (pins return to input) before
        // Sdmmc::new_4bit reconfigures them as SDMMC AF below.
        let mut d0 = Output::new(p.PC8.reborrow(), Level::Low, Speed::Low);
        let mut d1 = Output::new(p.PC9.reborrow(), Level::Low, Speed::Low);
        let mut d2 = Output::new(p.PC10.reborrow(), Level::Low, Speed::Low);
        let mut d3 = Output::new(p.PC11.reborrow(), Level::Low, Speed::Low);
        Timer::after_millis(10).await;
        d0.set_high();
        d1.set_high();
        d2.set_high();
        d3.set_high();
        Timer::after_millis(10).await;
        // d0-d3 drop here → pins reconfigured to floating input
    }

    // Create the SDMMC peripheral (configures PC8-PC11 as SDMMC AF).
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

    static SDMMC: StaticCell<Sdmmc<'static>> = StaticCell::new();
    static STATE: StaticCell<cyw43::State> = StaticCell::new();

    let sdmmc = SDMMC.init(sdmmc);
    let state = STATE.init(cyw43::State::new());

    trace!("WL_REG_ON high, wait for PMU before SDIO");
    wl_reg.set_high();
    Timer::after_millis(500).await;

    let (_net_device, mut control, runner) = cyw43::new_4373_sdio(state, sdmmc, fw, nvram, 12_500_000).await.unwrap();

    info!("spawn task");
    spawner.spawn(unwrap!(cyw43_task(runner)));

    info!("init control");
    control.init(clm).await;

    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let mut scanner = control.scan(Default::default()).await;
    let mut count = 0u32;
    while let Some(bss) = scanner.next().await {
        let ssid_bytes = &bss.ssid[..bss.ssid_len as usize];
        let b = bss.bssid;
        if let Ok(ssid_str) = str::from_utf8(ssid_bytes) {
            info!(
                "#{}: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}  rssi={}  \"{}\"",
                count, b[0], b[1], b[2], b[3], b[4], b[5], bss.rssi, ssid_str
            );
        } else {
            info!(
                "#{}: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}  rssi={}  (non-utf8 {} bytes)",
                count, b[0], b[1], b[2], b[3], b[4], b[5], bss.rssi, bss.ssid_len
            );
        }
        count += 1;
    }
    info!("scan complete, {} networks found", count);

    cortex_m::asm::bkpt();
}
