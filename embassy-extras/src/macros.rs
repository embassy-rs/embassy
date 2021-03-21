#[macro_export]
macro_rules! peripherals {
    ($($(#[$cfg:meta])? $name:ident: $type:ident),*$(,)?) => {
        $(
            $(#[$cfg])?
            #[allow(non_camel_case_types)]
            pub struct $type { _private: () }

            $(#[$cfg])?
            impl embassy::util::PeripheralBorrow for $type {
                type Target = $type;
                unsafe fn unborrow(self) -> $type {
                    self
                }
            }

            $(#[$cfg])?
            impl embassy::util::PeripheralBorrow for &mut $type {
                type Target = $type;
                unsafe fn unborrow(self) -> $type {
                    ::core::ptr::read(self)
                }
            }
        )*

        pub struct Peripherals {
            $(
                $(#[$cfg])?
                pub $name: $type,
            )*
        }

        impl Peripherals {
            pub unsafe fn steal() -> Self {
                Self {
                    $(
                        $(#[$cfg])?
                        $name: $type { _private: () },
                    )*
                }
            }
        }

    };
}
