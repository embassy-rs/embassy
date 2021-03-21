#[macro_export]
macro_rules! peripherals {
    ($($(#[$cfg:meta])? $name:ident: $type:ident),*$(,)?) => {
        pub mod peripherals {
            $(
                $(#[$cfg])?
                #[allow(non_camel_case_types)]
                pub struct $type { _private: () }

                impl embassy::util::Steal for $type {
                    #[inline]
                    unsafe fn steal() -> Self {
                        Self{ _private: ()}
                    }
                }

                $(#[$cfg])?
                impl embassy::util::PeripheralBorrow for $type {
                    type Target = $type;
                    #[inline]
                    unsafe fn unborrow(self) -> $type {
                        self
                    }
                }

                $(#[$cfg])?
                impl embassy::util::PeripheralBorrow for &mut $type {
                    type Target = $type;
                    #[inline]
                    unsafe fn unborrow(self) -> $type {
                        ::core::ptr::read(self)
                    }
                }
            )*
        }

        pub struct Peripherals {
            $(
                $(#[$cfg])?
                pub $name: peripherals::$type,
            )*
        }

        impl Peripherals {
            ///Returns all the peripherals *once*
            #[inline]
            pub fn take() -> Option<Self> {

                #[no_mangle]
                static mut _EMBASSY_DEVICE_PERIPHERALS: bool = false;

                cortex_m::interrupt::free(|_| {
                    if unsafe { _EMBASSY_DEVICE_PERIPHERALS } {
                        None
                    } else {
                        Some(unsafe { <Self as embassy::util::Steal>::steal() })
                    }
                })
            }
        }

        impl embassy::util::Steal for Peripherals {
            #[inline]
            unsafe fn steal() -> Self {
                Self {
                    $(
                        $(#[$cfg])?
                        $name: <peripherals::$type as embassy::util::Steal>::steal(),
                    )*
                }
            }
        }

    };
}
