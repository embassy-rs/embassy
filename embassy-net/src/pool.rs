#![macro_use]

use as_slice::{AsMutSlice, AsSlice};
use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::fmt::{assert, *};

struct AtomicBitset<const N: usize>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    used: [AtomicU32; (N + 31) / 32],
}

impl<const N: usize> AtomicBitset<N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    const fn new() -> Self {
        const Z: AtomicU32 = AtomicU32::new(0);
        Self {
            used: [Z; (N + 31) / 32],
        }
    }

    fn alloc(&self) -> Option<usize> {
        for (i, val) in self.used.iter().enumerate() {
            let res = val.fetch_update(Ordering::AcqRel, Ordering::Acquire, |val| {
                let n = val.trailing_ones() as usize + i * 32;
                if n >= N {
                    None
                } else {
                    Some(val | (1 << n))
                }
            });
            if let Ok(val) = res {
                let n = val.trailing_ones() as usize + i * 32;
                return Some(n);
            }
        }
        None
    }
    fn free(&self, i: usize) {
        assert!(i < N);
        self.used[i / 32].fetch_and(!(1 << ((i % 32) as u32)), Ordering::AcqRel);
    }
}

pub trait Pool<T> {
    fn alloc(&self) -> Option<*mut T>;
    unsafe fn free(&self, p: *mut T);
}

pub struct BitPool<T, const N: usize>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    used: AtomicBitset<N>,
    data: MaybeUninit<[T; N]>,
}

impl<T, const N: usize> BitPool<T, N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    pub const fn new() -> Self {
        Self {
            used: AtomicBitset::new(),
            data: MaybeUninit::uninit(),
        }
    }
}

impl<T, const N: usize> Pool<T> for BitPool<T, N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    fn alloc(&self) -> Option<*mut T> {
        let n = self.used.alloc()?;
        let origin = self.data.as_ptr() as *mut T;
        Some(unsafe { origin.add(n) })
    }

    /// safety: p must be a pointer obtained from self.alloc that hasn't been freed yet.
    unsafe fn free(&self, p: *mut T) {
        let origin = self.data.as_ptr() as *mut T;
        let n = p.offset_from(origin);
        assert!(n >= 0);
        assert!((n as usize) < N);
        self.used.free(n as usize);
    }
}

pub trait StaticPool: 'static {
    type Item: 'static;
    type Pool: Pool<Self::Item>;
    fn get() -> &'static Self::Pool;
}

pub struct Box<P: StaticPool> {
    ptr: *mut P::Item,
}

impl<P: StaticPool> Box<P> {
    pub fn new(item: P::Item) -> Option<Self> {
        let p = match P::get().alloc() {
            Some(p) => p,
            None => {
                warn!("alloc failed!");
                return None;
            }
        };
        //trace!("allocated {:u32}", p as u32);
        unsafe { p.write(item) };
        Some(Self { ptr: p })
    }
}

impl<P: StaticPool> Drop for Box<P> {
    fn drop(&mut self) {
        unsafe {
            //trace!("dropping {:u32}", self.ptr as u32);
            self.ptr.drop_in_place();
            P::get().free(self.ptr);
        };
    }
}

unsafe impl<P: StaticPool> Send for Box<P> where P::Item: Send {}

unsafe impl<P: StaticPool> Sync for Box<P> where P::Item: Sync {}

unsafe impl<P: StaticPool> stable_deref_trait::StableDeref for Box<P> {}

impl<P: StaticPool> AsSlice for Box<P>
where
    P::Item: AsSlice,
{
    type Element = <P::Item as AsSlice>::Element;

    fn as_slice(&self) -> &[Self::Element] {
        self.deref().as_slice()
    }
}

impl<P: StaticPool> AsMutSlice for Box<P>
where
    P::Item: AsMutSlice,
{
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        self.deref_mut().as_mut_slice()
    }
}

impl<P: StaticPool> Deref for Box<P> {
    type Target = P::Item;

    fn deref(&self) -> &P::Item {
        unsafe { &*self.ptr }
    }
}

impl<P: StaticPool> DerefMut for Box<P> {
    fn deref_mut(&mut self) -> &mut P::Item {
        unsafe { &mut *self.ptr }
    }
}

impl<P: StaticPool> fmt::Debug for Box<P>
where
    P::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <P::Item as fmt::Debug>::fmt(self, f)
    }
}

impl<P: StaticPool> fmt::Display for Box<P>
where
    P::Item: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <P::Item as fmt::Display>::fmt(self, f)
    }
}

impl<P: StaticPool> PartialEq for Box<P>
where
    P::Item: PartialEq,
{
    fn eq(&self, rhs: &Box<P>) -> bool {
        <P::Item as PartialEq>::eq(self, rhs)
    }
}

impl<P: StaticPool> Eq for Box<P> where P::Item: Eq {}

impl<P: StaticPool> PartialOrd for Box<P>
where
    P::Item: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Box<P>) -> Option<cmp::Ordering> {
        <P::Item as PartialOrd>::partial_cmp(self, rhs)
    }
}

impl<P: StaticPool> Ord for Box<P>
where
    P::Item: Ord,
{
    fn cmp(&self, rhs: &Box<P>) -> cmp::Ordering {
        <P::Item as Ord>::cmp(self, rhs)
    }
}

impl<P: StaticPool> Hash for Box<P>
where
    P::Item: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        <P::Item as Hash>::hash(self, state)
    }
}

macro_rules! pool {
    ($vis:vis $name:ident: [$ty:ty; $size:expr]) => {
        $vis struct $name;
        impl StaticPool for $name {
            type Item = $ty;
            type Pool = BitPool<$ty, $size>;
            fn get() -> &'static Self::Pool {
                static POOL: BitPool<$ty, $size> = BitPool::new();
                &POOL
            }
        }
    };
}
