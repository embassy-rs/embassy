#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    pool_size: Option<usize>,
    #[darling(default)]
    send: bool,
}

#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let macro_args = match MacroArgs::from_list(&macro_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let pool_size: usize = macro_args.pool_size.unwrap_or(1);

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
    if !task_fn.sig.generics.params.is_empty() {
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
    let mut args = task_fn.sig.inputs.clone();

    for arg in args.iter_mut() {
        match arg {
            syn::FnArg::Receiver(_) => {
                arg.span()
                    .unwrap()
                    .error("task functions must not have receiver arguments")
                    .emit();
                fail = true;
            }
            syn::FnArg::Typed(t) => match t.pat.as_mut() {
                syn::Pat::Ident(i) => {
                    arg_names.push(i.ident.clone());
                    i.mutability = None;
                }
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
    let impl_ty = if macro_args.send {
        quote!(impl ::core::future::Future + Send + 'static)
    } else {
        quote!(impl ::core::future::Future + 'static)
    };

    let result = quote! {
        #visibility fn #name(#args) -> ::embassy::executor::SpawnToken<#impl_ty> {
            use ::embassy::executor::raw::Task;
            #task_fn
            type F = #impl_ty;
            const NEW_TASK: Task<F> = Task::new();
            static POOL: [Task<F>; #pool_size] = [NEW_TASK; #pool_size];
            unsafe { Task::spawn_pool(&POOL, move || task(#arg_names)) }
        }
    };
    result.into()
}

#[proc_macro]
pub fn interrupt_declare(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    let name = format_ident!("{}", name);
    let name_interrupt = format_ident!("{}", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let result = quote! {
        #[allow(non_camel_case_types)]
        pub struct #name_interrupt(());
        unsafe impl Interrupt for #name_interrupt {
            type Priority = crate::interrupt::Priority;
            fn number(&self) -> u16 {
                use cortex_m::interrupt::InterruptNumber;
                let irq = crate::pac::Interrupt::#name;
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

        impl ::embassy::util::PeripheralBorrow for #name_interrupt {
            type Target = #name_interrupt;
            unsafe fn unborrow(self) -> #name_interrupt {
                self
            }
        }

        impl ::embassy::util::PeripheralBorrow for &mut #name_interrupt {
            type Target = #name_interrupt;
            unsafe fn unborrow(self) -> #name_interrupt {
                ::core::ptr::read(self)
            }
        }
    };
    result.into()
}

#[proc_macro]
pub fn interrupt_take(item: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(item as syn::Ident);
    let name = format!("{}", name);
    let name_interrupt = format_ident!("{}", name);
    let name_handler = format!("__EMBASSY_{}_HANDLER", name);

    let result = quote! {
        {
            #[allow(non_snake_case)]
            #[export_name = #name]
            pub unsafe extern "C" fn trampoline() {
                extern "C" {
                    #[link_name = #name_handler]
                    static HANDLER: ::embassy::interrupt::Handler;
                }

                let func = HANDLER.func.load(::embassy::export::atomic::Ordering::Relaxed);
                let ctx = HANDLER.ctx.load(::embassy::export::atomic::Ordering::Relaxed);
                let func: fn(*mut ()) = ::core::mem::transmute(func);
                func(ctx)
            }

            static TAKEN: ::embassy::export::atomic::AtomicBool = ::embassy::export::atomic::AtomicBool::new(false);

            if TAKEN.compare_exchange(false, true, ::embassy::export::atomic::Ordering::AcqRel, ::embassy::export::atomic::Ordering::Acquire).is_err() {
                panic!("IRQ Already taken");
            }

            let irq: interrupt::#name_interrupt = unsafe { ::core::mem::transmute(()) };
            irq
        }
    };
    result.into()
}

#[cfg(feature = "nrf")]
#[path = "chip/nrf.rs"]
mod chip;

#[cfg(feature = "stm32")]
#[path = "chip/stm32.rs"]
mod chip;

#[cfg(feature = "rp")]
#[path = "chip/rp.rs"]
mod chip;

#[cfg(any(feature = "nrf", feature = "stm32", feature = "rp"))]
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let macro_args = match chip::Args::from_list(&macro_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

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
    if !task_fn.sig.generics.params.is_empty() {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("main function must not be generic")
            .emit();
        fail = true;
    }

    let args = task_fn.sig.inputs.clone();

    if args.len() != 1 {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("main function must have one argument")
            .emit();
        fail = true;
    }

    if fail {
        return TokenStream::new();
    }

    let task_fn_body = task_fn.block.clone();
    let chip_setup = chip::generate(macro_args);

    let result = quote! {
        #[embassy::task]
        async fn __embassy_main(#args) {
            #task_fn_body
        }

        #[cortex_m_rt::entry]
        fn main() -> ! {
            unsafe fn make_static<T>(t: &mut T) -> &'static mut T {
                ::core::mem::transmute(t)
            }

            let mut executor = ::embassy::executor::Executor::new();
            let executor = unsafe { make_static(&mut executor) };

            #chip_setup

            executor.run(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            })

        }
    };
    result.into()
}

#[cfg(not(any(feature = "nrf", feature = "stm32", feature = "rp")))]
#[proc_macro_attribute]
pub fn main(_: TokenStream, item: TokenStream) -> TokenStream {
    let task_fn = syn::parse_macro_input!(item as syn::ItemFn);

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
    if !task_fn.sig.generics.params.is_empty() {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("main function must not be generic")
            .emit();
        fail = true;
    }

    let args = task_fn.sig.inputs.clone();

    if args.len() != 1 {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("main function must have one argument")
            .emit();
        fail = true;
    }

    if fail {
        return TokenStream::new();
    }

    let task_fn_body = task_fn.block.clone();

    let result = quote! {
        #[embassy::task]
        async fn __embassy_main(#args) {
            #task_fn_body
        }

        fn main() -> ! {
            unsafe fn make_static<T>(t: &mut T) -> &'static mut T {
                ::core::mem::transmute(t)
            }

            let mut executor = ::embassy_std::Executor::new();
            let executor = unsafe { make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            })

        }
    };
    result.into()
}
