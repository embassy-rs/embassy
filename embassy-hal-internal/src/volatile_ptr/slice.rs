use core::ops::RangeBounds;
use core::ptr::{self, NonNull};
use core::slice::SliceIndex;

use crate::volatile_ptr::VolatilePtr;
use crate::volatile_ptr::access::{Access, Readable, Writable};

impl<'a, T, A> VolatilePtr<'a, [T], A> {
    /// Returns the length of the slice.
    pub fn len(self) -> usize {
        self.pointer.len()
    }

    /// Returns whether the slice is empty.
    pub fn is_empty(self) -> bool {
        self.pointer.len() == 0
    }

    /// Applies the index operation on the wrapped slice.
    ///
    /// Returns a shared `Volatile` reference to the resulting subslice.
    ///
    /// This is a convenience method for the `map(|slice| slice.index(index))` operation, so it
    /// has the same behavior as the indexing operation on slice (e.g. panic if index is
    /// out-of-bounds).
    ///
    /// ## Examples
    ///
    /// Accessing a single slice element:
    ///
    /// ```
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = unsafe { VolatilePtr::new_read_only(NonNull::from(slice)) };
    /// assert_eq!(volatile.index(1).read(), 2);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = unsafe { VolatilePtr::new_read_only(NonNull::from(slice)) };
    /// let subslice = volatile.index(1..);
    /// assert_eq!(subslice.index(0).read(), 2);
    /// ```
    pub fn index(self, index: usize) -> VolatilePtr<'a, <usize as SliceIndex<[T]>>::Output, A>
    where
        A: Access,
    {
        bounds_check(self.pointer.len(), index.clone());

        unsafe { self.map(|slice| NonNull::new_unchecked((slice.as_ptr() as *mut T).add(index))) }
    }

    /// Returns an iterator over the slice.
    pub fn iter(self) -> impl Iterator<Item = VolatilePtr<'a, T, A>>
    where
        A: Access,
    {
        let ptr = self.as_raw_ptr().as_ptr() as *mut T;
        let len = self.len();
        (0..len).map(move |i| unsafe { VolatilePtr::new_generic(NonNull::new_unchecked(ptr.add(i))) })
    }

    /// Copies all elements from `self` into `dst`, using a volatile memcpy.
    ///
    /// The length of `dst` must be the same as `self`.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a volatile slice:
    ///
    /// ```
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// // the `Volatile` type does not work with arrays, so convert `src` to a slice
    /// let slice = &src[..];
    /// let volatile = unsafe { VolatilePtr::new_read_only(NonNull::from(slice)) };
    /// let mut dst = [5, 0, 0];
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the destination slice from three elements
    /// // to two. It will panic if we don't do this.
    /// volatile.copy_into_slice(&mut dst[1..]);
    ///
    /// assert_eq!(src, [1, 2]);
    /// assert_eq!(dst, [5, 1, 2]);
    /// ```
    pub fn copy_into_slice(self, dst: &mut [T])
    where
        T: Copy,
        A: Readable,
    {
        let len = self.pointer.len();
        assert_eq!(len, dst.len(), "destination and source slices have different lengths");

        // intrinsics::volatile_copy_nonoverlapping_memory(dst.as_mut_ptr(), self.pointer.as_mut_ptr(), len);

        todo!()
    }

    /// Copies all elements from `src` into `self`, using a volatile memcpy.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// This method is similar to the `slice::copy_from_slice` method of the standard library. The
    /// difference is that this method performs a volatile copy.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a slice into a volatile slice:
    ///
    /// ```
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// // the `Volatile` type does not work with arrays, so convert `dst` to a slice
    /// let slice = &mut dst[..];
    /// let volatile = unsafe { VolatilePtr::new(NonNull::from(slice)) };
    /// // Because the slices have to be the same length,
    /// // we slice the source slice from four elements
    /// // to two. It will panic if we don't do this.
    /// volatile.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(src, [1, 2, 3, 4]);
    /// assert_eq!(dst, [3, 4]);
    /// ```
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
        A: Writable,
    {
        let len = self.pointer.len();
        assert_eq!(len, src.len(), "destination and source slices have different lengths");

        unsafe {
            let src = core::slice::from_raw_parts(src as *const _ as *const u8, size_of_val(src));
            let dst = self.pointer.as_mut() as *mut _ as *mut u8;

            type NativeSizeArray = [u8; 8];
            let (prefix, middle, suffix) = src.align_to::<NativeSizeArray>();
            let mut i = 0;
            for &v in prefix.iter() {
                dst.add(i).write_volatile(v);
                i += 1;
            }
            for &v in middle.iter() {
                dst.add(i).cast::<NativeSizeArray>().write_volatile(v);
                i += core::mem::size_of::<NativeSizeArray>();
            }
            for &v in suffix.iter() {
                dst.add(i).write_volatile(v);
                i += 1;
            }
        }
    }

    /// Copies elements from one part of the slice to another part of itself, using a
    /// volatile `memmove`.
    ///
    /// `src` is the range within `self` to copy from. `dest` is the starting index of the
    /// range within `self` to copy to, which will have the same length as `src`. The two ranges
    /// may overlap. The ends of the two ranges must be less than or equal to `self.len()`.
    ///
    /// This method is similar to the `slice::copy_within` method of the standard library. The
    /// difference is that this method performs a volatile copy.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if either range exceeds the end of the slice, or if the end
    /// of `src` is before the start.
    ///
    /// ## Examples
    ///
    /// Copying four bytes within a slice:
    ///
    /// ```
    /// extern crate core;
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut byte_array = *b"Hello, World!";
    /// let slice: &mut [u8] = &mut byte_array[..];
    /// let volatile = unsafe { VolatilePtr::new(NonNull::from(slice)) };
    /// volatile.copy_within(1..5, 8);
    ///
    /// assert_eq!(&byte_array, b"Hello, Wello!");
    pub fn copy_within(self, _src: impl RangeBounds<usize>, _dest: usize)
    where
        T: Copy,
        A: Readable + Writable,
    {
        // let len = self.pointer.len();
        // implementation taken from https://github.com/rust-lang/rust/blob/683d1bcd405727fcc9209f64845bd3b9104878b8/library/core/src/slice/mod.rs#L2726-L2738
        //        let Range {
        //            start: src_start,
        //            end: src_end,
        //        } = range(src, ..len);
        //        let count = src_end - src_start;
        //        assert!(dest <= len - count, "dest is out of bounds");
        //        // SAFETY: the conditions for `volatile_copy_memory` have all been checked above,
        //        // as have those for `ptr::add`.
        //        unsafe {
        //            intrinsics::volatile_copy_memory(
        //                self.pointer.as_mut_ptr().add(dest),
        //                self.pointer.as_mut_ptr().add(src_start),
        //                count,
        //            );
        //        }

        todo!()
    }

    /// Divides one slice into two at an index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding
    /// the index `mid` itself) and the second will contain all
    /// indices from `[mid, len)` (excluding the index `len` itself).
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    ///
    pub fn split_at(self, mid: usize) -> (VolatilePtr<'a, [T], A>, VolatilePtr<'a, [T], A>)
    where
        A: Access,
    {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_unchecked(mid) }
    }

    unsafe fn split_at_unchecked(self, _mid: usize) -> (VolatilePtr<'a, [T], A>, VolatilePtr<'a, [T], A>)
    where
        A: Access,
    {
        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        // unsafe {
        //     (
        //         VolatilePtr::new_generic((self.pointer).get_unchecked_mut(..mid)),
        //         VolatilePtr::new_generic((self.pointer).get_unchecked_mut(mid..)),
        //     )
        // }

        todo!()
    }

    /// Splits the slice into a slice of `N`-element arrays,
    /// starting at the beginning of the slice,
    /// and a remainder slice with length strictly less than `N`.
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0.
    #[allow(clippy::type_complexity)]
    pub fn as_chunks<const N: usize>(self) -> (VolatilePtr<'a, [[T; N]], A>, VolatilePtr<'a, [T], A>)
    where
        A: Access,
    {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked() };
        (array_slice, remainder)
    }

    /// Splits the slice into a slice of `N`-element arrays,
    /// assuming that there's no remainder.
    ///
    /// # Safety
    ///
    /// This may only be called when
    /// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
    /// - `N != 0`.
    pub unsafe fn as_chunks_unchecked<const N: usize>(self) -> VolatilePtr<'a, [[T; N]], A>
    where
        A: Access,
    {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        // let new_len =
        // SAFETY: Our precondition is exactly what's needed to call this
        // unsafe { core::intrinsics::exact_div(self.pointer.len(), N) };

        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        // let pointer = NonNull::new(ptr::slice_from_raw_parts_mut(self.pointer.as_mut_ptr().cast(), new_len)).unwrap();
        // unsafe { VolatilePtr::new_generic(pointer) }

        todo!()
    }
}

/// Methods for volatile byte slices
impl<A> VolatilePtr<'_, [u8], A> {
    /// Sets all elements of the byte slice to the given `value` using a volatile `memset`.
    ///
    /// This method is similar to the `slice::fill` method of the standard library, with the
    /// difference that this method performs a volatile write operation. Another difference
    /// is that this method is only available for byte slices (not general `&mut [T]` slices)
    /// because there currently isn't a instrinsic function that allows non-`u8` values.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut vec = vec![0; 10];
    /// let buf = unsafe { VolatilePtr::new(NonNull::from(vec.as_mut_slice())) };
    /// buf.fill(1);
    /// assert_eq!(unsafe { buf.as_raw_ptr().as_mut() }, &mut vec![1; 10]);
    /// ```
    pub fn fill(self, _value: u8)
    where
        A: Writable,
    {
        //    intrinsics::volatile_set_memory(self.pointer.as_mut_ptr(), value, self.pointer.len());

        todo!()
    }
}

/// Methods for converting arrays to slices
///
/// These methods are only available with the `unstable` feature enabled (requires a nightly
/// Rust compiler).
impl<'a, T, A, const N: usize> VolatilePtr<'a, [T; N], A> {
    /// Converts an array pointer to a slice pointer.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements from a volatile array reference using `copy_into_slice`:
    ///
    /// ```
    /// use volatile::VolatilePtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// let volatile = unsafe { VolatilePtr::new_read_only(NonNull::from(&src)) };
    /// let mut dst = [0, 0];
    ///
    /// // convert the `Volatile<&[i32; 2]>` array reference to a `Volatile<&[i32]>` slice
    /// let volatile_slice = volatile.as_slice();
    /// // we can now use the slice methods
    /// volatile_slice.copy_into_slice(&mut dst);
    ///
    /// assert_eq!(dst, [1, 2]);
    /// ```
    pub fn as_slice(self) -> VolatilePtr<'a, [T], A>
    where
        A: Access,
    {
        unsafe { self.map(|array| NonNull::new(ptr::slice_from_raw_parts_mut(array.as_ptr() as *mut T, N)).unwrap()) }
    }
}

fn bounds_check(len: usize, index: impl SliceIndex<[()]>) {
    const MAX_ARRAY: [(); usize::MAX] = [(); usize::MAX];

    let bound_check_slice = &MAX_ARRAY[..len];
    let _ = &bound_check_slice[index];
}
