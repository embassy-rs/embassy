use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ReturnType, Type};

use crate::util::ctxt::Ctxt;

#[derive(Debug, FromMeta)]
struct Args {}

pub fn riscv() -> TokenStream {
    quote! {
        #[riscv_rt::entry]
        fn main() -> ! {
            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };
            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    }
}

pub fn cortex_m() -> TokenStream {
    quote! {
        #[cortex_m_rt::entry]
        fn main() -> ! {
            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };
            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    }
}

pub fn wasm() -> TokenStream {
    quote! {
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() -> Result<(), wasm_bindgen::JsValue> {
            static EXECUTOR: ::embassy_executor::_export::StaticCell<::embassy_executor::Executor> = ::embassy_executor::_export::StaticCell::new();
            let executor = EXECUTOR.init(::embassy_executor::Executor::new());

            executor.start(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            });

            Ok(())
        }
    }
}

pub fn std() -> TokenStream {
    quote! {
        fn main() -> ! {
            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    }
}

pub fn run(args: syn::AttributeArgs, f: syn::ItemFn, main: TokenStream) -> Result<TokenStream, TokenStream> {
    #[allow(unused_variables)]
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let fargs = f.sig.inputs.clone();

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "main function must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "main function must not be generic");
    }
    if !f.sig.generics.where_clause.is_none() {
        ctxt.error_spanned_by(&f.sig, "main function must not have `where` clauses");
    }
    if !f.sig.abi.is_none() {
        ctxt.error_spanned_by(&f.sig, "main function must not have an ABI qualifier");
    }
    if !f.sig.variadic.is_none() {
        ctxt.error_spanned_by(&f.sig, "main function must not be variadic");
    }
    match &f.sig.output {
        ReturnType::Default => {}
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => {}
            Type::Never(_) => {}
            _ => ctxt.error_spanned_by(
                &f.sig,
                "main function must either not return a value, return `()` or return `!`",
            ),
        },
    }

    if fargs.len() != 1 {
        ctxt.error_spanned_by(&f.sig, "main function must have 1 argument: the spawner.");
    }

    ctxt.check()?;

    let f_body = f.block;
    let out = &f.sig.output;

    let result = quote! {
        #[::embassy_executor::task()]
        async fn __embassy_main(#fargs) #out {
            #f_body
        }

        unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
            ::core::mem::transmute(t)
        }

        #main
    };

    Ok(result)
}
