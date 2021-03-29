use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
pub enum HfclkSource {
    Internal,
    ExternalXtal,
}

impl Default for HfclkSource {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, FromMeta)]
pub enum LfclkSource {
    InternalRC,
    Synthesized,
    ExternalXtal,
    ExternalLowSwing,
    ExternalFullSwing,
}

impl Default for LfclkSource {
    fn default() -> Self {
        Self::InternalRC
    }
}

#[derive(Debug, FromMeta)]
pub struct Args {
    #[darling(default)]
    pub hfclk_source: HfclkSource,
    #[darling(default)]
    pub lfclk_source: LfclkSource,
}

pub fn generate(args: Args) -> TokenStream {
    let hfclk_source = format_ident!("{}", format!("{:?}", args.hfclk_source));
    let lfclk_source = format_ident!("{}", format!("{:?}", args.lfclk_source));

    quote!(
        use embassy_nrf::{interrupt, peripherals, rtc};

        let mut config = embassy_nrf::system::Config::default();
        config.hfclk_source = embassy_nrf::system::HfclkSource::#hfclk_source;
        config.lfclk_source = embassy_nrf::system::LfclkSource::#lfclk_source;
        unsafe { embassy_nrf::system::configure(config) };

        let mut rtc = rtc::RTC::new(unsafe { <peripherals::RTC1 as embassy::util::Steal>::steal() }, interrupt::take!(RTC1));
        let rtc = unsafe { make_static(&mut rtc) };
        rtc.start();
        let mut alarm = rtc.alarm0();

        unsafe { embassy::time::set_clock(rtc) };

        let alarm = unsafe { make_static(&mut alarm) };
    )
}
