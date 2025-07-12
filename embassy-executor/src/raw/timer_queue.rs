//! Timer queue operations.

use core::cell::Cell;

use super::TaskRef;

#[cfg(feature = "_timer-item-payload")]
macro_rules! define_opaque {
    ($size:tt) => {
        /// An opaque data type.
        #[repr(align($size))]
        pub struct OpaqueData {
            data: [u8; $size],
        }

        impl OpaqueData {
            const fn new() -> Self {
                Self { data: [0; $size] }
            }

            /// Access the data as a reference to a type `T`.
            ///
            /// Safety:
            ///
            /// The caller must ensure that the size of the type `T` is less than, or equal to
            /// the size of the payload, and must ensure that the alignment of the type `T` is
            /// less than, or equal to the alignment of the payload.
            ///
            /// The type must be valid when zero-initialized.
            pub unsafe fn as_ref<T>(&self) -> &T {
                &*(self.data.as_ptr() as *const T)
            }
        }
    };
}

#[cfg(feature = "timer-item-payload-size-1")]
define_opaque!(1);
#[cfg(feature = "timer-item-payload-size-2")]
define_opaque!(2);
#[cfg(feature = "timer-item-payload-size-4")]
define_opaque!(4);
#[cfg(feature = "timer-item-payload-size-8")]
define_opaque!(8);

/// An item in the timer queue.
pub struct TimerQueueItem {
    /// The next item in the queue.
    ///
    /// If this field contains `Some`, the item is in the queue. The last item in the queue has a
    /// value of `Some(dangling_pointer)`
    pub next: Cell<Option<TaskRef>>,

    /// The time at which this item expires.
    pub expires_at: Cell<u64>,

    /// Some implementation-defined, zero-initialized piece of data.
    #[cfg(feature = "_timer-item-payload")]
    pub payload: OpaqueData,
}

unsafe impl Sync for TimerQueueItem {}

impl TimerQueueItem {
    pub(crate) const fn new() -> Self {
        Self {
            next: Cell::new(None),
            expires_at: Cell::new(0),
            #[cfg(feature = "_timer-item-payload")]
            payload: OpaqueData::new(),
        }
    }
}
