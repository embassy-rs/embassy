//! This module contains filter definitions and configuration that is common to both Classic CAN and FDCAN.

pub use embedded_can::{Id, StandardId, ExtendedId};

/// Possible errors when configuring a RX filter.
pub enum FilterConfigError {
    /// This error indicates that you have attempted to configure
    /// too many filters, or that your split of filters was invalid.
    /// 
    /// Note: Your split of standard and extended
    /// filters must adhere to the constraint:
    /// 2*(num_extended) + (num_standard) ≤ 32
    TooManyFilters,

    /// This error indicates that you've attempted to construct an
    /// empty filter, which is invalid.
    EmptyFilterConfig,
}

/// Represents the different possible kinds of RX filters.
///
/// A masked filter accepts any ID where every bit set in `mask` matches the
/// corresponding bit of `id`. Bits cleared in `mask` are "don't care". A plain
/// (unmasked) filter is just a masked filter whose mask is all-ones, i.e. an
/// exact match.
pub enum Filter {
    /// Filter for a single standard ID.
    Standard(StandardId),

    /// Filter for a single extended ID.
    Extended(ExtendedId),

    /// Filter for a standard ID, with a mask.
    StandardMasked { id: StandardId, mask: StandardId },

    /// Filter for an extended ID, with a mask.
    ExtendedMasked { id: ExtendedId, mask: ExtendedId },

    /// Filter that accepts all standard IDs.
    /// 
    /// Note: Configuring this effectively makes any other
    /// standard filters you configure redundant, since it causes all standard
    /// IDs to be accepted.
    AcceptAllStandard,

    /// Filter that accepts all extended IDs.
    /// 
    /// Note: Configuring this effectively makes any other
    /// extended filters you configure redundant, since it causes all extended
    /// IDs to be accepted.
    AcceptAllExtended,
}

impl Filter {
    /// Returns `true` if this filter targets an extended ID.
    pub(in crate::flexcan) const fn is_extended(&self) -> bool {
        matches!(self, Filter::Extended(_) | Filter::ExtendedMasked { .. } | Filter::AcceptAllExtended)
    }
}

/// Struct for configuring your RX ID filters. A `FilterConfig` can be constructed via 
/// the `filters!()` macro, like this:
/// ```rust
/// use embassy_mcxa::flexcan::filter::{Filter, filters, StandardId, ExtendedId};
/// 
/// const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFFF).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x100).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_THREE_MASK: StandardId = StandardId::new(0x7F0).expect("Invalid mask (too large).");
///
/// let filters = filters!(
///     Filter::Standard(EXAMPLE_MESSAGE_ONE),
///     Filter::Extended(EXAMPLE_MESSAGE_TWO),
///     Filter::StandardMasked { id: EXAMPLE_MESSAGE_THREE, mask: EXAMPLE_MESSAGE_THREE_MASK },
/// );
/// ```
/// Your config must adhere to the constraint: 2*(num_extended) + (num_standard) ≤ 32.
/// 
/// Notes: 
/// - If you need to reconfigure your filters dynamically based on runtime values, see `FilterConfig::try_new()`.
/// - If you don't care about filtering and just want to accept all incoming messages, see `FilterConfig::accept_all()`.
/// You can also use `Filter::AcceptAllStandard` and `Filter::AcceptAllExtended` directly inside `filters!()`/`FilterConfig::try_new()` if you require a more specific configuration.
pub struct FilterConfig<'a> {
    /// List of filters.
    pub(crate) filters: &'a [Filter],

    /// Number of standard filters.
    pub(crate) num_standard: usize,

    /// Number of extended filters.
    pub(crate) num_extended: usize,
}

impl<'a> FilterConfig<'a> {
    /// This is an internal function that should only be
    /// called via the `filters` macro.
    /// 
    /// This function calls Self::try_new(), but panics when an error is returned. This generates
    /// a nice compile-time error, so long as `__filters()` is called from a `const` context. The purpose
    /// of the `filters` macro is to ensure `__filters()` can only be called from a `const` context,
    /// so we can do all the nice compile-time validation stuff without the possibility of a runtime panic.
    #[doc(hidden)]
    pub const fn __filters(filters: &'a [Filter]) -> Self {
        match Self::try_new(filters) {
            Ok(me) => me,
            Err(FilterConfigError::TooManyFilters) => { panic!("Invalid FilterConfig (TooManyFilters)! A FilterConfig must adhere to the constraint `2*(num_extended) + (num_standard) <= 32`."); }
            Err(FilterConfigError::EmptyFilterConfig) => { panic!("Invalid FilterConfig (EmptyFilterConfig)! A FilterConfig cannot be empty."); }
        }
    }

    /// Creates a new `FilterConfig` from a declarative list of filters.
    /// 
    /// The preferred way of constructing a `FilterConfig` is through the
    /// `filters!()` macro (since it evaluates at compile-time), but this function
    /// may be useful if you need to reconfigure filters based on runtime values.
    pub const fn try_new(filters: &'a [Filter]) -> Result<Self, FilterConfigError> {
        if filters.is_empty() {
            return Err(FilterConfigError::EmptyFilterConfig);
        }

        // Calculate how many of each type of filter there is
        let mut num_standard = 0;
        let mut num_extended = 0;
        let mut i = 0;
        while i < filters.len() {
            if filters[i].is_extended() {
                num_extended += 1;
            } else {
                num_standard += 1;
            }
            i += 1;
        }

        // FilterConfigs need to adhere to the constraint: 2*(num_extended) + (num_standard) <= 32
        if 2 * num_extended + num_standard > 32 {
            return Err(FilterConfigError::TooManyFilters);
        }
        
        Ok(Self { filters, num_standard, num_extended })
    }

    /// Returns a `FilterConfig` that accepts all IDs.
    pub const fn accept_all() -> Self {
        Self {
            filters: &[Filter::AcceptAllStandard, Filter::AcceptAllExtended],
            num_extended: 1,
            num_standard: 1
        }
    }
}

impl Default for FilterConfig<'_> {
    /// Returns a `FilterConfig` that accepts all IDs.
    fn default() -> Self {
        FilterConfig::accept_all()
    }
}

/// Macro for constructing a `FilterConfig`. Can be used via this general syntax:
/// ```rust
/// use embassy_mcxa::flexcan::filter::{Filter, FilterConfig, filters, StandardId, ExtendedId};
/// 
/// const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFFF).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x100).expect("Invalid ID (too large).");
/// const EXAMPLE_MESSAGE_THREE_MASK: StandardId = StandardId::new(0x7F0).expect("Invalid mask (too large).");
///
/// let filters: FilterConfig = filters!(
///     Filter::Standard(EXAMPLE_MESSAGE_ONE),
///     Filter::Extended(EXAMPLE_MESSAGE_TWO),
///     Filter::StandardMasked { id: EXAMPLE_MESSAGE_THREE, mask: EXAMPLE_MESSAGE_THREE_MASK },
/// );
/// ```
/// Your config must adhere to the constraint: 2*(num_extended) + (num_standard) ≤ 32.
/// 
/// Note: If you need to reconfigure your filters dynamically based on runtime values, see `FilterConfig::try_new()`.
#[doc(hidden)] #[macro_export]
macro_rules! __filters_macro {
    ($($f:expr),* $(,)?) => {
        const { $crate::flexcan::filter::FilterConfig::__filters(&[$($f),*]) }
    };
}

#[doc(inline)]
pub use __filters_macro as filters;
// apparently this is the only (?) way to export a macro as part of your module rather than the top level crate
// ref: https://internals.rust-lang.org/t/pub-on-macro-rules/19358/16?u=dhm