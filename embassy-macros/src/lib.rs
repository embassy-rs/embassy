#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::{Diagnostic, Level, Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    pool_size: Option<usize>,
}

#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let args = match MacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let pool_size: usize = args.pool_size.unwrap_or(1);

    let mut fail = false;
    if task_fn.sig.asyncness.is_none() {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("task functions must be async")
            .emit();
        fail = true;
    }
    if task_fn.sig.generics.params.len() != 0 {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("task functions must not be generic")
            .emit();
        fail = true;
    }
    if pool_size < 1 {
        Span::call_site()
            .error("pool_size must be 1 or greater")
            .emit();
        fail = true
    }

    let mut arg_names: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
        syn::punctuated::Punctuated::new();
    let args = &task_fn.sig.inputs;

    for arg in args.iter() {
        match arg {
            syn::FnArg::Receiver(_) => {
                arg.span()
                    .unwrap()
                    .error("task functions must not have receiver arguments")
                    .emit();
                fail = true;
            }
            syn::FnArg::Typed(t) => match t.pat.as_ref() {
                syn::Pat::Ident(i) => arg_names.push(i.ident.clone()),
                _ => {
                    arg.span()
                        .unwrap()
                        .error("pattern matching in task arguments is not yet supporteds")
                        .emit();
                    fail = true;
                }
            },
        }
    }

    if fail {
        return TokenStream::new();
    }

    let name = task_fn.sig.ident.clone();

    let visibility = &task_fn.vis;
    task_fn.sig.ident = format_ident!("task");

    let result = quote! {
        #visibility fn #name(#args) -> ::embassy::executor::SpawnToken {
            #task_fn
            type F = impl ::core::future::Future + 'static;
            static POOL: [::embassy::executor::Task<F>; #pool_size] = [::embassy::executor::Task::new(); #pool_size];
            unsafe { ::embassy::executor::Task::spawn(&POOL, move || task(#arg_names)) }
        }
    };
    result.into()
}

#[proc_macro]
pub fn interrupt_declare(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    let name = format_ident!("{}", name);
    let name_interrupt = format_ident!("{}Interrupt", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let result = quote! {
        #[allow(non_camel_case_types)]
        pub struct #name_interrupt(());
        unsafe impl OwnedInterrupt for #name_interrupt {
            type Priority = Priority;
            fn number(&self) -> u8 {
                Interrupt::#name as u8
            }
            unsafe fn __handler(&self) -> &'static ::core::sync::atomic::AtomicPtr<u32> {
                #[export_name = #name_handler]
                static HANDLER: ::core::sync::atomic::AtomicPtr<u32> = ::core::sync::atomic::AtomicPtr::new(::core::ptr::null_mut());
                &HANDLER
            }
        }
    };
    result.into()
}

#[proc_macro]
pub fn interrupt_take(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    let name = format!("{}", name);
    let name_interrupt = format_ident!("{}Interrupt", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let result = quote! {
        {
            #[allow(non_snake_case)]
            #[export_name = #name]
            pub unsafe extern "C" fn trampoline() {
                extern "C" {
                    #[link_name = #name_handler]
                    static HANDLER: ::core::sync::atomic::AtomicPtr<u32>;
                }

                let p = HANDLER.load(::core::sync::atomic::Ordering::Acquire);
                if !p.is_null() {
                    let f: fn() = ::core::mem::transmute(p);
                    f()
                }
            }

            static TAKEN: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);

            if TAKEN.compare_and_swap(false, true, ::core::sync::atomic::Ordering::AcqRel) {
                panic!("IRQ Already taken");
            }

            let irq: interrupt::#name_interrupt = unsafe { ::core::mem::transmute(()) };
            irq
        }
    };
    result.into()
}
