//! Wait for the first of several futures to complete.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Result for [`select`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either<A, B> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
}

impl<A, B> Either<A, B> {
    /// Did the first future complete first?
    pub fn is_first(&self) -> bool {
        matches!(self, Either::First(_))
    }

    /// Did the second future complete first?
    pub fn is_second(&self) -> bool {
        matches!(self, Either::Second(_))
    }
}

/// Wait for one of two futures to complete.
///
/// This function returns a new future which polls all the futures.
/// When one of them completes, it will complete with its result value.
///
/// The other future is dropped.
pub fn select<A, B>(a: A, b: B) -> Select<A, B>
where
    A: Future,
    B: Future,
{
    Select { a, b }
}

/// Future for the [`select`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select<A, B> {
    a: A,
    b: B,
}

impl<A: Unpin, B: Unpin> Unpin for Select<A, B> {}

impl<A, B> Future for Select<A, B>
where
    A: Future,
    B: Future,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either::Second(x));
        }
        Poll::Pending
    }
}

// ====================================================================

/// Result for [`select3`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either3<A, B, C> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
    /// Third future finished first.
    Third(C),
}

impl<A, B, C> Either3<A, B, C> {
    /// Did the first future complete first?
    pub fn is_first(&self) -> bool {
        matches!(self, Either3::First(_))
    }

    /// Did the second future complete first?
    pub fn is_second(&self) -> bool {
        matches!(self, Either3::Second(_))
    }

    /// Did the third future complete first?
    pub fn is_third(&self) -> bool {
        matches!(self, Either3::Third(_))
    }
}

/// Same as [`select`], but with more futures.
pub fn select3<A, B, C>(a: A, b: B, c: C) -> Select3<A, B, C>
where
    A: Future,
    B: Future,
    C: Future,
{
    Select3 { a, b, c }
}

/// Future for the [`select3`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select3<A, B, C> {
    a: A,
    b: B,
    c: C,
}

impl<A, B, C> Future for Select3<A, B, C>
where
    A: Future,
    B: Future,
    C: Future,
{
    type Output = Either3<A::Output, B::Output, C::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        let c = unsafe { Pin::new_unchecked(&mut this.c) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either3::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either3::Second(x));
        }
        if let Poll::Ready(x) = c.poll(cx) {
            return Poll::Ready(Either3::Third(x));
        }
        Poll::Pending
    }
}

// ====================================================================

/// Result for [`select4`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either4<A, B, C, D> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
    /// Third future finished first.
    Third(C),
    /// Fourth future finished first.
    Fourth(D),
}

impl<A, B, C, D> Either4<A, B, C, D> {
    /// Did the first future complete first?
    pub fn is_first(&self) -> bool {
        matches!(self, Either4::First(_))
    }

    /// Did the second future complete first?
    pub fn is_second(&self) -> bool {
        matches!(self, Either4::Second(_))
    }

    /// Did the third future complete first?
    pub fn is_third(&self) -> bool {
        matches!(self, Either4::Third(_))
    }

    /// Did the fourth future complete first?
    pub fn is_fourth(&self) -> bool {
        matches!(self, Either4::Fourth(_))
    }
}

/// Same as [`select`], but with more futures.
pub fn select4<A, B, C, D>(a: A, b: B, c: C, d: D) -> Select4<A, B, C, D>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
{
    Select4 { a, b, c, d }
}

/// Future for the [`select4`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select4<A, B, C, D> {
    a: A,
    b: B,
    c: C,
    d: D,
}

impl<A, B, C, D> Future for Select4<A, B, C, D>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
{
    type Output = Either4<A::Output, B::Output, C::Output, D::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        let c = unsafe { Pin::new_unchecked(&mut this.c) };
        let d = unsafe { Pin::new_unchecked(&mut this.d) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either4::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either4::Second(x));
        }
        if let Poll::Ready(x) = c.poll(cx) {
            return Poll::Ready(Either4::Third(x));
        }
        if let Poll::Ready(x) = d.poll(cx) {
            return Poll::Ready(Either4::Fourth(x));
        }
        Poll::Pending
    }
}

// ====================================================================

/// Result for [`select5`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either5<A, B, C, D, E> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
    /// Third future finished first.
    Third(C),
    /// Fourth future finished first.
    Fourth(D),
    /// Fifth future finished first.
    Fifth(E),
}

impl<A, B, C, D, E> Either5<A, B, C, D, E> {
    /// Did the first future complete first?
    pub fn is_first(&self) -> bool {
        matches!(self, Either5::First(_))
    }

    /// Did the second future complete first?
    pub fn is_second(&self) -> bool {
        matches!(self, Either5::Second(_))
    }

    /// Did the third future complete first?
    pub fn is_third(&self) -> bool {
        matches!(self, Either5::Third(_))
    }

    /// Did the fourth future complete first?
    pub fn is_fourth(&self) -> bool {
        matches!(self, Either5::Fourth(_))
    }

    /// Did the fifth future complete first?
    pub fn is_fifth(&self) -> bool {
        matches!(self, Either5::Fifth(_))
    }
}

/// Same as [`select`], but with more futures.
pub fn select5<A, B, C, D, E>(a: A, b: B, c: C, d: D, e: E) -> Select5<A, B, C, D, E>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
{
    Select5 { a, b, c, d, e }
}

/// Future for the [`select5`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select5<A, B, C, D, E> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
}

impl<A, B, C, D, E> Future for Select5<A, B, C, D, E>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
{
    type Output = Either5<A::Output, B::Output, C::Output, D::Output, E::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        let c = unsafe { Pin::new_unchecked(&mut this.c) };
        let d = unsafe { Pin::new_unchecked(&mut this.d) };
        let e = unsafe { Pin::new_unchecked(&mut this.e) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either5::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either5::Second(x));
        }
        if let Poll::Ready(x) = c.poll(cx) {
            return Poll::Ready(Either5::Third(x));
        }
        if let Poll::Ready(x) = d.poll(cx) {
            return Poll::Ready(Either5::Fourth(x));
        }
        if let Poll::Ready(x) = e.poll(cx) {
            return Poll::Ready(Either5::Fifth(x));
        }
        Poll::Pending
    }
}

// ====================================================================

/// Result for [`select6`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either6<A, B, C, D, E, F> {
    /// First future finished first.
    First(A),
    /// Second future finished first.
    Second(B),
    /// Third future finished first.
    Third(C),
    /// Fourth future finished first.
    Fourth(D),
    /// Fifth future finished first.
    Fifth(E),
    /// Sixth future finished first.
    Sixth(F),
}

impl<A, B, C, D, E, F> Either6<A, B, C, D, E, F> {
    /// Did the first future complete first?
    pub fn is_first(&self) -> bool {
        matches!(self, Either6::First(_))
    }

    /// Did the second future complete first?
    pub fn is_second(&self) -> bool {
        matches!(self, Either6::Second(_))
    }

    /// Did the third future complete first?
    pub fn is_third(&self) -> bool {
        matches!(self, Either6::Third(_))
    }

    /// Did the fourth future complete first?
    pub fn is_fourth(&self) -> bool {
        matches!(self, Either6::Fourth(_))
    }

    /// Did the fifth future complete first?
    pub fn is_fifth(&self) -> bool {
        matches!(self, Either6::Fifth(_))
    }

    /// Did the sixth future complete first?
    pub fn is_sixth(&self) -> bool {
        matches!(self, Either6::Sixth(_))
    }
}

/// Same as [`select`], but with more futures.
pub fn select6<A, B, C, D, E, F>(a: A, b: B, c: C, d: D, e: E, f: F) -> Select6<A, B, C, D, E, F>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
    F: Future,
{
    Select6 { a, b, c, d, e, f }
}

/// Future for the [`select6`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select6<A, B, C, D, E, F> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
}

impl<A, B, C, D, E, F> Future for Select6<A, B, C, D, E, F>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
    F: Future,
{
    type Output = Either6<A::Output, B::Output, C::Output, D::Output, E::Output, F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { Pin::new_unchecked(&mut this.a) };
        let b = unsafe { Pin::new_unchecked(&mut this.b) };
        let c = unsafe { Pin::new_unchecked(&mut this.c) };
        let d = unsafe { Pin::new_unchecked(&mut this.d) };
        let e = unsafe { Pin::new_unchecked(&mut this.e) };
        let f = unsafe { Pin::new_unchecked(&mut this.f) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either6::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either6::Second(x));
        }
        if let Poll::Ready(x) = c.poll(cx) {
            return Poll::Ready(Either6::Third(x));
        }
        if let Poll::Ready(x) = d.poll(cx) {
            return Poll::Ready(Either6::Fourth(x));
        }
        if let Poll::Ready(x) = e.poll(cx) {
            return Poll::Ready(Either6::Fifth(x));
        }
        if let Poll::Ready(x) = f.poll(cx) {
            return Poll::Ready(Either6::Sixth(x));
        }
        Poll::Pending
    }
}

// ====================================================================

/// Future for the [`select_array`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectArray<Fut, const N: usize> {
    inner: [Fut; N],
}

/// Creates a new future which will select over an array of futures.
///
/// The returned future will wait for any future to be ready. Upon
/// completion the item resolved will be returned, along with the index of the
/// future that was ready.
///
/// If the array is empty, the resulting future will be Pending forever.
pub fn select_array<Fut: Future, const N: usize>(arr: [Fut; N]) -> SelectArray<Fut, N> {
    SelectArray { inner: arr }
}

impl<Fut: Future, const N: usize> Future for SelectArray<Fut, N> {
    type Output = (Fut::Output, usize);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: Since `self` is pinned, `inner` cannot move. Since `inner` cannot move,
        // its elements also cannot move. Therefore it is safe to access `inner` and pin
        // references to the contained futures.
        let item = unsafe {
            self.get_unchecked_mut()
                .inner
                .iter_mut()
                .enumerate()
                .find_map(|(i, f)| match Pin::new_unchecked(f).poll(cx) {
                    Poll::Pending => None,
                    Poll::Ready(e) => Some((i, e)),
                })
        };

        match item {
            Some((idx, res)) => Poll::Ready((res, idx)),
            None => Poll::Pending,
        }
    }
}

// ====================================================================

/// Future for the [`select_slice`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectSlice<'a, Fut> {
    inner: Pin<&'a mut [Fut]>,
}

/// Creates a new future which will select over a slice of futures.
///
/// The returned future will wait for any future to be ready. Upon
/// completion the item resolved will be returned, along with the index of the
/// future that was ready.
///
/// If the slice is empty, the resulting future will be Pending forever.
pub fn select_slice<'a, Fut: Future>(slice: Pin<&'a mut [Fut]>) -> SelectSlice<'a, Fut> {
    SelectSlice { inner: slice }
}

impl<'a, Fut: Future> Future for SelectSlice<'a, Fut> {
    type Output = (Fut::Output, usize);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: refer to
        //   https://users.rust-lang.org/t/working-with-pinned-slices-are-there-any-structurally-pinning-vec-like-collection-types/50634/2
        #[inline(always)]
        fn pin_iter<T>(slice: Pin<&mut [T]>) -> impl Iterator<Item = Pin<&mut T>> {
            unsafe { slice.get_unchecked_mut().iter_mut().map(|v| Pin::new_unchecked(v)) }
        }
        for (i, fut) in pin_iter(self.inner.as_mut()).enumerate() {
            if let Poll::Ready(res) = fut.poll(cx) {
                return Poll::Ready((res, i));
            }
        }

        Poll::Pending
    }
}
