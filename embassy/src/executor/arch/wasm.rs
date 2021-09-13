use core::marker::PhantomData;
use js_sys::Promise;
use wasm_bindgen::prelude::*;

use super::{
    raw::{self, util::UninitCell},
    Spawner,
};

/// WASM executor, wasm_bindgen to schedule tasks on the JS event loop.
pub struct Executor {
    inner: raw::Executor,
    ctx: &'static WasmContext,
    not_send: PhantomData<*mut ()>,
}

pub(crate) struct WasmContext {
    promise: Promise,
    closure: UninitCell<Closure<dyn FnMut(JsValue)>>,
}

impl WasmContext {
    pub fn new() -> Self {
        Self {
            promise: Promise::resolve(&JsValue::undefined()),
            closure: UninitCell::uninit(),
        }
    }
}

impl Executor {
    /// Create a new Executor.
    pub fn new() -> Self {
        let ctx = &*Box::leak(Box::new(WasmContext::new()));
        let inner = raw::Executor::new(
            |p| unsafe {
                let ctx = &*(p as *const () as *const WasmContext);
                let _ = ctx.promise.then(ctx.closure.as_mut());
            },
            ctx as *const _ as _,
        );
        Self {
            inner,
            not_send: PhantomData,
            ctx,
        }
    }

    /// Run the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [Forever](crate::util::Forever) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    pub fn start(&'static mut self, init: impl FnOnce(Spawner)) {
        unsafe {
            let executor = &self.inner;
            self.ctx.closure.write(Closure::new(move |_| {
                executor.poll();
            }));
            init(self.inner.spawner());
        }
    }
}
