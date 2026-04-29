#[macro_export]
macro_rules! numeric_enum {
    (#[repr($repr:ident)]
     $(#$attrs:tt)* $vis:vis enum $name:ident {
        $($(#$enum_attrs:tt)* $enum:ident = $constant:expr),* $(,)?
    } ) => {
        #[repr($repr)]
        $(#$attrs)*
        $vis enum $name {
            $($(#$enum_attrs)* $enum = $constant),*
        }

        impl ::core::convert::TryFrom<$repr> for $name {
            type Error = ();

            fn try_from(value: $repr) -> ::core::result::Result<Self, ()> {
                match value {
                    $($constant => Ok( $name :: $enum ),)*
                    _ => Err(())
                }
            }
        }

        impl ::core::convert::From<$name> for $repr {
            fn from(value: $name) -> $repr {
                match value {
                    $($name :: $enum => $constant,)*
                }
            }
        }
    }
}
