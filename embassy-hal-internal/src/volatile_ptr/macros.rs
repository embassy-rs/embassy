/// Provides safe field projection for volatile pointers referencing structs.
///
/// ## Examples
///
/// Accessing a struct field:
///
/// ```
/// use volatile::{VolatilePtr, map_field};
///
/// struct Example { field_1: u32, field_2: u8, }
/// let mut value = Example { field_1: 15, field_2: 255 };
/// let volatile = unsafe { VolatilePtr::new((&mut value).into()) };
///
/// // construct a volatile reference to a field
/// let field_2 = map_field!(volatile.field_2);
/// assert_eq!(field_2.read(), 255);
/// ```
///
/// Creating `VolatilePtr`s to unaligned field in packed structs is not allowed:
/// ```compile_fail
/// use volatile::{VolatilePtr, map_field};
///
/// #[repr(packed)]
/// struct Example { field_1: u8, field_2: usize, }
/// let mut value = Example { field_1: 15, field_2: 255 };
/// let volatile = unsafe { VolatilePtr::new((&mut value).into()) };
///
/// // Constructing a volatile reference to an unaligned field doesn't compile.
/// let field_2 = map_field!(volatile.field_2);
/// ```
#[macro_export]
macro_rules! map_field {
    ($volatile:ident.$($place:ident).+) => {{
        // Simulate creating a reference to the field. This is done to make
        // sure that the field is not potentially unaligned. The body of the
        // if statement will never be executed, so it can never cause any UB.
        if false {
            let _ref_to_field = &(unsafe { &*$volatile.as_raw_ptr().as_ptr() }).$($place).+;
        }

        unsafe {
            $volatile.map(|ptr| {
                core::ptr::NonNull::new(core::ptr::addr_of_mut!((*ptr.as_ptr()).$($place).+)).unwrap()
            })
        }
    }};
}
