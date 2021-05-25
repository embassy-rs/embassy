#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::iter;
use syn::spanned::Spanned;
use syn::{parse, Type, Visibility};
use syn::{ItemFn, ReturnType};

mod path;

use path::ModulePrefix;

#[derive(Debug, FromMeta)]
struct TaskArgs {
    #[darling(default)]
    pool_size: Option<usize>,
    #[darling(default)]
    send: bool,
    #[darling(default)]
    embassy_prefix: ModulePrefix,
}

#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let macro_args = match TaskArgs::from_list(&macro_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let embassy_prefix = macro_args.embassy_prefix.append("embassy");
    let embassy_path = embassy_prefix.path();

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
        return parse::Error::new(Span::call_site(), "pool_size must be 1 or greater")
            .to_compile_error()
            .into();
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
        #visibility fn #name(#args) -> #embassy_path::executor::SpawnToken<#impl_ty> {
            use #embassy_path::executor::raw::Task;
            #task_fn
            type F = #impl_ty;
            const NEW_TASK: Task<F> = Task::new();
            static POOL: [Task<F>; #pool_size] = [NEW_TASK; #pool_size];
            unsafe { Task::spawn_pool(&POOL, move || task(#arg_names)) }
        }
    };
    result.into()
}

#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f: ItemFn = syn::parse(input).expect("`#[interrupt]` must be applied to a function");

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let fspan = f.span();
    let ident = f.sig.ident.clone();
    let ident_s = ident.to_string();

    // XXX should we blacklist other attributes?

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                Type::Never(..) => true,
                _ => false,
            },
        };

    if !valid_signature {
        return parse::Error::new(
            fspan,
            "`#[interrupt]` handlers must have signature `[unsafe] fn() [-> !]`",
        )
        .to_compile_error()
        .into();
    }

    f.block.stmts = iter::once(
        syn::parse2(quote! {{
            // Check that this interrupt actually exists
            let __irq_exists_check: interrupt::#ident;
        }})
        .unwrap(),
    )
    .chain(f.block.stmts)
    .collect();

    quote!(
        #[doc(hidden)]
        #[export_name = #ident_s]
        #[allow(non_snake_case)]
        #f
    )
    .into()
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

#[cfg(feature = "stm32")]
#[path = "chip/stm32.rs"]
mod chip;

#[cfg(feature = "nrf")]
#[path = "chip/nrf.rs"]
mod chip;

#[cfg(feature = "rp")]
#[path = "chip/rp.rs"]
mod chip;

#[derive(Debug, FromMeta)]
struct MainArgs {
    #[darling(default)]
    embassy_prefix: ModulePrefix,

    #[darling(default)]
    config: Option<syn::LitStr>,
}

#[cfg(any(feature = "nrf", feature = "rp", feature = "stm32"))]
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let macro_args = match MainArgs::from_list(&macro_args) {
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

    if args.len() != 2 {
        task_fn
            .sig
            .span()
            .unwrap()
            .error("main function must have 2 arguments")
            .emit();
        fail = true;
    }

    if fail {
        return TokenStream::new();
    }

    let embassy_prefix = macro_args.embassy_prefix;
    let embassy_prefix_lit = embassy_prefix.literal();
    let embassy_path = embassy_prefix.append("embassy").path();
    let task_fn_body = task_fn.block;

    let config = macro_args
        .config
        .map(|s| s.parse::<syn::Expr>().unwrap())
        .unwrap_or_else(|| {
            syn::Expr::Verbatim(quote! {
                Default::default()
            })
        });

    let chip_setup = chip::generate(&embassy_prefix, config);

    let result = quote! {
        #[#embassy_path::task(embassy_prefix = #embassy_prefix_lit)]
        async fn __embassy_main(#args) {
            #task_fn_body
        }

        #[cortex_m_rt::entry]
        fn main() -> ! {
            unsafe fn make_static<T>(t: &mut T) -> &'static mut T {
                ::core::mem::transmute(t)
            }

            let mut executor = #embassy_path::executor::Executor::new();

            let executor = unsafe { make_static(&mut executor) };

            #chip_setup

            executor.run(|spawner| {
                spawner.spawn(__embassy_main(spawner, p)).unwrap();
            })

        }
    };
    result.into()
}

#[cfg(feature = "std")]
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let macro_args = match MainArgs::from_list(&macro_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let embassy_path = macro_args.embassy_prefix.append("embassy");
    let embassy_std_path = macro_args.embassy_prefix.append("embassy_std");

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

    let embassy_path = embassy_path.path();
    let embassy_std_path = embassy_std_path.path();
    let embassy_prefix_lit = macro_args.embassy_prefix.literal();

    let result = quote! {
        #[#embassy_path::task(embassy_prefix = #embassy_prefix_lit)]
        async fn __embassy_main(#args) {
            #task_fn_body
        }

        fn main() -> ! {
            unsafe fn make_static<T>(t: &mut T) -> &'static mut T {
                ::core::mem::transmute(t)
            }

            let mut executor = #embassy_std_path::Executor::new();
            let executor = unsafe { make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            })

        }
    };
    result.into()
}
