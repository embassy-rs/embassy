use crate::pac;

pub struct Spinlock<const N: usize>(core::marker::PhantomData<()>)
where
    Spinlock<N>: SpinlockValid;

impl<const N: usize> Spinlock<N>
where
    Spinlock<N>: SpinlockValid,
{
    /// Try to claim the spinlock. Will return `Some(Self)` if the lock is obtained, and `None` if the lock is
    /// already in use somewhere else.
    pub fn try_claim() -> Option<Self> {
        let lock = pac::SIO.spinlock(N).read();
        if lock > 0 {
            Some(Self(core::marker::PhantomData))
        } else {
            None
        }
    }

    /// Clear a locked spin-lock.
    ///
    /// # Safety
    ///
    /// Only call this function if you hold the spin-lock.
    pub unsafe fn release() {
        // Write (any value): release the lock
        pac::SIO.spinlock(N).write_value(1);
    }
}

impl<const N: usize> Drop for Spinlock<N>
where
    Spinlock<N>: SpinlockValid,
{
    fn drop(&mut self) {
        // This is safe because we own the object, and hence hold the lock.
        unsafe { Self::release() }
    }
}

pub trait SpinlockValid {}
impl SpinlockValid for Spinlock<0> {}
impl SpinlockValid for Spinlock<1> {}
impl SpinlockValid for Spinlock<2> {}
impl SpinlockValid for Spinlock<3> {}
impl SpinlockValid for Spinlock<4> {}
impl SpinlockValid for Spinlock<5> {}
impl SpinlockValid for Spinlock<6> {}
impl SpinlockValid for Spinlock<7> {}
impl SpinlockValid for Spinlock<8> {}
impl SpinlockValid for Spinlock<9> {}
impl SpinlockValid for Spinlock<10> {}
impl SpinlockValid for Spinlock<11> {}
impl SpinlockValid for Spinlock<12> {}
impl SpinlockValid for Spinlock<13> {}
impl SpinlockValid for Spinlock<14> {}
impl SpinlockValid for Spinlock<15> {}
impl SpinlockValid for Spinlock<16> {}
impl SpinlockValid for Spinlock<17> {}
impl SpinlockValid for Spinlock<18> {}
impl SpinlockValid for Spinlock<19> {}
impl SpinlockValid for Spinlock<20> {}
impl SpinlockValid for Spinlock<21> {}
impl SpinlockValid for Spinlock<22> {}
impl SpinlockValid for Spinlock<23> {}
impl SpinlockValid for Spinlock<24> {}
impl SpinlockValid for Spinlock<25> {}
impl SpinlockValid for Spinlock<26> {}
impl SpinlockValid for Spinlock<27> {}
impl SpinlockValid for Spinlock<28> {}
impl SpinlockValid for Spinlock<29> {}
impl SpinlockValid for Spinlock<30> {}
impl SpinlockValid for Spinlock<31> {}
