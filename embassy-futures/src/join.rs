//! Wait for multiple futures to complete.

use core::future::Future;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::{fmt, mem};

#[derive(Debug)]
enum MaybeDone<Fut: Future> {
    /// A not-yet-completed future
    Future(/* #[pin] */ Fut),
    /// The output of the completed future
    Done(Fut::Output),
    /// The empty variant after the result of a [`MaybeDone`] has been
    /// taken using the [`take_output`](MaybeDone::take_output) method.
    Gone,
}

impl<Fut: Future> MaybeDone<Fut> {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        match this {
            Self::Future(fut) => match unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                Poll::Ready(res) => {
                    *this = Self::Done(res);
                    true
                }
                Poll::Pending => false,
            },
            _ => true,
        }
    }

    fn take_output(&mut self) -> Fut::Output {
        match &*self {
            Self::Done(_) => {}
            Self::Future(_) | Self::Gone => panic!("take_output when MaybeDone is not done."),
        }
        match mem::replace(self, Self::Gone) {
            MaybeDone::Done(output) => output,
            _ => unreachable!(),
        }
    }
}

impl<Fut: Future + Unpin> Unpin for MaybeDone<Fut> {}

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, <$($Fut:ident),*>),
    )*) => ($(
        $(#[$doc])*
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        #[allow(non_snake_case)]
        pub struct $Join<$($Fut: Future),*> {
            $(
                $Fut: MaybeDone<$Fut>,
            )*
        }

        impl<$($Fut),*> fmt::Debug for $Join<$($Fut),*>
        where
            $(
                $Fut: Future + fmt::Debug,
                $Fut::Output: fmt::Debug,
            )*
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($Join))
                    $(.field(stringify!($Fut), &self.$Fut))*
                    .finish()
            }
        }

        impl<$($Fut: Future),*> $Join<$($Fut),*> {
            #[allow(non_snake_case)]
            fn new($($Fut: $Fut),*) -> Self {
                Self {
                    $($Fut: MaybeDone::Future($Fut)),*
                }
            }
        }

        impl<$($Fut: Future),*> Future for $Join<$($Fut),*> {
            type Output = ($($Fut::Output),*);

            fn poll(
                self: Pin<&mut Self>, cx: &mut Context<'_>
            ) -> Poll<Self::Output> {
                let this = unsafe { self.get_unchecked_mut() };
                let mut all_done = true;
                $(
                    all_done &= unsafe { Pin::new_unchecked(&mut this.$Fut) }.poll(cx);
                )*

                if all_done {
                    Poll::Ready(($(this.$Fut.take_output()), *))
                } else {
                    Poll::Pending
                }
            }
        }
    )*)
}

generate! {
    /// Future for the [`join`](join()) function.
    (Join, <Fut1, Fut2>),

    /// Future for the [`join3`] function.
    (Join3, <Fut1, Fut2, Fut3>),

    /// Future for the [`join4`] function.
    (Join4, <Fut1, Fut2, Fut3, Fut4>),

    /// Future for the [`join5`] function.
    (Join5, <Fut1, Fut2, Fut3, Fut4, Fut5>),
}

/// Joins the result of two futures, waiting for them both to complete.
///
/// This function will return a new future which awaits both futures to
/// complete. The returned future will finish with a tuple of both results.
///
/// Note that this function consumes the passed futures and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # embassy_futures::block_on(async {
///
/// let a = async { 1 };
/// let b = async { 2 };
/// let pair = embassy_futures::join::join(a, b).await;
///
/// assert_eq!(pair, (1, 2));
/// # });
/// ```
pub fn join<Fut1, Fut2>(future1: Fut1, future2: Fut2) -> Join<Fut1, Fut2>
where
    Fut1: Future,
    Fut2: Future,
{
    Join::new(future1, future2)
}

/// Joins the result of three futures, waiting for them all to complete.
///
/// This function will return a new future which awaits all futures to
/// complete. The returned future will finish with a tuple of all results.
///
/// Note that this function consumes the passed futures and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # embassy_futures::block_on(async {
///
/// let a = async { 1 };
/// let b = async { 2 };
/// let c = async { 3 };
/// let res = embassy_futures::join::join3(a, b, c).await;
///
/// assert_eq!(res, (1, 2, 3));
/// # });
/// ```
pub fn join3<Fut1, Fut2, Fut3>(future1: Fut1, future2: Fut2, future3: Fut3) -> Join3<Fut1, Fut2, Fut3>
where
    Fut1: Future,
    Fut2: Future,
    Fut3: Future,
{
    Join3::new(future1, future2, future3)
}

/// Joins the result of four futures, waiting for them all to complete.
///
/// This function will return a new future which awaits all futures to
/// complete. The returned future will finish with a tuple of all results.
///
/// Note that this function consumes the passed futures and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # embassy_futures::block_on(async {
///
/// let a = async { 1 };
/// let b = async { 2 };
/// let c = async { 3 };
/// let d = async { 4 };
/// let res = embassy_futures::join::join4(a, b, c, d).await;
///
/// assert_eq!(res, (1, 2, 3, 4));
/// # });
/// ```
pub fn join4<Fut1, Fut2, Fut3, Fut4>(
    future1: Fut1,
    future2: Fut2,
    future3: Fut3,
    future4: Fut4,
) -> Join4<Fut1, Fut2, Fut3, Fut4>
where
    Fut1: Future,
    Fut2: Future,
    Fut3: Future,
    Fut4: Future,
{
    Join4::new(future1, future2, future3, future4)
}

/// Joins the result of five futures, waiting for them all to complete.
///
/// This function will return a new future which awaits all futures to
/// complete. The returned future will finish with a tuple of all results.
///
/// Note that this function consumes the passed futures and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # embassy_futures::block_on(async {
///
/// let a = async { 1 };
/// let b = async { 2 };
/// let c = async { 3 };
/// let d = async { 4 };
/// let e = async { 5 };
/// let res = embassy_futures::join::join5(a, b, c, d, e).await;
///
/// assert_eq!(res, (1, 2, 3, 4, 5));
/// # });
/// ```
pub fn join5<Fut1, Fut2, Fut3, Fut4, Fut5>(
    future1: Fut1,
    future2: Fut2,
    future3: Fut3,
    future4: Fut4,
    future5: Fut5,
) -> Join5<Fut1, Fut2, Fut3, Fut4, Fut5>
where
    Fut1: Future,
    Fut2: Future,
    Fut3: Future,
    Fut4: Future,
    Fut5: Future,
{
    Join5::new(future1, future2, future3, future4, future5)
}

// =====================================================

/// Future for the [`join_array`] function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct JoinArray<Fut: Future, const N: usize> {
    futures: [MaybeDone<Fut>; N],
}

impl<Fut: Future, const N: usize> fmt::Debug for JoinArray<Fut, N>
where
    Fut: Future + fmt::Debug,
    Fut::Output: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JoinArray").field("futures", &self.futures).finish()
    }
}

impl<Fut: Future, const N: usize> Future for JoinArray<Fut, N> {
    type Output = [Fut::Output; N];
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut all_done = true;
        for f in this.futures.iter_mut() {
            all_done &= unsafe { Pin::new_unchecked(f) }.poll(cx);
        }

        if all_done {
            let mut array: [MaybeUninit<Fut::Output>; N] = unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..N {
                array[i].write(this.futures[i].take_output());
            }
            Poll::Ready(unsafe { (&array as *const _ as *const [Fut::Output; N]).read() })
        } else {
            Poll::Pending
        }
    }
}

/// Joins the result of an array of futures, waiting for them all to complete.
///
/// This function will return a new future which awaits all futures to
/// complete. The returned future will finish with a tuple of all results.
///
/// Note that this function consumes the passed futures and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # embassy_futures::block_on(async {
///
/// async fn foo(n: u32) -> u32 { n }
/// let a = foo(1);
/// let b = foo(2);
/// let c = foo(3);
/// let res = embassy_futures::join::join_array([a, b, c]).await;
///
/// assert_eq!(res, [1, 2, 3]);
/// # });
/// ```
pub fn join_array<Fut: Future, const N: usize>(futures: [Fut; N]) -> JoinArray<Fut, N> {
    JoinArray {
        futures: futures.map(MaybeDone::Future),
    }
}
