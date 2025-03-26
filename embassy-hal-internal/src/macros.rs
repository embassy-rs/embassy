/// Types for the peripheral singletons.
#[macro_export]
macro_rules! peripherals_definition {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        /// Types for the peripheral singletons.
        pub mod peripherals {
            $(
                $(#[$cfg])?
                #[allow(non_camel_case_types)]
                #[doc = concat!(stringify!($name), " peripheral")]
                pub struct $name { _private: () }

                $(#[$cfg])?
                impl $name {
                    /// Unsafely create an instance of this peripheral out of thin air.
                    ///
                    /// # Safety
                    ///
                    /// You must ensure that you're only using one instance of this type at a time.
                    #[inline]
                    pub unsafe fn steal() -> $crate::Peri<'static, Self> {
                        $crate::Peri::new_unchecked(Self{ _private: ()})
                    }
                }

                $(#[$cfg])?
                $crate::impl_peripheral!($name);
            )*
        }
    };
}

/// Define the peripherals struct.
#[macro_export]
macro_rules! peripherals_struct {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        /// Struct containing all the peripheral singletons.
        ///
        /// To obtain the peripherals, you must initialize the HAL, by calling [`crate::init`].
        #[allow(non_snake_case)]
        pub struct Peripherals {
            $(
                #[doc = concat!(stringify!($name), " peripheral")]
                $(#[$cfg])?
                pub $name: $crate::Peri<'static, peripherals::$name>,
            )*
        }

        impl Peripherals {
            ///Returns all the peripherals *once*
            #[inline]
            pub(crate) fn take() -> Self {
                critical_section::with(Self::take_with_cs)
            }

            ///Returns all the peripherals *once*
            #[inline]
            pub(crate) fn take_with_cs(_cs: critical_section::CriticalSection) -> Self {
                #[no_mangle]
                static mut _EMBASSY_DEVICE_PERIPHERALS: bool = false;

                // safety: OK because we're inside a CS.
                unsafe {
                    if _EMBASSY_DEVICE_PERIPHERALS {
                        panic!("init called more than once!")
                    }
                    _EMBASSY_DEVICE_PERIPHERALS = true;
                    Self::steal()
                }
            }
        }

        impl Peripherals {
            /// Unsafely create an instance of this peripheral out of thin air.
            ///
            /// # Safety
            ///
            /// You must ensure that you're only using one instance of this type at a time.
            #[inline]
            pub unsafe fn steal() -> Self {
                Self {
                    $(
                        $(#[$cfg])?
                        $name: peripherals::$name::steal(),
                    )*
                }
            }
        }
    };
}

/// Defining peripheral type.
#[macro_export]
macro_rules! peripherals {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        $crate::peripherals_definition!(
            $(
                $(#[$cfg])?
                $name,
            )*
        );
        $crate::peripherals_struct!(
            $(
                $(#[$cfg])?
                $name,
            )*
        );
    };
}

/// Implement the peripheral trait.
#[macro_export]
macro_rules! impl_peripheral {
    ($type:ident<$($T:ident $(: $bound:tt $(+ $others:tt )*)?),*>) => {
        impl<$($T: $($bound $(+$others)*)?),*> Copy for $type <$($T),*> {}
        impl<$($T: $($bound $(+$others)*)?),*> Clone for $type <$($T),*> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<$($T: $($bound $(+$others)*)?),*> PeripheralType for $type <$($T),*> {}
    };

    ($type:ident) => {
        impl Copy for $type {}
        impl Clone for $type {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl $crate::PeripheralType for $type {}
    };
}
