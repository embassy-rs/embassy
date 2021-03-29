use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromMeta)]
pub struct Args {}

pub fn generate(_args: Args) -> TokenStream {
    quote!(
        use embassy_rp::{interrupt, peripherals};

        let mut config = embassy_rp::system::Config::default();
        unsafe { embassy_rp::system::configure(config) };
    )
}
