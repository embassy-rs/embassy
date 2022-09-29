#![macro_use]
// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/intrinsics.rs

/// Generate a series of aliases for an intrinsic function.
macro_rules! intrinsics_aliases {
    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
    ) => {};
    (
        unsafe extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
    ) => {};

    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
        $alias:ident
        $($rest:ident)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        intrinsics! {
            extern $abi fn $alias( $($argname: $ty),* ) -> $ret {
                $name($($argname),*)
            }
        }

        intrinsics_aliases! {
            extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($rest)*
        }
    };

    (
        unsafe extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
        $alias:ident
        $($rest:ident)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        intrinsics! {
            unsafe extern $abi fn $alias( $($argname: $ty),* ) -> $ret {
                $name($($argname),*)
            }
        }

        intrinsics_aliases! {
            unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($rest)*
        }
    };
}

/// The macro used to define overridden intrinsics.
///
/// This is heavily inspired by the macro used by compiler-builtins.  The idea
/// is to abstract anything special that needs to be done to override an
/// intrinsic function.  Intrinsic generation is disabled for non-ARM targets
/// so things like CI and docs generation do not have problems.  Additionally
/// they can be disabled by disabling the crate feature `intrinsics` for
/// testing or comparing performance.
///
/// Like the compiler-builtins macro, it accepts a series of functions that
/// looks like normal Rust code:
///
///     intrinsics! {
///         extern "C" fn foo(a: i32) -> u32 {
///             // ...
///         }
///
///         #[nonstandard_attribute]
///         extern "C" fn bar(a: i32) -> u32 {
///             // ...
///         }
///     }
///
/// Each function can also be decorated with nonstandard attributes to control
/// additional behaviour:
///
/// * `slower_than_default` - indicates that the override is slower than the
///   default implementation.  Currently this just disables the override
///   entirely.
/// * `bootrom_v2` - indicates that the override is only available
///   on a V2 bootrom or higher.  Only enabled when the feature
///   `rom-v2-intrinsics` is set.
/// * `alias` - accepts a list of names to alias the intrinsic to.
/// * `aeabi` - accepts a list of ARM EABI names to alias to.
///
macro_rules! intrinsics {
    () => {};

    (
        #[slower_than_default]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        // Not exported, but defined so the actual implementation is
        // considered used
        #[allow(dead_code)]
        fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        intrinsics!($($rest)*);
    };

    (
        #[bootrom_v2]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        // Not exported, but defined so the actual implementation is
        // considered used
        #[cfg(not(feature = "rom-v2-intrinsics"))]
        #[allow(dead_code)]
        fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        #[cfg(feature = "rom-v2-intrinsics")]
        intrinsics! {
            $(#[$($attr)*])*
            extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                $($body)*
            }
        }

        intrinsics!($($rest)*);
    };

    (
        #[alias = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                $($body)*
            }
        }

        intrinsics_aliases! {
            extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        #[alias = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        unsafe extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                $($body)*
            }
        }

        intrinsics_aliases! {
            unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        #[aeabi = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                $($body)*
            }
        }

        intrinsics_aliases! {
            extern "aapcs" fn $name( $($argname: $ty),* ) -> $ret,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        $(#[$($attr)*])*
        extern $abi fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        mod $name {
            #[no_mangle]
            $(#[$($attr)*])*
            pub extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                super::$name($($argname),*)
            }
        }

        // Not exported, but defined so the actual implementation is
        // considered used
        #[cfg(not(all(target_arch = "arm", feature = "intrinsics")))]
        #[allow(dead_code)]
        fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        intrinsics!($($rest)*);
    };

    (
        $(#[$($attr:tt)*])*
        unsafe extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        $(#[$($attr)*])*
        unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        mod $name {
            #[no_mangle]
            $(#[$($attr)*])*
            pub unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret {
                super::$name($($argname),*)
            }
        }

        // Not exported, but defined so the actual implementation is
        // considered used
        #[cfg(not(all(target_arch = "arm", feature = "intrinsics")))]
        #[allow(dead_code)]
        unsafe fn $name( $($argname: $ty),* ) -> $ret {
            $($body)*
        }

        intrinsics!($($rest)*);
    };
}
