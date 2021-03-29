use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
pub struct Args {}

pub fn generate(args: Args) -> TokenStream {
    quote!(
        use embassy_rp::{interrupt, peripherals};

        let mut config = embassy_rp::system::Config::default();
        unsafe { embassy_rp::system::configure(config) };
    )
}
