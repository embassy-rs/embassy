use core::marker::PhantomData;
use core::ops::Deref;

/// Borrow of a peripheral instance.
///
/// This is equivalent to `&mut T`, except smaller. For example, if
/// T is zero-sized, Borrowed<T> is too, while `&mut T` would be 32 bits.
pub struct Borrow<'a, T> {
    inner: T,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Borrow<'a, T> {
    pub fn from_owned(inner: T) -> Self {
        Borrow {
            inner,
            phantom: PhantomData,
        }
    }
}
impl<'a, T> Deref for Borrow<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait Borrowable: Sized {
    fn borrow(&mut self) -> Borrow<'_, Self>;
}
