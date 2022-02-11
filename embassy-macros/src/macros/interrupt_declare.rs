use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn run(name: syn::Ident) -> Result<TokenStream, TokenStream> {
    let name = format_ident!("{}", name);
    let name_interrupt = format_ident!("{}", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let result = quote! {
        #[allow(non_camel_case_types)]
        pub struct #name_interrupt(());
        unsafe impl ::embassy::interrupt::Interrupt for #name_interrupt {
            type Priority = crate::interrupt::Priority;
            fn number(&self) -> u16 {
                use cortex_m::interrupt::InterruptNumber;
                let irq = InterruptEnum::#name;
                irq.number() as u16
            }
            unsafe fn steal() -> Self {
                Self(())
            }
            unsafe fn __handler(&self) -> &'static ::embassy::interrupt::Handler {
                #[export_name = #name_handler]
                static HANDLER: ::embassy::interrupt::Handler = ::embassy::interrupt::Handler::new();
                &HANDLER
            }
        }

        unsafe impl ::embassy::util::Unborrow for #name_interrupt {
            type Target = #name_interrupt;
            unsafe fn unborrow(self) -> #name_interrupt {
                self
            }
        }
    };
    Ok(result)
}
