use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Future for the [`select_all`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectAll<Fut, const N: usize> {
    inner: [Fut; N],
}

impl<Fut: Unpin, const N: usize> Unpin for SelectAll<Fut, N> {}

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

impl<Fut, const N: usize> SelectAll<Fut, N> {
    /// Consumes this combinator, returning the underlying futures.
    pub fn into_inner(self) -> [Fut; N] {
        self.inner
    }
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
