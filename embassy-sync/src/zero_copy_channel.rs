use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

pub struct Channel<'a, M: RawMutex, T> {
    buf: *mut T,
    phantom: PhantomData<&'a mut T>,
    state: Mutex<M, RefCell<State>>,
}

impl<'a, M: RawMutex, T> Channel<'a, M, T> {
    pub fn new(buf: &'a mut [T]) -> Self {
        let len = buf.len();
        assert!(len != 0);

        Self {
            buf: buf.as_mut_ptr(),
            phantom: PhantomData,
            state: Mutex::new(RefCell::new(State {
                len,
                front: 0,
                back: 0,
                full: false,
                send_waker: WakerRegistration::new(),
                receive_waker: WakerRegistration::new(),
            })),
        }
    }

    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        (Sender { channel: self }, Receiver { channel: self })
    }
}

pub struct Sender<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
}

impl<'a, M: RawMutex, T> Sender<'a, M, T> {
    pub fn borrow(&mut self) -> Sender<'_, M, T> {
        Sender { channel: self.channel }
    }

    pub fn try_send(&mut self) -> Option<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.push_index() {
                Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                None => None,
            }
        })
    }

    pub fn poll_send(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.push_index() {
                Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
                None => {
                    s.receive_waker.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    pub async fn send(&mut self) -> &mut T {
        let i = poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.push_index() {
                    Some(i) => Poll::Ready(i),
                    None => {
                        s.receive_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await;
        unsafe { &mut *self.channel.buf.add(i) }
    }

    pub fn send_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().push_done())
    }
}
pub struct Receiver<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
}

impl<'a, M: RawMutex, T> Receiver<'a, M, T> {
    pub fn borrow(&mut self) -> Receiver<'_, M, T> {
        Receiver { channel: self.channel }
    }

    pub fn try_receive(&mut self) -> Option<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.pop_index() {
                Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                None => None,
            }
        })
    }

    pub fn poll_receive(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.pop_index() {
                Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
                None => {
                    s.send_waker.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    pub async fn receive(&mut self) -> &mut T {
        let i = poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.pop_index() {
                    Some(i) => Poll::Ready(i),
                    None => {
                        s.send_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await;
        unsafe { &mut *self.channel.buf.add(i) }
    }

    pub fn receive_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().pop_done())
    }
}

struct State {
    len: usize,

    /// Front index. Always 0..=(N-1)
    front: usize,
    /// Back index. Always 0..=(N-1).
    back: usize,

    /// Used to distinguish "empty" and "full" cases when `front == back`.
    /// May only be `true` if `front == back`, always `false` otherwise.
    full: bool,

    send_waker: WakerRegistration,
    receive_waker: WakerRegistration,
}

impl State {
    fn increment(&self, i: usize) -> usize {
        if i + 1 == self.len {
            0
        } else {
            i + 1
        }
    }

    fn is_full(&self) -> bool {
        self.full
    }

    fn is_empty(&self) -> bool {
        self.front == self.back && !self.full
    }

    fn push_index(&mut self) -> Option<usize> {
        match self.is_full() {
            true => None,
            false => Some(self.back),
        }
    }

    fn push_done(&mut self) {
        assert!(!self.is_full());
        self.back = self.increment(self.back);
        if self.back == self.front {
            self.full = true;
        }
        self.send_waker.wake();
    }

    fn pop_index(&mut self) -> Option<usize> {
        match self.is_empty() {
            true => None,
            false => Some(self.front),
        }
    }

    fn pop_done(&mut self) {
        assert!(!self.is_empty());
        self.front = self.increment(self.front);
        self.full = false;
        self.receive_waker.wake();
    }
}
