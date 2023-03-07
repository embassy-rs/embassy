use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn run(name: syn::Ident) -> Result<TokenStream, TokenStream> {
    let name = format_ident!("{}", name);
    let name_interrupt = format_ident!("{}", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let doc = format!("{} interrupt singleton.", name);

    let result = quote! {
        #[doc = #doc]
        #[allow(non_camel_case_types)]
        pub struct #name_interrupt(());
        unsafe impl ::embassy_cortex_m::interrupt::Interrupt for #name_interrupt {
            fn number(&self) -> u16 {
                use cortex_m::interrupt::InterruptNumber;
                let irq = InterruptEnum::#name;
                irq.number() as u16
            }
            unsafe fn steal() -> Self {
                Self(())
            }
            unsafe fn __handler(&self) -> &'static ::embassy_cortex_m::interrupt::DynHandler {
                #[export_name = #name_handler]
                static HANDLER: ::embassy_cortex_m::interrupt::DynHandler = ::embassy_cortex_m::interrupt::DynHandler::new();
                &HANDLER
            }
        }

        ::embassy_hal_common::impl_peripheral!(#name_interrupt);
    };
    Ok(result)
}
