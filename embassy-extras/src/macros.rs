#[macro_export]
macro_rules! peripherals {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        pub mod peripherals {
            $(
                $(#[$cfg])?
                #[allow(non_camel_case_types)]
                pub struct $name { _private: () }

                $(#[$cfg])?
                impl embassy::util::Steal for $name {
                    #[inline]
                    unsafe fn steal() -> Self {
                        Self{ _private: ()}
                    }
                }

                $(#[$cfg])?
                impl embassy::util::PeripheralBorrow for $name {
                    type Target = $name;
                    #[inline]
                    unsafe fn unborrow(self) -> $name {
                        self
                    }
                }

                $(#[$cfg])?
                impl embassy::util::PeripheralBorrow for &mut $name {
                    type Target = $name;
                    #[inline]
                    unsafe fn unborrow(self) -> $name {
                        ::core::ptr::read(self)
                    }
                }
            )*
        }

        #[allow(non_snake_case)]
        pub struct Peripherals {
            $(
                $(#[$cfg])?
                pub $name: peripherals::$name,
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
                        $name: <peripherals::$name as embassy::util::Steal>::steal(),
                    )*
                }
            }
        }

    };
}

#[macro_export]
macro_rules! unborrow {
    ($($name:ident),*) => {
        $(
            let mut $name = unsafe { $name.unborrow() };
        )*
    }
}

#[macro_export]
macro_rules! impl_unborrow {
    ($type:ident) => {
        impl ::embassy::util::PeripheralBorrow for $type {
            type Target = $type;
            #[inline]
            unsafe fn unborrow(self) -> Self::Target {
                self
            }
        }

        impl<'a> ::embassy::util::PeripheralBorrow for &'a mut $type {
            type Target = $type;
            #[inline]
            unsafe fn unborrow(self) -> Self::Target {
                unsafe { ::core::ptr::read(self) }
            }
        }
    };
}

#[macro_export]
macro_rules! std_peripherals {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        #[doc = r"All the peripherals"]
        #[allow(non_snake_case)]
        pub struct Peripherals {
            $(
                $(#[$cfg])?
                pub $name: pac::$name,
            )+
        }

        static mut GLOBAL_CLOCKS: Option<Clocks> = None;

        impl Peripherals {
            pub fn take() -> Option<(Peripherals, Clocks)> {
                match unsafe {GLOBAL_CLOCKS.take()} {
                    Some(clocks) => {
                        let dp = unsafe { pac::Peripherals::steal() };
                        let peripherals = Peripherals {
                            $(
                                $(#[$cfg])?
                                $name: dp.$name,
                            )+
                        };

                        Some((peripherals, clocks))
                    },
                    None => None,
                }
            }

            pub unsafe fn set_peripherals(clocks: Clocks) {
                GLOBAL_CLOCKS.replace(clocks);
            }
        }
    };
}
