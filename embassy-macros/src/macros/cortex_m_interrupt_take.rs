use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn run(name: syn::Ident) -> Result<TokenStream, TokenStream> {
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
                    static HANDLER: interrupt::Handler;
                }

                let func = HANDLER.func.load(::embassy_executor::export::atomic::Ordering::Relaxed);
                let ctx = HANDLER.ctx.load(::embassy_executor::export::atomic::Ordering::Relaxed);
                let func: fn(*mut ()) = ::core::mem::transmute(func);
                ::embassy_executor::rtos_trace_interrupt! {
                    ::embassy_executor::export::trace::isr_enter();
                }
                func(ctx);
                ::embassy_executor::rtos_trace_interrupt! {
                    ::embassy_executor::export::trace::isr_exit();
                }
            }

            static TAKEN: ::embassy_executor::export::atomic::AtomicBool = ::embassy_executor::export::atomic::AtomicBool::new(false);

            if TAKEN.compare_exchange(false, true, ::embassy_executor::export::atomic::Ordering::AcqRel, ::embassy_executor::export::atomic::Ordering::Acquire).is_err() {
                core::panic!("IRQ Already taken");
            }

            let irq: interrupt::#name_interrupt = unsafe { ::core::mem::transmute(()) };
            irq
        }
    };
    Ok(result)
}
