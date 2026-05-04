#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32_wpan::bindings::mac::{ST_MAC_callbacks_t, ST_MAC_init};
use {defmt_rtt as _, panic_probe as _};

static _MAC_CALLBACKS: ST_MAC_callbacks_t = ST_MAC_callbacks_t {
    mlmeAssociateCnfCb: None,       // ST_MAC_MLMEAssociateCnfCbPtr,
    mlmeAssociateIndCb: None,       // ST_MAC_MLMEAssociateIndCbPtr,
    mlmeBeaconNotifyIndCb: None,    // ST_MAC_MLMEBeaconNotifyIndCbPtr,
    mlmeCalibrateCnfCb: None,       // ST_MAC_MLMECalibrateCnfCbPtr,
    mlmeCommStatusIndCb: None,      // ST_MAC_MLMECommStatusIndCbPtr,
    mlmeDisassociateCnfCb: None,    // ST_MAC_MLMEDisassociateCnfCbPtr,
    mlmeDisassociateIndCb: None,    // ST_MAC_MLMEDisassociateIndCbPtr,
    mlmeDpsCnfCb: None,             // ST_MAC_MLMEDpsCnfCbPtr,
    mlmeDpsIndCb: None,             // ST_MAC_MLMEDpsIndCbPtr,
    mlmeGetCnfCb: None,             // ST_MAC_MLMEGetCnfCbPtr,
    mlmeGtsCnfCb: None,             // ST_MAC_MLMEGtsCnfCbPtr,
    mlmeGtsIndCb: None,             // ST_MAC_MLMEGtsIndCbPtr,
    mlmeOrphanIndCb: None,          // ST_MAC_MLMEOrphanIndCbPtr,
    mlmePollCnfCb: None,            // ST_MAC_MLMEPollCnfCbPtr,
    mlmeResetCnfCb: None,           // ST_MAC_MLMEResetCnfCbPtr,
    mlmeRxEnableCnfCb: None,        // ST_MAC_MLMERxEnableCnfCbPtr,
    mlmeScanCnfCb: None,            // ST_MAC_MLMEScanCnfCbPtr,
    mlmeSetCnfCb: None,             // ST_MAC_MLMESetCnfCbPtr,
    mlmeSoundingCnfCb: None,        // ST_MAC_MLMESoundingCnfCbPtr,
    mlmeStartCnfCb: None,           // ST_MAC_MLMEStartCnfCbPtr,
    mlmeSyncLossIndCb: None,        // ST_MAC_MLMESyncLossIndCbPtr,
    mcpsDataIndCb: None,            // ST_MAC_MCPSDataIndCbPtr,
    mcpsDataCnfCb: None,            // ST_MAC_MCPSDataCnfCbPtr,
    mcpsPurgeCnfCb: None,           // ST_MAC_MCPSPurgeCnfCbPtr,
    mlmePollIndCb: None,            // ST_MAC_MLMEPollIndCbPtr,
    mlmeBeaconReqIndCb: None,       // ST_MAC_MLMEBeaconReqIndCbPtr,
    mlmeBeaconCnfCb: None,          // ST_MAC_MLMEBeaconCnfCbPtr,
    mlmeGetPwrInfoTableCnfCb: None, // ST_MAC_MLMEGetPwrInfoTableCnfCbPtr,
    mlmeSetPwrInfoTableCnfCb: None, // ST_MAC_MLMESetPwrInfoTableCnfCbPtr,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // Fine-tune PLL1 dividers/multipliers
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        // divq: Some(PllDiv::DIV10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (USBOTG)
        frac: Some(0),             // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;

    // voltage scale for max performance
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    // route PLL1_P into the USB‐OTG‐HS block
    config.rcc.sys = Sysclk::PLL1_R;

    // let p = embassy_stm32::init(config);

    // config.rcc.sys = Sysclk::HSI;
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let _p = embassy_stm32::init(config);
    info!("Hello World!");

    let status = unsafe { ST_MAC_init(&_MAC_CALLBACKS as *const _ as *mut _) };

    info!("mac init: {}", status);

    cortex_m::asm::bkpt();
}
