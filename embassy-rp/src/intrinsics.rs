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
/// ```rust,ignore
/// intrinsics! {
///     extern "C" fn foo(a: i32) -> u32 {
///         // ...
///     }
///     #[nonstandard_attribute]
///     extern "C" fn bar(a: i32) -> u32 {
///         // ...
///     }
/// }
/// ```
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

// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/sio.rs

// This takes advantage of how AAPCS defines a 64-bit return on 32-bit registers
// by packing it into r0[0:31] and r1[32:63].  So all we need to do is put
// the remainder in the high order 32 bits of a 64 bit result.   We can also
// alias the division operators to these for a similar reason r0 is the
// result either way and r1 a scratch register, so the caller can't assume it
// retains the argument value.
#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    ".macro hwdivider_head",
    "ldr    r2, =(0xd0000000)", // SIO_BASE
    // Check the DIRTY state of the divider by shifting it into the C
    // status bit.
    "ldr    r3, [r2, #0x078]", // DIV_CSR
    "lsrs   r3, #2",           // DIRTY = 1, so shift 2 down
    // We only need to save the state when DIRTY, otherwise we can just do the
    // division directly.
    "bcs    2f",
    "1:",
    // Do the actual division now, we're either not DIRTY, or we've saved the
    // state and branched back here so it's safe now.
    ".endm",
    ".macro hwdivider_tail",
    // 8 cycle delay to wait for the result.  Each branch takes two cycles
    // and fits into a 2-byte Thumb instruction, so this is smaller than
    // 8 NOPs.
    "b      3f",
    "3: b   3f",
    "3: b   3f",
    "3: b   3f",
    "3:",
    // Read the quotient last, since that's what clears the dirty flag.
    "ldr    r1, [r2, #0x074]", // DIV_REMAINDER
    "ldr    r0, [r2, #0x070]", // DIV_QUOTIENT
    // Either return to the caller or back to the state restore.
    "bx     lr",
    "2:",
    // Since we can't save the signed-ness of the calculation, we have to make
    // sure that there's at least an 8 cycle delay before we read the result.
    // The push takes 5 cycles, and we've already spent at least 7 checking
    // the DIRTY state to get here.
    "push   {{r4-r6, lr}}",
    // Read the quotient last, since that's what clears the dirty flag.
    "ldr    r3, [r2, #0x060]", // DIV_UDIVIDEND
    "ldr    r4, [r2, #0x064]", // DIV_UDIVISOR
    "ldr    r5, [r2, #0x074]", // DIV_REMAINDER
    "ldr    r6, [r2, #0x070]", // DIV_QUOTIENT
    // If we get interrupted here (before a write sets the DIRTY flag) it's
    // fine, since we have the full state, so the interruptor doesn't have to
    // restore it.  Once the write happens and the DIRTY flag is set, the
    // interruptor becomes responsible for restoring our state.
    "bl     1b",
    // If we are interrupted here, then the interruptor will start an incorrect
    // calculation using a wrong divisor, but we'll restore the divisor and
    // result ourselves correctly. This sets DIRTY, so any interruptor will
    // save the state.
    "str    r3, [r2, #0x060]", // DIV_UDIVIDEND
    // If we are interrupted here, the interruptor may start the calculation
    // using incorrectly signed inputs, but we'll restore the result ourselves.
    // This sets DIRTY, so any interruptor will save the state.
    "str    r4, [r2, #0x064]", // DIV_UDIVISOR
    // If we are interrupted here, the interruptor will have restored
    // everything but the quotient may be wrongly signed.  If the calculation
    // started by the above writes is still ongoing it is stopped, so it won't
    // replace the result we're restoring.  DIRTY and READY set, but only
    // DIRTY matters to make the interruptor save the state.
    "str    r5, [r2, #0x074]", // DIV_REMAINDER
    // State fully restored after the quotient write.  This sets both DIRTY
    // and READY, so whatever we may have interrupted can read the result.
    "str    r6, [r2, #0x070]", // DIV_QUOTIENT
    "pop    {{r4-r6, pc}}",
    ".endm",
);

macro_rules! division_function {
    (
        $name:ident $($intrinsic:ident)* ( $argty:ty ) {
            $($begin:literal),+
        }
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        core::arch::global_asm!(
            // Mangle the name slightly, since this is a global symbol.
            concat!(".section .text._erphal_", stringify!($name)),
            concat!(".global _erphal_", stringify!($name)),
            concat!(".type _erphal_", stringify!($name), ", %function"),
            ".align 2",
            concat!("_erphal_", stringify!($name), ":"),
            $(
                concat!(".global ", stringify!($intrinsic)),
                concat!(".type ", stringify!($intrinsic), ", %function"),
                concat!(stringify!($intrinsic), ":"),
            )*

            "hwdivider_head",
            $($begin),+ ,
            "hwdivider_tail",
        );

        #[cfg(all(target_arch = "arm", not(feature = "intrinsics")))]
        core::arch::global_asm!(
            // Mangle the name slightly, since this is a global symbol.
            concat!(".section .text._erphal_", stringify!($name)),
            concat!(".global _erphal_", stringify!($name)),
            concat!(".type _erphal_", stringify!($name), ", %function"),
            ".align 2",
            concat!("_erphal_", stringify!($name), ":"),

            "hwdivider_head",
            $($begin),+ ,
            "hwdivider_tail",
        );

        #[cfg(target_arch = "arm")]
        extern "aapcs" {
            // Connect a local name to global symbol above through FFI.
            #[link_name = concat!("_erphal_", stringify!($name)) ]
            fn $name(n: $argty, d: $argty) -> u64;
        }

        #[cfg(not(target_arch = "arm"))]
        #[allow(unused_variables)]
        unsafe fn $name(n: $argty, d: $argty) -> u64 { 0 }
    };
}

division_function! {
    unsigned_divmod __aeabi_uidivmod __aeabi_uidiv ( u32 ) {
        "str    r0, [r2, #0x060]", // DIV_UDIVIDEND
        "str    r1, [r2, #0x064]"  // DIV_UDIVISOR
    }
}

division_function! {
    signed_divmod __aeabi_idivmod __aeabi_idiv ( i32 ) {
        "str    r0, [r2, #0x068]", // DIV_SDIVIDEND
        "str    r1, [r2, #0x06c]"  // DIV_SDIVISOR
    }
}

fn divider_unsigned(n: u32, d: u32) -> DivResult<u32> {
    let packed = unsafe { unsigned_divmod(n, d) };
    DivResult {
        quotient: packed as u32,
        remainder: (packed >> 32) as u32,
    }
}

fn divider_signed(n: i32, d: i32) -> DivResult<i32> {
    let packed = unsafe { signed_divmod(n, d) };
    // Double casts to avoid sign extension
    DivResult {
        quotient: packed as u32 as i32,
        remainder: (packed >> 32) as u32 as i32,
    }
}

/// Result of divide/modulo operation
struct DivResult<T> {
    /// The quotient of divide/modulo operation
    pub quotient: T,
    /// The remainder of divide/modulo operation
    pub remainder: T,
}

intrinsics! {
    extern "C" fn __udivsi3(n: u32, d: u32) -> u32 {
        divider_unsigned(n, d).quotient
    }

    extern "C" fn __umodsi3(n: u32, d: u32) -> u32 {
        divider_unsigned(n, d).remainder
    }

    extern "C" fn __udivmodsi4(n: u32, d: u32, rem: Option<&mut u32>) -> u32 {
        let quo_rem = divider_unsigned(n, d);
        if let Some(rem) = rem {
            *rem = quo_rem.remainder;
        }
        quo_rem.quotient
    }

    extern "C" fn __divsi3(n: i32, d: i32) -> i32 {
        divider_signed(n, d).quotient
    }

    extern "C" fn __modsi3(n: i32, d: i32) -> i32 {
        divider_signed(n, d).remainder
    }

    extern "C" fn __divmodsi4(n: i32, d: i32, rem: &mut i32) -> i32 {
        let quo_rem = divider_signed(n, d);
        *rem = quo_rem.remainder;
        quo_rem.quotient
    }
}
