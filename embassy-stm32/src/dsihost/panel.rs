//! DSI Panel Configuration

use super::mode::DsiColor;
use super::{DsiHost, Error, Instance};
use crate::ltdc::{LtdcConfiguration, PolarityActive, PolarityEdge};

/// DSI Panel trait
///
/// This trait must be implemented by supported panels to provide
/// timing information and perform a vendor specific init sequence.
pub trait DsiPanel {
    /// Active panel width in pixels
    const ACTIVE_WIDTH: u16;

    /// Active panel height in pixels
    const ACTIVE_HEIGHT: u16;

    /// Horizontal back porch
    const HBP: u16;

    /// Horizontal front porch
    const HFP: u16;

    /// Vertical back porch
    const VBP: u16;

    /// Vertical front porch
    const VFP: u16;

    /// Horizontal sync width
    const HSYNC: u16;

    /// Vertical sync width
    const VSYNC: u16;

    /// Horizontal sync polarity
    const HSYNC_POLARITY: PolarityActive;

    /// Vertical sync polarity
    const VSYNC_POLARITY: PolarityActive;

    /// Data enable polarity
    const DATA_ENABLE_POLARITY: PolarityActive;

    /// Pixel clock edge polarity
    const PIXEL_CLOCK_POLARITY: PolarityEdge;

    /// Size of a null packet in Sync Pulse or Event modes
    const NULL_PACKET_SIZE: u16;

    /// Enable loose packed in RGB666 Video mode
    const LOOSELY_PACKED: bool;

    /// Total horizontal line
    const HLINE_TOTAL: u16 = Self::ACTIVE_WIDTH + Self::HSYNC + Self::HBP + Self::HFP;

    /// Run panel specific DSI command initialization
    async fn init<DSI: super::Instance>(dsi: &mut DsiHost<'_, DSI>, color: DsiColor) -> Result<(), Error>;

    /// Return an [`LtdcConfiguration`] for this panel
    ///
    /// NOTE: The polarities are set for communication between LTDC and DSIHOST
    /// and are not the values specific to the panel, which are handled by DSI.
    fn ltdc_config() -> LtdcConfiguration {
        LtdcConfiguration {
            active_width: Self::ACTIVE_WIDTH,
            active_height: Self::ACTIVE_HEIGHT,
            h_back_porch: Self::HBP,
            h_front_porch: Self::HFP,
            v_back_porch: Self::VBP,
            v_front_porch: Self::VFP,
            h_sync: Self::HSYNC,
            v_sync: Self::VSYNC,

            h_sync_polarity: PolarityActive::ActiveLow,
            v_sync_polarity: PolarityActive::ActiveLow,
            data_enable_polarity: PolarityActive::ActiveLow,
            pixel_clock_polarity: PolarityEdge::FallingEdge,
        }
    }
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Initialize the panel using the generic type implementing [`DsiPanel`]
    pub async fn init_panel<Panel: DsiPanel>(&mut self, color: DsiColor) -> Result<(), Error> {
        Panel::init::<T>(self, color).await
    }
}
