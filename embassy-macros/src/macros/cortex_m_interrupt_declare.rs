use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn run(name: syn::Ident) -> Result<TokenStream, TokenStream> {
    let name = format_ident!("{}", name);
    let doc = format!("{} interrupt.", name);

    let result = quote! {
        #[doc = #doc]
        #[allow(non_camel_case_types)]
        pub enum #name{}
        unsafe impl ::embassy_cortex_m::interrupt::Interrupt for #name {
            fn number() -> u16 {
                use cortex_m::interrupt::InterruptNumber;
                let irq = InterruptEnum::#name;
                irq.number() as u16
            }
        }
    };
    Ok(result)
}
