use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
pub struct Args {
    #[darling(default)]
    pub use_hse: Option<u32>,
    #[darling(default)]
    pub sysclk: Option<u32>,
    #[darling(default)]
    pub pclk1: Option<u32>,
}

pub fn generate(args: Args) -> TokenStream {
    let mut clock_cfg_args = quote! {};
    if args.use_hse.is_some() {
        let mhz = args.use_hse.unwrap();
        clock_cfg_args = quote! { #clock_cfg_args.use_hse(#mhz.mhz()) };
    }

    if args.sysclk.is_some() {
        let mhz = args.sysclk.unwrap();
        clock_cfg_args = quote! { #clock_cfg_args.sysclk(#mhz.mhz()) };
    }

    if args.pclk1.is_some() {
        let mhz = args.pclk1.unwrap();
        clock_cfg_args = quote! { #clock_cfg_args.pclk1(#mhz.mhz()) };
    }

    quote!(
        use embassy_stm32::{rtc, interrupt, Peripherals, pac, hal::rcc::RccExt, hal::time::U32Ext};

        let dp = pac::Peripherals::take().unwrap();
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr#clock_cfg_args.freeze();

        unsafe { Peripherals::set_peripherals(clocks) };

        let mut rtc = rtc::RTC::new(dp.TIM3, interrupt::take!(TIM3), clocks);
        let rtc = unsafe { make_static(&mut rtc) };
        rtc.start();
        let mut alarm = rtc.alarm1();

        unsafe { embassy::time::set_clock(rtc) };

        let alarm = unsafe { make_static(&mut alarm) };
    )
}
