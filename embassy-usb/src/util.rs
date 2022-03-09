use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Debug, Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub fn select<A, B>(a: A, b: B) -> Select<A, B>
where
    A: Future,
    B: Future,
{
    Select { a, b }
}

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
        match a.poll(cx) {
            Poll::Ready(x) => Poll::Ready(Either::Left(x)),
            Poll::Pending => match b.poll(cx) {
                Poll::Ready(x) => Poll::Ready(Either::Right(x)),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
