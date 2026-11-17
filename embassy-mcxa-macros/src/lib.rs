//! Host-side implementation of Embassy MCXA assembly macros.
//!
//! Applications should use the frontends re-exported by `embassy-mcxa`, not
//! invoke this implementation crate directly.

use proc_macro::TokenStream;

mod smartdma;

/// Implementation detail of `embassy_mcxa::smartdma_asm!`.
#[proc_macro]
pub fn smartdma_asm(input: TokenStream) -> TokenStream {
    smartdma::expand(input.into(), false)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Implementation detail of `embassy_mcxa::smartdma_asm_file!`.
#[proc_macro]
pub fn smartdma_asm_file(input: TokenStream) -> TokenStream {
    smartdma::expand(input.into(), true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
