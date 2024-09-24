use crate::can::filter::{ActivateFilter, ExtendedFilter, StandardFilter};

use super::CanLowLevel;

impl CanLowLevel {
    pub fn set_standard_filter(&self, slot: u8, filter: StandardFilter) {
        self.message_ram
            .standard_filter
            .get_mut(slot as usize)
            .data
            .activate(filter);
    }

    pub fn set_extended_filter(&self, slot: u8, filter: ExtendedFilter) {
        self.message_ram
            .extended_filter
            .get_mut(slot as usize)
            .data
            .activate(filter);
    }
}
