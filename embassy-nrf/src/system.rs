use crate::pac;

pub enum HfclkSource {
    Internal,
    ExternalXtal,
}

pub enum LfclkSource {
    InternalRC,
    Synthesized,
    ExternalXtal,
    ExternalLowSwing,
    ExternalFullSwing,
}

#[non_exhaustive]
pub struct Config {
    pub hfclk_source: HfclkSource,
    pub lfclk_source: LfclkSource,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // There are hobby nrf52 boards out there without external XTALs...
            // Default everything to internal so it Just Works. User can enable external
            // xtals if they know they have them.
            hfclk_source: HfclkSource::Internal,
            lfclk_source: LfclkSource::InternalRC,
        }
    }
}

/// safety: must only call once.
pub unsafe fn configure(config: Config) {
    let r = &*pac::CLOCK::ptr();

    // Start HFCLK.
    match config.hfclk_source {
        HfclkSource::Internal => {}
        HfclkSource::ExternalXtal => {
            // Datasheet says this is likely to take 0.36ms
            r.events_hfclkstarted.write(|w| unsafe { w.bits(0) });
            r.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
            while r.events_hfclkstarted.read().bits() == 0 {}
        }
    }

    // Configure LFCLK.
    match config.lfclk_source {
        LfclkSource::InternalRC => r.lfclksrc.write(|w| w.src().rc()),
        LfclkSource::Synthesized => r.lfclksrc.write(|w| w.src().synth()),

        LfclkSource::ExternalXtal => r.lfclksrc.write(move |w| w.src().xtal()),

        LfclkSource::ExternalLowSwing => r.lfclksrc.write(move |w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().disabled();
            w
        }),
        LfclkSource::ExternalFullSwing => r.lfclksrc.write(move |w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().enabled();
            w
        }),
    }

    // Start LFCLK.
    // Datasheet says this could take 100us from synth source
    // 600us from rc source, 0.25s from an external source.
    r.events_lfclkstarted.write(|w| unsafe { w.bits(0) });
    r.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
    while r.events_lfclkstarted.read().bits() == 0 {}

    // Init GPIOTE
    crate::gpiote::init();
}
