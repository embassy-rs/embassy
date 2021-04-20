use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_path = embassy_prefix.append("embassy").path();
    let embassy_stm32_path = embassy_prefix.append("embassy_stm32").path();

    quote!(
        use #embassy_stm32_path::{rtc, interrupt, Peripherals, pac, hal::rcc::RccExt, hal::time::U32Ext};

        unsafe { #embassy_stm32_path::system::configure(#config) };

        let (dp, clocks) = Peripherals::take().unwrap();

        let mut rtc = rtc::RTC::new(dp.TIM2, interrupt::take!(TIM2), clocks);
        let rtc = unsafe { make_static(&mut rtc) };
        rtc.start();
        let mut alarm = rtc.alarm1();

        unsafe { #embassy_path::time::set_clock(rtc) };

        let alarm = unsafe { make_static(&mut alarm) };
        executor.set_alarm(alarm);

        unsafe { Peripherals::set_peripherals(clocks) };
    )
}
