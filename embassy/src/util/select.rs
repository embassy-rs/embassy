use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Debug, Clone)]
pub enum Either<A, B> {
    First(A),
    Second(B),
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

#[derive(Debug, Clone)]
pub enum Either3<A, B, C> {
    First(A),
    Second(B),
    Third(C),
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

#[derive(Debug, Clone)]
pub enum Either4<A, B, C, D> {
    First(A),
    Second(B),
    Third(C),
    Fourth(D),
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

/// Future for the [`select_all`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectAll<Fut, const N: usize> {
    inner: [Fut; N],
}

/// Creates a new future which will select over a list of futures.
///
/// The returned future will wait for any future within `iter` to be ready. Upon
/// completion the item resolved will be returned, along with the index of the
/// future that was ready.
///
/// # Panics
///
/// This function will panic if the array specified contains no items.
pub fn select_all<Fut: Future, const N: usize>(arr: [Fut; N]) -> SelectAll<Fut, N> {
    assert!(N > 0);
    SelectAll { inner: arr }
}

impl<Fut: Future, const N: usize> Future for SelectAll<Fut, N> {
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
