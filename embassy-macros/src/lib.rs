extern crate proc_macro;

use proc_macro::TokenStream;

mod macros;
mod util;
use macros::*;

#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let f = syn::parse_macro_input!(item as syn::ItemFn);

    task::run(args, f).unwrap_or_else(|x| x).into()
}

#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(args, f).unwrap_or_else(|x| x).into()
}

#[proc_macro_attribute]
pub fn cortex_m_interrupt(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    cortex_m_interrupt::run(args, f).unwrap_or_else(|x| x).into()
}

#[proc_macro]
pub fn cortex_m_interrupt_declare(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    cortex_m_interrupt_declare::run(name).unwrap_or_else(|x| x).into()
}

/// # interrupt_take procedural macro
///
/// core::panic! is used as a default way to panic in this macro as there is no sensible way of enabling/disabling defmt for macro generation.
/// We are aware that this brings bloat in the form of core::fmt, but the bloat is already included with e.g. array indexing panics.
/// To get rid of this bloat, use the compiler flags `-Zbuild-std=core -Zbuild-std-features=panic_immediate_abort`.
#[proc_macro]
pub fn cortex_m_interrupt_take(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    cortex_m_interrupt_take::run(name).unwrap_or_else(|x| x).into()
}
