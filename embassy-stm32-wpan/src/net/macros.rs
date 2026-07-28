/// Define a newtype over a primitive integer with a set of named values.
///
/// These types appear in the `#[repr(C)]` structs that are reinterpreted directly from the buffers
/// shared with the radio coprocessor, so they must accept *every* bit pattern: a real `enum` with
/// an out-of-range discriminant is instant UB. Hence a transparent newtype with associated
/// constants instead of variants. Unknown values are preserved and printed numerically.
#[macro_export]
macro_rules! numeric_enum {
    (#[repr($repr:ident)]
     $(#$attrs:tt)* $vis:vis enum $name:ident {
        $($(#$value_attrs:tt)* $value:ident = $constant:expr),* $(,)?
    }
    $(default = $default:ident;)?
    ) => {
        $(#$attrs)*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        impl $name {
            $($(#$value_attrs)* pub const $value: Self = Self($constant);)*
        }

        $(
            impl ::core::default::Default for $name {
                fn default() -> Self {
                    Self::$default
                }
            }
        )?

        impl ::core::convert::From<$repr> for $name {
            fn from(value: $repr) -> Self {
                Self(value)
            }
        }

        impl ::core::convert::From<$name> for $repr {
            fn from(value: $name) -> $repr {
                value.0
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match *self {
                    $(Self::$value => f.write_str(::core::stringify!($value)),)*
                    Self(other) => ::core::write!(f, "Unknown({})", other),
                }
            }
        }

        // Formatting goes through a shadow enum rather than writing the name as a `{=str}`
        // argument: defmt propagates the enclosing `{=?}` hint into nested arguments, which would
        // render the name quoted. Derived enum formatting puts the name in the interned format
        // string instead, so it comes out bare.
        #[cfg(feature = "defmt")]
        const _: () = {
            #[derive(defmt::Format)]
            enum Fmt {
                $($value,)*
                Unknown($repr),
            }

            impl defmt::Format for $name {
                fn format(&self, f: defmt::Formatter) {
                    let fmt = match *self {
                        $(<$name>::$value => Fmt::$value,)*
                        $name(other) => Fmt::Unknown(other),
                    };

                    defmt::write!(f, "{}", fmt)
                }
            }
        };
    }
}
