#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dsihost::{DsiHost, PacketType};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::Ltdc;
use embassy_stm32::pac::dsihost::regs::{Ier0, Ier1};
use embassy_stm32::pac::ltdc::vals::{Bf1, Bf2, Depol, Hspol, Imr, Pcpol, Pf, Vspol};
use embassy_stm32::pac::{DSIHOST, LTDC};
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllQDiv, PllRDiv, PllSource, Sysclk,
};
use embassy_stm32::time::mhz;
use embassy_time::{Duration, Timer, block_for};
use {defmt_rtt as _, panic_probe as _};

enum _Orientation {
    Landscape,
    Portrait,
}

const _LCD_ORIENTATION: _Orientation = _Orientation::Landscape;
const LCD_X_SIZE: u16 = 800;
const LCD_Y_SIZE: u16 = 480;

static FERRIS_IMAGE: &[u8; 1536000] = include_bytes!("ferris.bin");

// This example allows to display an image on the STM32F469NI-DISCO boards
// with the Revision C, that is at least the boards marked DK32F469I$AU1.
// These boards have the NT35510 display driver. This example does not work
// for the older revisions with  OTM8009A, though there are lots of C-examples
// available online where the correct config for the OTM8009A could be gotten from.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    config.rcc.apb2_pre = APBPrescaler::DIV2;

    // HSE is on and ready
    config.rcc.hse = Some(Hse {
        freq: mhz(8),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = PllSource::HSE;

    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV8, // PLLM
        mul: PllMul::MUL360,     // PLLN
        divp: Some(PllPDiv::DIV2),
        divq: Some(PllQDiv::DIV7), // was DIV4, but STM BSP example uses 7
        divr: Some(PllRDiv::DIV6),
    });

    // This seems to be working, the values in the RCC.PLLSAICFGR are correct according to the debugger. Also on and ready according to CR
    config.rcc.pllsai = Some(Pll {
        prediv: PllPreDiv::DIV8,   // Actually ignored
        mul: PllMul::MUL384,       // PLLN
        divp: None,                // PLLP
        divq: None,                // PLLQ
        divr: Some(PllRDiv::DIV7), // PLLR (Sai actually has special clockdiv register)
    });

    let p = embassy_stm32::init(config);
    info!("Starting...");

    let mut led = Output::new(p.PG6, Level::High, Speed::Low);

    // According to UM for the discovery kit, PH7 is an active-low reset for the LCD and touchsensor
    let mut reset = Output::new(p.PH7, Level::Low, Speed::High);

    // CubeMX example waits 20 ms before de-asserting reset
    embassy_time::block_for(embassy_time::Duration::from_millis(20));

    // Disable the reset signal and wait 140ms as in the Linux driver (CubeMX waits only 20)
    reset.set_high();
    embassy_time::block_for(embassy_time::Duration::from_millis(140));

    let mut ltdc = Ltdc::new(p.LTDC);
    let mut dsi = DsiHost::new(p.DSIHOST, p.PJ2);
    let version = dsi.get_version();
    defmt::warn!("en: {:x}", version);

    // Disable the DSI wrapper
    dsi.disable_wrapper_dsi();

    // Disable the DSI host
    dsi.disable();

    // D-PHY clock and digital disable
    DSIHOST.pctlr().modify(|w| {
        w.set_cke(false);
        w.set_den(false)
    });

    // Turn off the DSI PLL
    DSIHOST.wrpcr().modify(|w| w.set_pllen(false));

    // Disable the regulator
    DSIHOST.wrpcr().write(|w| w.set_regen(false));

    // Enable regulator
    info!("DSIHOST: enabling regulator");
    DSIHOST.wrpcr().write(|w| w.set_regen(true));

    for _ in 1..1000 {
        // The regulator status (ready or not) can be monitored with the RRS flag in the DSI_WISR register.
        // Once it is set, we stop waiting.
        if DSIHOST.wisr().read().rrs() {
            info!("DSIHOST Regulator ready");
            break;
        }
        embassy_time::block_for(embassy_time::Duration::from_millis(1));
    }

    if !DSIHOST.wisr().read().rrs() {
        defmt::panic!("DSIHOST: enabling regulator FAILED");
    }

    // Set up PLL and enable it
    DSIHOST.wrpcr().modify(|w| {
        w.set_pllen(true);
        w.set_ndiv(125); // PLL loop division factor set to 125
        w.set_idf(2); // PLL input divided by 2
        w.set_odf(0); // PLL output divided by 1
    });

    /* 500 MHz / 8 = 62.5 MHz = 62500 kHz */
    const LANE_BYTE_CLK_K_HZ: u16 = 62500; // https://github.com/STMicroelectronics/32f469idiscovery-bsp/blob/ec051de2bff3e1b73a9ccd49c9b85abf7320add9/stm32469i_discovery_lcd.c#L224C21-L224C26

    const _LCD_CLOCK: u16 = 27429; // https://github.com/STMicroelectronics/32f469idiscovery-bsp/blob/ec051de2bff3e1b73a9ccd49c9b85abf7320add9/stm32469i_discovery_lcd.c#L183

    /* TX_ESCAPE_CKDIV = f(LaneByteClk)/15.62 = 4 */
    const TX_ESCAPE_CKDIV: u8 = (LANE_BYTE_CLK_K_HZ / 15620) as u8; // https://github.com/STMicroelectronics/32f469idiscovery-bsp/blob/ec051de2bff3e1b73a9ccd49c9b85abf7320add9/stm32469i_discovery_lcd.c#L230

    for _ in 1..1000 {
        embassy_time::block_for(embassy_time::Duration::from_millis(1));
        // The PLL status (lock or unlock) can be monitored with the PLLLS flag in the DSI_WISR register.
        // Once it is set, we stop waiting.
        if DSIHOST.wisr().read().pllls() {
            info!("DSIHOST PLL locked");
            break;
        }
    }

    if !DSIHOST.wisr().read().pllls() {
        defmt::panic!("DSIHOST: enabling PLL FAILED");
    }

    // Set the PHY parameters

    // D-PHY clock and digital enable
    DSIHOST.pctlr().write(|w| {
        w.set_cke(true);
        w.set_den(true);
    });

    // Set Clock lane to high-speed mode and disable automatic clock lane control
    DSIHOST.clcr().modify(|w| {
        w.set_dpcc(true);
        w.set_acr(false);
    });

    // Set number of active data lanes to two (lanes 0 and 1)
    DSIHOST.pconfr().modify(|w| w.set_nl(1));

    // Set the DSI clock parameters

    // Set the TX escape clock division factor to 4
    DSIHOST.ccr().modify(|w| w.set_txeckdiv(TX_ESCAPE_CKDIV));

    // Calculate the bit period in high-speed mode in unit of 0.25 ns (UIX4)
    // The equation is : UIX4 = IntegerPart( (1000/F_PHY_Mhz) * 4 )
    // Where : F_PHY_Mhz = (NDIV * HSE_Mhz) / (IDF * ODF)
    // Set the bit period in high-speed mode
    DSIHOST.wpcr0().modify(|w| w.set_uix4(8)); // 8 is set in the BSP example (confirmed with Debugger)

    // Disable all error interrupts and reset the Error Mask
    DSIHOST.ier0().write_value(Ier0(0));
    DSIHOST.ier1().write_value(Ier1(0));

    // Enable this to fix read timeout
    DSIHOST.pcr().modify(|w| w.set_btae(true));

    const DSI_PIXEL_FORMAT_RGB888: u8 = 0x05;
    const _DSI_PIXEL_FORMAT_ARGB888: u8 = 0x00;

    const HACT: u16 = LCD_X_SIZE;
    const VACT: u16 = LCD_Y_SIZE;

    const VSA: u16 = 120;
    const VBP: u16 = 150;
    const VFP: u16 = 150;
    const HSA: u16 = 2;
    const HBP: u16 = 34;
    const HFP: u16 = 34;

    const VIRTUAL_CHANNEL_ID: u8 = 0;

    const COLOR_CODING: u8 = DSI_PIXEL_FORMAT_RGB888;
    const VS_POLARITY: bool = false; // DSI_VSYNC_ACTIVE_HIGH == 0
    const HS_POLARITY: bool = false; // DSI_HSYNC_ACTIVE_HIGH == 0
    const DE_POLARITY: bool = false; // DSI_DATA_ENABLE_ACTIVE_HIGH == 0
    const MODE: u8 = 2; // DSI_VID_MODE_BURST; /* Mode Video burst ie : one LgP per line */
    const NULL_PACKET_SIZE: u16 = 0xFFF;
    const NUMBER_OF_CHUNKS: u16 = 0;
    const PACKET_SIZE: u16 = HACT; /* Value depending on display orientation choice portrait/landscape */
    const HORIZONTAL_SYNC_ACTIVE: u16 = 4; // ((HSA as u32 * LANE_BYTE_CLK_K_HZ as u32 ) / LCD_CLOCK as u32 ) as u16;
    const HORIZONTAL_BACK_PORCH: u16 = 77; //((HBP as u32  * LANE_BYTE_CLK_K_HZ as u32 ) / LCD_CLOCK as u32) as u16;
    const HORIZONTAL_LINE: u16 = 1982; //(((HACT + HSA + HBP + HFP) as u32  * LANE_BYTE_CLK_K_HZ as u32 ) / LCD_CLOCK as u32 ) as u16; /* Value depending on display orientation choice portrait/landscape */
    // FIXME: Make depend on orientation
    const VERTICAL_SYNC_ACTIVE: u16 = VSA;
    const VERTICAL_BACK_PORCH: u16 = VBP;
    const VERTICAL_FRONT_PORCH: u16 = VFP;
    const VERTICAL_ACTIVE: u16 = VACT;
    const LP_COMMAND_ENABLE: bool = true; /* Enable sending commands in mode LP (Low Power) */

    /* Largest packet size possible to transmit in LP mode in VSA, VBP, VFP regions */
    /* Only useful when sending LP packets is allowed while streaming is active in video mode */
    const LP_LARGEST_PACKET_SIZE: u8 = 16;

    /* Largest packet size possible to transmit in LP mode in HFP region during VACT period */
    /* Only useful when sending LP packets is allowed while streaming is active in video mode */
    const LPVACT_LARGEST_PACKET_SIZE: u8 = 0;

    const LPHORIZONTAL_FRONT_PORCH_ENABLE: bool = true; /* Allow sending LP commands during HFP period */
    const LPHORIZONTAL_BACK_PORCH_ENABLE: bool = true; /* Allow sending LP commands during HBP period */
    const LPVERTICAL_ACTIVE_ENABLE: bool = true; /* Allow sending LP commands during VACT period */
    const LPVERTICAL_FRONT_PORCH_ENABLE: bool = true; /* Allow sending LP commands during VFP period */
    const LPVERTICAL_BACK_PORCH_ENABLE: bool = true; /* Allow sending LP commands during VBP period */
    const LPVERTICAL_SYNC_ACTIVE_ENABLE: bool = true; /* Allow sending LP commands during VSync = VSA period */
    const FRAME_BTAACKNOWLEDGE_ENABLE: bool = false; /* Frame bus-turn-around acknowledge enable => false according to debugger */

    /* Select video mode by resetting CMDM and DSIM bits */
    DSIHOST.mcr().modify(|w| w.set_cmdm(false));
    DSIHOST.wcfgr().modify(|w| w.set_dsim(false));

    /* Configure the video mode transmission type */
    DSIHOST.vmcr().modify(|w| w.set_vmt(MODE));

    /* Configure the video packet size */
    DSIHOST.vpcr().modify(|w| w.set_vpsize(PACKET_SIZE));

    /* Set the chunks number to be transmitted through the DSI link */
    DSIHOST.vccr().modify(|w| w.set_numc(NUMBER_OF_CHUNKS));

    /* Set the size of the null packet */
    DSIHOST.vnpcr().modify(|w| w.set_npsize(NULL_PACKET_SIZE));

    /* Select the virtual channel for the LTDC interface traffic */
    DSIHOST.lvcidr().modify(|w| w.set_vcid(VIRTUAL_CHANNEL_ID));

    /* Configure the polarity of control signals */
    DSIHOST.lpcr().modify(|w| {
        w.set_dep(DE_POLARITY);
        w.set_hsp(HS_POLARITY);
        w.set_vsp(VS_POLARITY);
    });

    /* Select the color coding for the host */
    DSIHOST.lcolcr().modify(|w| w.set_colc(COLOR_CODING));

    /* Select the color coding for the wrapper */
    DSIHOST.wcfgr().modify(|w| w.set_colmux(COLOR_CODING));

    /* Set the Horizontal Synchronization Active (HSA) in lane byte clock cycles */
    DSIHOST.vhsacr().modify(|w| w.set_hsa(HORIZONTAL_SYNC_ACTIVE));

    /* Set the Horizontal Back Porch (HBP) in lane byte clock cycles */
    DSIHOST.vhbpcr().modify(|w| w.set_hbp(HORIZONTAL_BACK_PORCH));

    /* Set the total line time (HLINE=HSA+HBP+HACT+HFP) in lane byte clock cycles */
    DSIHOST.vlcr().modify(|w| w.set_hline(HORIZONTAL_LINE));

    /* Set the Vertical Synchronization Active (VSA) */
    DSIHOST.vvsacr().modify(|w| w.set_vsa(VERTICAL_SYNC_ACTIVE));

    /* Set the Vertical Back Porch (VBP)*/
    DSIHOST.vvbpcr().modify(|w| w.set_vbp(VERTICAL_BACK_PORCH));

    /* Set the Vertical Front Porch (VFP)*/
    DSIHOST.vvfpcr().modify(|w| w.set_vfp(VERTICAL_FRONT_PORCH));

    /* Set the Vertical Active period*/
    DSIHOST.vvacr().modify(|w| w.set_va(VERTICAL_ACTIVE));

    /* Configure the command transmission mode */
    DSIHOST.vmcr().modify(|w| w.set_lpce(LP_COMMAND_ENABLE));

    /* Low power largest packet size */
    DSIHOST.lpmcr().modify(|w| w.set_lpsize(LP_LARGEST_PACKET_SIZE));

    /* Low power VACT largest packet size */
    DSIHOST.lpmcr().modify(|w| w.set_lpsize(LP_LARGEST_PACKET_SIZE));
    DSIHOST.lpmcr().modify(|w| w.set_vlpsize(LPVACT_LARGEST_PACKET_SIZE));

    /* Enable LP transition in HFP period */
    DSIHOST.vmcr().modify(|w| w.set_lphfpe(LPHORIZONTAL_FRONT_PORCH_ENABLE));

    /* Enable LP transition in HBP period */
    DSIHOST.vmcr().modify(|w| w.set_lphbpe(LPHORIZONTAL_BACK_PORCH_ENABLE));

    /* Enable LP transition in VACT period */
    DSIHOST.vmcr().modify(|w| w.set_lpvae(LPVERTICAL_ACTIVE_ENABLE));

    /* Enable LP transition in VFP period */
    DSIHOST.vmcr().modify(|w| w.set_lpvfpe(LPVERTICAL_FRONT_PORCH_ENABLE));

    /* Enable LP transition in VBP period */
    DSIHOST.vmcr().modify(|w| w.set_lpvbpe(LPVERTICAL_BACK_PORCH_ENABLE));

    /* Enable LP transition in vertical sync period */
    DSIHOST.vmcr().modify(|w| w.set_lpvsae(LPVERTICAL_SYNC_ACTIVE_ENABLE));

    /* Enable the request for an acknowledge response at the end of a frame */
    DSIHOST.vmcr().modify(|w| w.set_fbtaae(FRAME_BTAACKNOWLEDGE_ENABLE));

    /* Configure DSI PHY HS2LP and LP2HS timings */
    const CLOCK_LANE_HS2_LPTIME: u16 = 35;
    const CLOCK_LANE_LP2_HSTIME: u16 = 35;
    const DATA_LANE_HS2_LPTIME: u8 = 35;
    const DATA_LANE_LP2_HSTIME: u8 = 35;
    const DATA_LANE_MAX_READ_TIME: u16 = 0;
    const STOP_WAIT_TIME: u8 = 10;

    const MAX_TIME: u16 = if CLOCK_LANE_HS2_LPTIME > CLOCK_LANE_LP2_HSTIME {
        CLOCK_LANE_HS2_LPTIME
    } else {
        CLOCK_LANE_LP2_HSTIME
    };

    /* Clock lane timer configuration */

    /* In Automatic Clock Lane control mode, the DSI Host can turn off the clock lane between two
     High-Speed transmission.
     To do so, the DSI Host calculates the time required for the clock lane to change from HighSpeed
     to Low-Power and from Low-Power to High-Speed.
     This timings are configured by the HS2LP_TIME and LP2HS_TIME in the DSI Host Clock Lane Timer Configuration
     Register (DSI_CLTCR).
     But the DSI Host is not calculating LP2HS_TIME + HS2LP_TIME but 2 x HS2LP_TIME.

     Workaround : Configure HS2LP_TIME and LP2HS_TIME with the same value being the max of HS2LP_TIME or LP2HS_TIME.
    */

    DSIHOST.cltcr().modify(|w| {
        w.set_hs2lp_time(MAX_TIME);
        w.set_lp2hs_time(MAX_TIME)
    });

    // Data lane timer configuration
    DSIHOST.dltcr().modify(|w| {
        w.set_hs2lp_time(DATA_LANE_HS2_LPTIME);
        w.set_lp2hs_time(DATA_LANE_LP2_HSTIME);
        w.set_mrd_time(DATA_LANE_MAX_READ_TIME);
    });

    // Configure the wait period to request HS transmission after a stop state
    DSIHOST.pconfr().modify(|w| w.set_sw_time(STOP_WAIT_TIME));

    const _PCPOLARITY: bool = false; // LTDC_PCPOLARITY_IPC == 0

    const LTDC_DE_POLARITY: Depol = if !DE_POLARITY {
        Depol::ACTIVE_LOW
    } else {
        Depol::ACTIVE_HIGH
    };
    const LTDC_VS_POLARITY: Vspol = if !VS_POLARITY {
        Vspol::ACTIVE_HIGH
    } else {
        Vspol::ACTIVE_LOW
    };

    const LTDC_HS_POLARITY: Hspol = if !HS_POLARITY {
        Hspol::ACTIVE_HIGH
    } else {
        Hspol::ACTIVE_LOW
    };

    /* Timing Configuration */
    const HORIZONTAL_SYNC: u16 = HSA - 1;
    const VERTICAL_SYNC: u16 = VERTICAL_SYNC_ACTIVE - 1;
    const ACCUMULATED_HBP: u16 = HSA + HBP - 1;
    const ACCUMULATED_VBP: u16 = VERTICAL_SYNC_ACTIVE + VERTICAL_BACK_PORCH - 1;
    const ACCUMULATED_ACTIVE_W: u16 = LCD_X_SIZE + HSA + HBP - 1;
    const ACCUMULATED_ACTIVE_H: u16 = VERTICAL_SYNC_ACTIVE + VERTICAL_BACK_PORCH + VERTICAL_ACTIVE - 1;
    const TOTAL_WIDTH: u16 = LCD_X_SIZE + HSA + HBP + HFP - 1;
    const TOTAL_HEIGHT: u16 = VERTICAL_SYNC_ACTIVE + VERTICAL_BACK_PORCH + VERTICAL_ACTIVE + VERTICAL_FRONT_PORCH - 1;

    // DISABLE LTDC before making changes
    ltdc.disable();

    // Configure the HS, VS, DE and PC polarity
    LTDC.gcr().modify(|w| {
        w.set_hspol(LTDC_HS_POLARITY);
        w.set_vspol(LTDC_VS_POLARITY);
        w.set_depol(LTDC_DE_POLARITY);
        w.set_pcpol(Pcpol::RISING_EDGE);
    });

    // Set Synchronization size
    LTDC.sscr().modify(|w| {
        w.set_hsw(HORIZONTAL_SYNC);
        w.set_vsh(VERTICAL_SYNC)
    });

    // Set Accumulated Back porch
    LTDC.bpcr().modify(|w| {
        w.set_ahbp(ACCUMULATED_HBP);
        w.set_avbp(ACCUMULATED_VBP);
    });

    // Set Accumulated Active Width
    LTDC.awcr().modify(|w| {
        w.set_aah(ACCUMULATED_ACTIVE_H);
        w.set_aaw(ACCUMULATED_ACTIVE_W);
    });

    // Set Total Width
    LTDC.twcr().modify(|w| {
        w.set_totalh(TOTAL_HEIGHT);
        w.set_totalw(TOTAL_WIDTH);
    });

    // Set the background color value
    LTDC.bccr().modify(|w| {
        w.set_bcred(0);
        w.set_bcgreen(0);
        w.set_bcblue(0)
    });

    // Enable the Transfer Error and FIFO underrun interrupts
    LTDC.ier().modify(|w| {
        w.set_terrie(true);
        w.set_fuie(true);
    });

    // ENABLE LTDC after making changes
    ltdc.enable();

    dsi.enable();
    dsi.enable_wrapper_dsi();

    // First, delay 120 ms (reason unknown, STM32 Cube Example does it)
    block_for(Duration::from_millis(120));

    // 1 to 26
    dsi.write_cmd(0, NT35510_WRITES_0[0], &NT35510_WRITES_0[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_1[0], &NT35510_WRITES_1[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_2[0], &NT35510_WRITES_2[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_3[0], &NT35510_WRITES_3[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_4[0], &NT35510_WRITES_4[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_5[0], &NT35510_WRITES_5[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_6[0], &NT35510_WRITES_6[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_7[0], &NT35510_WRITES_7[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_8[0], &NT35510_WRITES_8[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_9[0], &NT35510_WRITES_9[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_10[0], &NT35510_WRITES_10[1..]).unwrap();
    // 11 missing
    dsi.write_cmd(0, NT35510_WRITES_12[0], &NT35510_WRITES_12[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_13[0], &NT35510_WRITES_13[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_14[0], &NT35510_WRITES_14[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_15[0], &NT35510_WRITES_15[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_16[0], &NT35510_WRITES_16[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_17[0], &NT35510_WRITES_17[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_18[0], &NT35510_WRITES_18[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_19[0], &NT35510_WRITES_19[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_20[0], &NT35510_WRITES_20[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_21[0], &NT35510_WRITES_21[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_22[0], &NT35510_WRITES_22[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_23[0], &NT35510_WRITES_23[1..]).unwrap();
    dsi.write_cmd(0, NT35510_WRITES_24[0], &NT35510_WRITES_24[1..]).unwrap();

    // Tear on
    dsi.write_cmd(0, NT35510_WRITES_26[0], &NT35510_WRITES_26[1..]).unwrap();

    // Set Pixel color format to RGB888
    dsi.write_cmd(0, NT35510_WRITES_37[0], &NT35510_WRITES_37[1..]).unwrap();

    // Add a delay, otherwise MADCTL not taken
    block_for(Duration::from_millis(200));

    // Configure orientation as landscape
    dsi.write_cmd(0, NT35510_MADCTL_LANDSCAPE[0], &NT35510_MADCTL_LANDSCAPE[1..])
        .unwrap();
    dsi.write_cmd(0, NT35510_CASET_LANDSCAPE[0], &NT35510_CASET_LANDSCAPE[1..])
        .unwrap();
    dsi.write_cmd(0, NT35510_RASET_LANDSCAPE[0], &NT35510_RASET_LANDSCAPE[1..])
        .unwrap();

    // Sleep out
    dsi.write_cmd(0, NT35510_WRITES_27[0], &NT35510_WRITES_27[1..]).unwrap();

    // Wait for sleep out exit
    block_for(Duration::from_millis(120));

    // Configure COLOR_CODING
    dsi.write_cmd(0, NT35510_WRITES_37[0], &NT35510_WRITES_37[1..]).unwrap();

    /* CABC : Content Adaptive Backlight Control section start >> */
    /* Note : defaut is 0 (lowest Brightness), 0xFF is highest Brightness, try 0x7F : intermediate value */
    dsi.write_cmd(0, NT35510_WRITES_31[0], &NT35510_WRITES_31[1..]).unwrap();
    /* defaut is 0, try 0x2C - Brightness Control Block, Display Dimming & BackLight on */
    dsi.write_cmd(0, NT35510_WRITES_32[0], &NT35510_WRITES_32[1..]).unwrap();
    /* defaut is 0, try 0x02 - image Content based Adaptive Brightness [Still Picture] */
    dsi.write_cmd(0, NT35510_WRITES_33[0], &NT35510_WRITES_33[1..]).unwrap();
    /* defaut is 0 (lowest Brightness), 0xFF is highest Brightness */
    dsi.write_cmd(0, NT35510_WRITES_34[0], &NT35510_WRITES_34[1..]).unwrap();
    /* CABC : Content Adaptive Backlight Control section end << */
    /* Display on */
    dsi.write_cmd(0, NT35510_WRITES_30[0], &NT35510_WRITES_30[1..]).unwrap();

    /* Send Command GRAM memory write (no parameters) : this initiates frame write via other DSI commands sent by */
    /* DSI host from LTDC incoming pixels in video mode */
    dsi.write_cmd(0, NT35510_WRITES_35[0], &NT35510_WRITES_35[1..]).unwrap();

    /* Initialize the LCD pixel width and pixel height */
    const WINDOW_X0: u16 = 0;
    const WINDOW_X1: u16 = LCD_X_SIZE; // 480 for ferris
    const WINDOW_Y0: u16 = 0;
    const WINDOW_Y1: u16 = LCD_Y_SIZE; // 800 for ferris
    const PIXEL_FORMAT: Pf = Pf::ARGB8888;
    //const FBStartAdress: u16 = FB_Address;
    const ALPHA: u8 = 255;
    const ALPHA0: u8 = 0;
    const BACKCOLOR_BLUE: u8 = 0;
    const BACKCOLOR_GREEN: u8 = 0;
    const BACKCOLOR_RED: u8 = 0;
    const IMAGE_WIDTH: u16 = LCD_X_SIZE; // 480 for ferris
    const IMAGE_HEIGHT: u16 = LCD_Y_SIZE; // 800 for ferris

    const PIXEL_SIZE: u8 = match PIXEL_FORMAT {
        Pf::ARGB8888 => 4,
        Pf::RGB888 => 3,
        Pf::ARGB4444 | Pf::RGB565 | Pf::ARGB1555 | Pf::AL88 => 2,
        _ => 1,
    };

    // Configure the horizontal start and stop position
    LTDC.layer(0).whpcr().write(|w| {
        w.set_whstpos(LTDC.bpcr().read().ahbp() + 1 + WINDOW_X0);
        w.set_whsppos(LTDC.bpcr().read().ahbp() + WINDOW_X1);
    });

    // Configures the vertical start and stop position
    LTDC.layer(0).wvpcr().write(|w| {
        w.set_wvstpos(LTDC.bpcr().read().avbp() + 1 + WINDOW_Y0);
        w.set_wvsppos(LTDC.bpcr().read().avbp() + WINDOW_Y1);
    });

    // Specify the pixel format
    LTDC.layer(0).pfcr().write(|w| w.set_pf(PIXEL_FORMAT));

    // Configures the default color values as zero
    LTDC.layer(0).dccr().modify(|w| {
        w.set_dcblue(BACKCOLOR_BLUE);
        w.set_dcgreen(BACKCOLOR_GREEN);
        w.set_dcred(BACKCOLOR_RED);
        w.set_dcalpha(ALPHA0);
    });

    // Specifies the constant ALPHA value
    LTDC.layer(0).cacr().write(|w| w.set_consta(ALPHA));

    // Specifies the blending factors
    LTDC.layer(0).bfcr().write(|w| {
        w.set_bf1(Bf1::CONSTANT);
        w.set_bf2(Bf2::CONSTANT);
    });

    // Configure the color frame buffer start address
    let fb_start_address: u32 = &FERRIS_IMAGE[0] as *const _ as u32;
    info!("Setting Framebuffer Start Address: {:010x}", fb_start_address);
    LTDC.layer(0).cfbar().write(|w| w.set_cfbadd(fb_start_address));

    // Configures the color frame buffer pitch in byte
    LTDC.layer(0).cfblr().write(|w| {
        w.set_cfbp(IMAGE_WIDTH * PIXEL_SIZE as u16);
        w.set_cfbll(((WINDOW_X1 - WINDOW_X0) * PIXEL_SIZE as u16) + 3);
    });

    // Configures the frame buffer line number
    LTDC.layer(0).cfblnr().write(|w| w.set_cfblnbr(IMAGE_HEIGHT));

    // Enable LTDC_Layer by setting LEN bit
    LTDC.layer(0).cr().modify(|w| w.set_len(true));

    //LTDC->SRCR = LTDC_SRCR_IMR;
    LTDC.srcr().modify(|w| w.set_imr(Imr::RELOAD));

    block_for(Duration::from_millis(5000));

    const READ_SIZE: u16 = 1;
    let mut data = [1u8; READ_SIZE as usize];
    dsi.read(0, PacketType::DcsShortPktRead(0xDA), READ_SIZE, &mut data)
        .unwrap();
    info!("Display ID1: {:#04x}", data);

    dsi.read(0, PacketType::DcsShortPktRead(0xDB), READ_SIZE, &mut data)
        .unwrap();
    info!("Display ID2: {:#04x}", data);

    dsi.read(0, PacketType::DcsShortPktRead(0xDC), READ_SIZE, &mut data)
        .unwrap();
    info!("Display ID3: {:#04x}", data);

    block_for(Duration::from_millis(500));

    info!("Config done, start blinking LED");
    loop {
        led.set_high();
        Timer::after_millis(1000).await;

        // Increase screen brightness
        dsi.write_cmd(0, NT35510_CMD_WRDISBV, &[0xFF]).unwrap();

        led.set_low();
        Timer::after_millis(1000).await;

        // Reduce screen brightness
        dsi.write_cmd(0, NT35510_CMD_WRDISBV, &[0x50]).unwrap();
    }
}

const NT35510_WRITES_0: &[u8] = &[0xF0, 0x55, 0xAA, 0x52, 0x08, 0x01]; // LV2:  Page 1 enable
const NT35510_WRITES_1: &[u8] = &[0xB0, 0x03, 0x03, 0x03]; // AVDD: 5.2V
const NT35510_WRITES_2: &[u8] = &[0xB6, 0x46, 0x46, 0x46]; // AVDD: Ratio
const NT35510_WRITES_3: &[u8] = &[0xB1, 0x03, 0x03, 0x03]; // AVEE: -5.2V
const NT35510_WRITES_4: &[u8] = &[0xB7, 0x36, 0x36, 0x36]; // AVEE: Ratio
const NT35510_WRITES_5: &[u8] = &[0xB2, 0x00, 0x00, 0x02]; // VCL: -2.5V
const NT35510_WRITES_6: &[u8] = &[0xB8, 0x26, 0x26, 0x26]; // VCL: Ratio
const NT35510_WRITES_7: &[u8] = &[0xBF, 0x01]; // VGH: 15V (Free Pump)
const NT35510_WRITES_8: &[u8] = &[0xB3, 0x09, 0x09, 0x09];
const NT35510_WRITES_9: &[u8] = &[0xB9, 0x36, 0x36, 0x36]; // VGH: Ratio
const NT35510_WRITES_10: &[u8] = &[0xB5, 0x08, 0x08, 0x08]; // VGL_REG: -10V
const NT35510_WRITES_12: &[u8] = &[0xBA, 0x26, 0x26, 0x26]; // VGLX: Ratio
const NT35510_WRITES_13: &[u8] = &[0xBC, 0x00, 0x80, 0x00]; // VGMP/VGSP: 4.5V/0V
const NT35510_WRITES_14: &[u8] = &[0xBD, 0x00, 0x80, 0x00]; // VGMN/VGSN:-4.5V/0V
const NT35510_WRITES_15: &[u8] = &[0xBE, 0x00, 0x50]; // VCOM: -1.325V
const NT35510_WRITES_16: &[u8] = &[0xF0, 0x55, 0xAA, 0x52, 0x08, 0x00]; // LV2: Page 0 enable
const NT35510_WRITES_17: &[u8] = &[0xB1, 0xFC, 0x00]; // Display control
const NT35510_WRITES_18: &[u8] = &[0xB6, 0x03]; // Src hold time
const NT35510_WRITES_19: &[u8] = &[0xB5, 0x51];
const NT35510_WRITES_20: &[u8] = &[0x00, 0x00, 0xB7]; // Gate EQ control
const NT35510_WRITES_21: &[u8] = &[0xB8, 0x01, 0x02, 0x02, 0x02]; // Src EQ control(Mode2)
const NT35510_WRITES_22: &[u8] = &[0xBC, 0x00, 0x00, 0x00]; // Inv. mode(2-dot)
const NT35510_WRITES_23: &[u8] = &[0xCC, 0x03, 0x00, 0x00];
const NT35510_WRITES_24: &[u8] = &[0xBA, 0x01];

const _NT35510_MADCTL_PORTRAIT: &[u8] = &[NT35510_CMD_MADCTL, 0x00];
const _NT35510_CASET_PORTRAIT: &[u8] = &[NT35510_CMD_CASET, 0x00, 0x00, 0x01, 0xDF];
const _NT35510_RASET_PORTRAIT: &[u8] = &[NT35510_CMD_RASET, 0x00, 0x00, 0x03, 0x1F];
const NT35510_MADCTL_LANDSCAPE: &[u8] = &[NT35510_CMD_MADCTL, 0x60];
const NT35510_CASET_LANDSCAPE: &[u8] = &[NT35510_CMD_CASET, 0x00, 0x00, 0x03, 0x1F];
const NT35510_RASET_LANDSCAPE: &[u8] = &[NT35510_CMD_RASET, 0x00, 0x00, 0x01, 0xDF];

const NT35510_WRITES_26: &[u8] = &[NT35510_CMD_TEEON, 0x00]; // Tear on
const NT35510_WRITES_27: &[u8] = &[NT35510_CMD_SLPOUT, 0x00]; // Sleep out
// 28,29 missing
const NT35510_WRITES_30: &[u8] = &[NT35510_CMD_DISPON, 0x00]; // Display on

const NT35510_WRITES_31: &[u8] = &[NT35510_CMD_WRDISBV, 0x7F];
const NT35510_WRITES_32: &[u8] = &[NT35510_CMD_WRCTRLD, 0x2C];
const NT35510_WRITES_33: &[u8] = &[NT35510_CMD_WRCABC, 0x02];
const NT35510_WRITES_34: &[u8] = &[NT35510_CMD_WRCABCMB, 0xFF];
const NT35510_WRITES_35: &[u8] = &[NT35510_CMD_RAMWR, 0x00];

//const NT35510_WRITES_36: &[u8] = &[NT35510_CMD_COLMOD, NT35510_COLMOD_RGB565]; // FIXME: Example sets it to 888 but rest of the code seems to configure DSI for 565
const NT35510_WRITES_37: &[u8] = &[NT35510_CMD_COLMOD, NT35510_COLMOD_RGB888];

// More of these: https://elixir.bootlin.com/linux/latest/source/include/video/mipi_display.h#L83
const _NT35510_CMD_TEEON_GET_DISPLAY_ID: u8 = 0x04;

const NT35510_CMD_TEEON: u8 = 0x35;
const NT35510_CMD_MADCTL: u8 = 0x36;

const NT35510_CMD_SLPOUT: u8 = 0x11;
const NT35510_CMD_DISPON: u8 = 0x29;
const NT35510_CMD_CASET: u8 = 0x2A;
const NT35510_CMD_RASET: u8 = 0x2B;
const NT35510_CMD_RAMWR: u8 = 0x2C; /* Memory write */
const NT35510_CMD_COLMOD: u8 = 0x3A;

const NT35510_CMD_WRDISBV: u8 = 0x51; /* Write display brightness */
const _NT35510_CMD_RDDISBV: u8 = 0x52; /* Read display brightness */
const NT35510_CMD_WRCTRLD: u8 = 0x53; /* Write CTRL display */
const _NT35510_CMD_RDCTRLD: u8 = 0x54; /* Read CTRL display value */
const NT35510_CMD_WRCABC: u8 = 0x55; /* Write content adaptative brightness control */
const NT35510_CMD_WRCABCMB: u8 = 0x5E; /* Write CABC minimum brightness */

const _NT35510_COLMOD_RGB565: u8 = 0x55;
const NT35510_COLMOD_RGB888: u8 = 0x77;
