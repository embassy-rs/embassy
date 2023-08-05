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
                    pub unsafe fn steal() -> Self {
                        Self{ _private: ()}
                    }
                }

                $(#[$cfg])?
                $crate::impl_peripheral!($name);
            )*
        }
    };
}

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
                pub $name: peripherals::$name,
            )*
        }

        impl Peripherals {
            ///Returns all the peripherals *once*
            #[inline]
            pub(crate) fn take() -> Self {

                #[no_mangle]
                static mut _EMBASSY_DEVICE_PERIPHERALS: bool = false;

                critical_section::with(|_| unsafe {
                    if _EMBASSY_DEVICE_PERIPHERALS {
                        panic!("init called more than once!")
                    }
                    _EMBASSY_DEVICE_PERIPHERALS = true;
                    Self::steal()
                })
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

#[macro_export]
macro_rules! into_ref {
    ($($name:ident),*) => {
        $(
            let mut $name = $name.into_ref();
        )*
    }
}

#[macro_export]
macro_rules! impl_peripheral {
    ($type:ident) => {
        impl $crate::Peripheral for $type {
            type P = $type;

            #[inline]
            unsafe fn clone_unchecked(&self) -> Self::P {
                $type { ..*self }
            }
        }
    };
}
