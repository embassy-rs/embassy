/// A handle for a USB interface that contains its number.
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceNumber(pub(crate) u8);

impl InterfaceNumber {
    pub(crate) fn new(index: u8) -> InterfaceNumber {
        InterfaceNumber(index)
    }
}

impl From<InterfaceNumber> for u8 {
    fn from(n: InterfaceNumber) -> u8 {
        n.0
    }
}

/// A handle for a USB string descriptor that contains its index.
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StringIndex(u8);

impl StringIndex {
    pub(crate) fn new(index: u8) -> StringIndex {
        StringIndex(index)
    }
}

impl From<StringIndex> for u8 {
    fn from(i: StringIndex) -> u8 {
        i.0
    }
}
