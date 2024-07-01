#![doc = include_str!("../README.md")]
extern crate proc_macro;

use darling::ast::NestedMeta;
use proc_macro::TokenStream;

mod macros;
mod util;
use macros::*;
use syn::parse::{Parse, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::Token;

struct Args {
    meta: Vec<NestedMeta>,
}

impl Parse for Args {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let meta = Punctuated::<NestedMeta, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            meta: meta.into_iter().collect(),
        })
    }
}

/// Declares an async task that can be run by `embassy-executor`. The optional `pool_size` parameter can be used to specify how
/// many concurrent tasks can be spawned (default is 1) for the function.
///
///
/// The following restrictions apply:
///
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * The optional `pool_size` attribute must be 1 or greater.
///
///
/// ## Examples
///
/// Declaring a task taking no arguments:
///
/// ``` rust
/// #[embassy_executor::task]
/// async fn mytask() {
///     // Function body
/// }
/// ```
///
/// Declaring a task with a given pool size:
///
/// ``` rust
/// #[embassy_executor::task(pool_size = 4)]
/// async fn mytask() {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);

    task::run(&args.meta, f).unwrap_or_else(|x| x).into()
}

#[proc_macro_attribute]
pub fn main_avr(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(&args.meta, f, main::avr()).unwrap_or_else(|x| x).into()
}

/// Creates a new `executor` instance and declares an application entry point for Cortex-M spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_cortex_m(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(&args.meta, f, main::cortex_m()).unwrap_or_else(|x| x).into()
}

/// Creates a new `executor` instance and declares an application entry point for RISC-V spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// A user-defined entry macro can be optionally provided via the `entry` argument to override the default of `riscv_rt::entry`.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
///
/// Spawning a task using a custom entry macro:
/// ``` rust
/// #[embassy_executor::main(entry = "esp_riscv_rt::entry")]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_riscv(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(&args.meta, f, main::riscv(&args.meta))
        .unwrap_or_else(|x| x)
        .into()
}

/// Creates a new `executor` instance and declares an application entry point for STD spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_std(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(&args.meta, f, main::std()).unwrap_or_else(|x| x).into()
}

/// Creates a new `executor` instance and declares an application entry point for WASM spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_wasm(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    main::run(&args.meta, f, main::wasm()).unwrap_or_else(|x| x).into()
}
