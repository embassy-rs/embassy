//! SYSCTL configuration for MSPM0C1105/6 SYSCTL.

use mspm0_metapac::sysctl::vals;

use crate::sysctl::{ClkOutDiv, div_to_pac};

/// Source and configuration for CLK_OUT pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClkOutSource {
    /// Use SYSOSC as the source.
    ///
    /// The divider is optional for this clock source.
    Sysosc(Option<ClkOutDiv>),

    /// Use ULPCLK as the source.
    ///
    /// The divider is required for this clock source.
    UlpClk(ClkOutDiv),

    /// Use LFCLK as the source.
    ///
    /// The divider is optional for this clock source.
    LfClk(Option<ClkOutDiv>),

    /// Use MFPCLK as the source.
    ///
    /// The divider is required for this clock source.
    MfpClk(ClkOutDiv),

    /// Use HFCLK as the source.
    ///
    /// The divider is optional for this clock source.
    Hfclk(Option<ClkOutDiv>),
}

impl ClkOutSource {
    pub(super) fn convert_div(self) -> (bool, vals::Exclkdivval) {
        match self {
            ClkOutSource::Sysosc(div) => div_to_pac(div),
            ClkOutSource::UlpClk(div) => div_to_pac(Some(div)),
            ClkOutSource::LfClk(div) => div_to_pac(div),
            ClkOutSource::MfpClk(div) => div_to_pac(Some(div)),
            ClkOutSource::Hfclk(div) => div_to_pac(div),
        }
    }

    pub(super) fn convert_src(self) -> vals::Exclksrc {
        match self {
            ClkOutSource::Sysosc(_) => vals::Exclksrc::SYSOSC,
            ClkOutSource::UlpClk(_) => vals::Exclksrc::ULPCLK,
            ClkOutSource::LfClk(_) => vals::Exclksrc::LFCLK,
            // FIXME: Wrong name from SVD
            ClkOutSource::MfpClk(_) => vals::Exclksrc::MFCLK,
            ClkOutSource::Hfclk(_) => vals::Exclksrc::HFCLK,
        }
    }
}
