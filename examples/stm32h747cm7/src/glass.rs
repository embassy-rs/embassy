//! DSI LCD ST MB1166-A09 module with NT35510 controller

use embassy_stm32::dsihost::panel::DsiPanel;
use embassy_stm32::dsihost::{self, DsiColor, Error};
use embassy_stm32::ltdc::{PolarityActive, PolarityEdge};
use embassy_time::{Duration, Timer};

pub struct Glass;

impl DsiPanel for Glass {
    const ACTIVE_WIDTH: u16 = 480;
    const ACTIVE_HEIGHT: u16 = 800;

    const HSYNC: u16 = 2;
    const HBP: u16 = 34;
    const HFP: u16 = 34;

    const VSYNC: u16 = 12;
    const VBP: u16 = 15;
    const VFP: u16 = 15;

    const HSYNC_POLARITY: PolarityActive = PolarityActive::ActiveHigh;

    const VSYNC_POLARITY: PolarityActive = PolarityActive::ActiveHigh;

    const DATA_ENABLE_POLARITY: PolarityActive = PolarityActive::ActiveHigh;

    const PIXEL_CLOCK_POLARITY: PolarityEdge = PolarityEdge::FallingEdge;

    const NULL_PACKET_SIZE: u16 = 0x0fff;

    const LOOSELY_PACKED: bool = false;

    async fn init<DSI: dsihost::Instance>(dsi: &mut dsihost::DsiHost<'_, DSI>, color: DsiColor) -> Result<(), Error> {
        defmt::debug!("NT35510 init");

        // LV2: Page 1 enable
        dsi.write_cmd(0, 0xF0, &[0x55, 0xAA, 0x52, 0x08, 0x01]).unwrap();

        // Proprietary analog/power init
        dsi.write_cmd(0, 0xB0, &[0x03, 0x03, 0x03])?; // AVDD
        dsi.write_cmd(0, 0xB6, &[0x46, 0x46, 0x46])?; // PCK = 2 x Hsync, BTP = 2.5 x VDDB
        dsi.write_cmd(0, 0xB1, &[0x03, 0x03, 0x03])?; // AVEE
        dsi.write_cmd(0, 0xB7, &[0x36, 0x36, 0x36])?; // AVEE ratio
        dsi.write_cmd(0, 0xB2, &[0x00, 0x00, 0x02])?; // VCL
        dsi.write_cmd(0, 0xB8, &[0x26, 0x26, 0x26])?; // VCL ratio
        dsi.write_cmd(0, 0xBF, &[0x01])?; // VGH
        dsi.write_cmd(0, 0xB3, &[0x09, 0x09, 0x09])?;
        dsi.write_cmd(0, 0xB9, &[0x36, 0x36, 0x36])?; // VGH ratio
        dsi.write_cmd(0, 0xB5, &[0x08, 0x08, 0x08])?; // VGL_REG
        dsi.write_cmd(0, 0xBA, &[0x26, 0x26, 0x26])?; // VGLX ratio
        dsi.write_cmd(0, 0xBC, &[0x00, 0x80, 0x00])?; // VGMP/VGSP
        dsi.write_cmd(0, 0xBD, &[0x00, 0x80, 0x00])?; // VGMN/VGSN
        dsi.write_cmd(0, 0xBE, &[0x00, 0x50])?; // VCOM

        // LV2: Page 0 enable
        dsi.write_cmd(0, 0xF0, &[0x55, 0xAA, 0x52, 0x08, 0x00])?;

        // Proprietary DCS init
        dsi.write_cmd(0, 0xB1, &[0xFC, 0x00])?; // Display optional control
        dsi.write_cmd(0, 0xB6, &[0x03])?; // Source output data hold time
        dsi.write_cmd(0, 0xB5, &[0x50])?; // Display resolution control
        dsi.write_cmd(0, 0xB7, &[0x00, 0x00])?; // Gate EQ control
        dsi.write_cmd(0, 0xB8, &[0x01, 0x02, 0x02, 0x02])?; // Src EQ control
        dsi.write_cmd(0, 0xBC, &[0x00, 0x00, 0x00])?;
        dsi.write_cmd(0, 0xCC, &[0x03, 0x00, 0x00])?;
        dsi.write_cmd(0, 0xBA, &[0x01])?;

        // Required by ST BSP: MADCTL not taken otherwise
        Timer::after(Duration::from_millis(200)).await;

        // Portrait orientation
        dsi.write_cmd(0, 0x36, &[0x00])?; // MADCTL
        dsi.write_cmd(0, 0x2A, &[0x00, 0x00, 0x01, 0xDF])?; // CASET 0..479
        dsi.write_cmd(0, 0x2B, &[0x00, 0x00, 0x03, 0x1F])?; // RASET 0..799

        // Landscape orientation
        //dsi.write_cmd(0, 0x36, &[0x60])?; // MADCTL
        //dsi.write_cmd(0, 0x2A, &[0x00, 0x00, 0x03, 0x1F])?;
        //dsi.write_cmd(0, 0x2B, &[0x00, 0x00, 0x01, 0xDF])?;

        // Wakeup from sleep mode
        dsi.write_cmd(0, 0x11, &[])?;
        Timer::after(Duration::from_millis(20)).await;

        // Pixel format
        let color_code = match color {
            DsiColor::Rgb565Config1 | DsiColor::Rgb565Config2 | DsiColor::Rgb565Config3 => 0x55,
            DsiColor::Rgb666Config1 | DsiColor::Rgb666Config2 => 0x66,
            DsiColor::Rgb888 => 0x77,
        };
        dsi.write_cmd(0, 0x3A, &[color_code])?;

        // Display brightness
        dsi.write_cmd(0, 0x51, &[0xFF])?;

        // Ambient/brightness/gamma setting
        // Bit 0: Gamma in profile, manual when clear
        // Bit 2: backlight on/off
        // Bit 3: Display dimming on/off
        // Bit 5: Brightness control block
        dsi.write_cmd(0, 0x53, &[0x24])?;

        // CABC Control
        // 0 = Off
        // 1 = UI Mode
        // 2 = Still mode
        // 3 = Moving mode
        dsi.write_cmd(0, 0x55, &[0x00])?;

        // Minimum CABC brightness
        dsi.write_cmd(0, 0x5E, &[0xFF])?;

        // Display on
        dsi.write_cmd(0, 0x29, &[])?;

        // Start video-mode frame write handoff
        dsi.write_cmd(0, 0x2C, &[])?;

        Ok(())
    }
}
