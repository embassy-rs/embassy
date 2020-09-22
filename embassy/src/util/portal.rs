use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::mem::MaybeUninit;

use crate::util::*;

/// Utility to call a closure across tasks.
pub struct Portal<T> {
    state: UnsafeCell<State<T>>,
}

enum State<T> {
    None,
    Running,
    Waiting(*mut dyn FnMut(T)),
}

impl<T> Portal<T> {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(State::None),
        }
    }

    pub fn call(&self, val: T) {
        unsafe {
            match *self.state.get() {
                State::None => {}
                State::Running => depanic!("Portall::call() called reentrantly"),
                State::Waiting(func) => (*func)(val),
            }
        }
    }

    pub fn wait_once<'a, R, F>(&'a self, mut func: F) -> impl Future<Output = R> + 'a
    where
        F: FnMut(T) -> R + 'a,
    {
        async move {
            let bomb = DropBomb::new();

            let signal = Signal::new();
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            let mut call_func = |val: T| {
                unsafe {
                    let state = &mut *self.state.get();
                    *state = State::None;
                    result.as_mut_ptr().write(func(val))
                };
                signal.signal(());
            };

            let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
            let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

            unsafe {
                let state = &mut *self.state.get();
                match state {
                    State::None => {}
                    _ => depanic!("Multiple tasks waiting on same portal"),
                }
                *state = State::Waiting(func_ptr);
            }

            signal.wait().await;

            bomb.defuse();

            unsafe { result.assume_init() }
        }
    }

    pub fn wait_many<'a, R, F>(&'a self, mut func: F) -> impl Future<Output = R> + 'a
    where
        F: FnMut(T) -> Option<R> + 'a,
    {
        async move {
            let bomb = DropBomb::new();

            let signal = Signal::new();
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            let mut call_func = |val: T| {
                unsafe {
                    let state = &mut *self.state.get();

                    let func_ptr = match *state {
                        State::Waiting(p) => p,
                        _ => unreachable!(),
                    };

                    // Set state to Running while running the function to avoid reentrancy.
                    *state = State::Running;

                    *state = match func(val) {
                        None => State::Waiting(func_ptr),
                        Some(res) => {
                            result.as_mut_ptr().write(res);
                            signal.signal(());
                            State::None
                        }
                    };
                };
            };

            let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
            let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

            unsafe {
                let state = &mut *self.state.get();
                match *state {
                    State::None => {}
                    _ => depanic!("Multiple tasks waiting on same portal"),
                }
                *state = State::Waiting(func_ptr);
            }

            signal.wait().await;

            bomb.defuse();

            unsafe { result.assume_init() }
        }
    }
}
