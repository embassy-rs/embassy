use core::marker::PhantomData;

pub trait Packet {
    fn data(&self) -> &[u8];
    fn data_mut(&mut self) -> &mut [u8];

    #[inline]
    fn get<F: AsPacketField>(&self, field: &F) -> F::T {
        field.get(self.data())
    }

    #[inline]
    fn set<F: AsPacketField>(&mut self, field: &F, val: F::T) {
        field.set(self.data_mut(), val)
    }
}

pub trait AsPacketField {
    type T;

    fn get(&self, data: &[u8]) -> Self::T;
    fn set(&self, data: &mut [u8], val: Self::T);
}

pub struct BitField {
    pub byte: usize,
    pub bit: u8,
    pub mask: u8,
}

impl BitField {
    pub const fn new(byte: usize, bit: u8, size: u8) -> Self {
        let mask = (0xFF >> size) << bit;
        Self { byte, bit, mask }
    }
}

impl AsPacketField for BitField {
    type T = u8;

    #[inline]
    fn get(&self, data: &[u8]) -> Self::T {
        (data[self.byte] & self.mask) >> self.bit
    }

    #[inline]
    fn set(&self, data: &mut [u8], val: Self::T) {
        data[self.byte] = (val << self.bit) | (data[self.byte] & !self.mask)
    }
}

pub struct BoolField {
    pub byte: usize,
    pub bit: u8,
}

impl BoolField {
    pub const fn new(byte: usize, bit: u8) -> Self {
        Self { byte, bit }
    }
}

impl AsPacketField for BoolField {
    type T = bool;

    #[inline]
    fn get(&self, data: &[u8]) -> Self::T {
        data[self.byte] & (1 << self.bit) != 0
    }

    #[inline]
    fn set(&self, data: &mut [u8], val: Self::T) {
        data[self.byte] = ((val as u8) << self.bit) | (data[self.byte] & !(1 << self.bit))
    }
}

pub struct Field<T> {
    pub byte: usize,
    _phantom: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(byte: usize) -> Self {
        Self {
            byte,
            _phantom: PhantomData,
        }
    }
}

impl<T> AsPacketField for Field<T> {
    type T = T;

    #[inline]
    fn get(&self, data: &[u8]) -> Self::T {
        unsafe { core::ptr::read(data.as_ptr().offset(self.byte as _) as *const T) }
    }

    #[inline]
    fn set(&self, data: &mut [u8], val: Self::T) {
        unsafe { core::ptr::write(data.as_mut_ptr().offset(self.byte as _) as *mut T, val) }
    }
}

#[cfg(test)]
mod tests {
    use super::{AsPacketField, BitField, BoolField};

    #[test]
    fn bitfield() {
        let field = BitField::new(0, 4, 3);

        let mut data = [0b1111_1111];
        assert_eq!(field.get(&data), 0b111);

        field.set(&mut data, 0b000);
        assert_eq!(field.get(&data), 0b000);
        assert_eq!(data, [0b1000_1111]);
    }

    #[test]
    fn boolfield() {
        let field = BoolField::new(0, 5);

        let mut data = [0b1111_1111];
        assert_eq!(field.get(&data), true);

        field.set(&mut data, false);
        assert_eq!(field.get(&data), false);
        assert_eq!(data, [0b1101_1111]);
    }
}
