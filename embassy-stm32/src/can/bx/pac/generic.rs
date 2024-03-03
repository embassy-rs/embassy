use core::marker;

///This trait shows that register has `read` method
///
///Registers marked with `Writable` can be also `modify`'ed
pub trait Readable {}

///This trait shows that register has `write`, `write_with_zero` and `reset` method
///
///Registers marked with `Readable` can be also `modify`'ed
pub trait Writable {}

///Reset value of the register
///
///This value is initial value for `write` method.
///It can be also directly writed to register by `reset` method.
pub trait ResetValue {
    ///Register size
    type Type;
    ///Reset value of the register
    fn reset_value() -> Self::Type;
}

///This structure provides volatile access to register
pub struct Reg<U, REG> {
    register: vcell::VolatileCell<U>,
    _marker: marker::PhantomData<REG>,
}

unsafe impl<U: Send, REG> Send for Reg<U, REG> {}

impl<U, REG> Reg<U, REG>
where
    Self: Readable,
    U: Copy,
{
    ///Reads the contents of `Readable` register
    ///
    ///You can read the contents of a register in such way:
    ///```ignore
    ///let bits = periph.reg.read().bits();
    ///```
    ///or get the content of a particular field of a register.
    ///```ignore
    ///let reader = periph.reg.read();
    ///let bits = reader.field1().bits();
    ///let flag = reader.field2().bit_is_set();
    ///```
    #[inline(always)]
    pub fn read(&self) -> R<U, Self> {
        R {
            bits: self.register.get(),
            _reg: marker::PhantomData,
        }
    }
}

impl<U, REG> Reg<U, REG>
where
    Self: ResetValue<Type = U> + Writable,
    U: Copy,
{
    ///Writes the reset value to `Writable` register
    ///
    ///Resets the register to its initial state
    #[inline(always)]
    pub fn reset(&self) {
        self.register.set(Self::reset_value())
    }
}

impl<U, REG> Reg<U, REG>
where
    Self: ResetValue<Type = U> + Writable,
    U: Copy,
{
    ///Writes bits to `Writable` register
    ///
    ///You can write raw bits into a register:
    ///```ignore
    ///periph.reg.write(|w| unsafe { w.bits(rawbits) });
    ///```
    ///or write only the fields you need:
    ///```ignore
    ///periph.reg.write(|w| w
    ///    .field1().bits(newfield1bits)
    ///    .field2().set_bit()
    ///    .field3().variant(VARIANT)
    ///);
    ///```
    ///Other fields will have reset value.
    #[inline(always)]
    pub fn write<F>(&self, f: F)
    where
        F: FnOnce(&mut W<U, Self>) -> &mut W<U, Self>,
    {
        self.register.set(
            f(&mut W {
                bits: Self::reset_value(),
                _reg: marker::PhantomData,
            })
            .bits,
        );
    }
}

impl<U, REG> Reg<U, REG>
where
    Self: Writable,
    U: Copy + Default,
{
    ///Writes Zero to `Writable` register
    ///
    ///Similar to `write`, but unused bits will contain 0.
    #[inline(always)]
    pub fn write_with_zero<F>(&self, f: F)
    where
        F: FnOnce(&mut W<U, Self>) -> &mut W<U, Self>,
    {
        self.register.set(
            f(&mut W {
                bits: U::default(),
                _reg: marker::PhantomData,
            })
            .bits,
        );
    }
}

impl<U, REG> Reg<U, REG>
where
    Self: Readable + Writable,
    U: Copy,
{
    ///Modifies the contents of the register
    ///
    ///E.g. to do a read-modify-write sequence to change parts of a register:
    ///```ignore
    ///periph.reg.modify(|r, w| unsafe { w.bits(
    ///   r.bits() | 3
    ///) });
    ///```
    ///or
    ///```ignore
    ///periph.reg.modify(|_, w| w
    ///    .field1().bits(newfield1bits)
    ///    .field2().set_bit()
    ///    .field3().variant(VARIANT)
    ///);
    ///```
    ///Other fields will have value they had before call `modify`.
    #[inline(always)]
    pub fn modify<F>(&self, f: F)
    where
        for<'w> F: FnOnce(&R<U, Self>, &'w mut W<U, Self>) -> &'w mut W<U, Self>,
    {
        let bits = self.register.get();
        self.register.set(
            f(
                &R {
                    bits,
                    _reg: marker::PhantomData,
                },
                &mut W {
                    bits,
                    _reg: marker::PhantomData,
                },
            )
            .bits,
        );
    }
}

///Register/field reader
///
///Result of the [`read`](Reg::read) method of a register.
///Also it can be used in the [`modify`](Reg::read) method
pub struct R<U, T> {
    pub(crate) bits: U,
    _reg: marker::PhantomData<T>,
}

impl<U, T> R<U, T>
where
    U: Copy,
{
    ///Create new instance of reader
    #[inline(always)]
    pub(crate) fn new(bits: U) -> Self {
        Self {
            bits,
            _reg: marker::PhantomData,
        }
    }
    ///Read raw bits from register/field
    #[inline(always)]
    pub fn bits(&self) -> U {
        self.bits
    }
}

impl<U, T, FI> PartialEq<FI> for R<U, T>
where
    U: PartialEq,
    FI: Copy + Into<U>,
{
    #[inline(always)]
    fn eq(&self, other: &FI) -> bool {
        self.bits.eq(&(*other).into())
    }
}

impl<FI> R<bool, FI> {
    ///Value of the field as raw bits
    #[inline(always)]
    pub fn bit(&self) -> bool {
        self.bits
    }
    ///Returns `true` if the bit is clear (0)
    #[inline(always)]
    pub fn bit_is_clear(&self) -> bool {
        !self.bit()
    }
    ///Returns `true` if the bit is set (1)
    #[inline(always)]
    pub fn bit_is_set(&self) -> bool {
        self.bit()
    }
}

///Register writer
///
///Used as an argument to the closures in the [`write`](Reg::write) and [`modify`](Reg::modify) methods of the register
pub struct W<U, REG> {
    ///Writable bits
    pub(crate) bits: U,
    _reg: marker::PhantomData<REG>,
}

impl<U, REG> W<U, REG> {
    ///Writes raw bits to the register
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: U) -> &mut Self {
        self.bits = bits;
        self
    }
}

///Used if enumerated values cover not the whole range
#[derive(Clone, Copy, PartialEq)]
pub enum Variant<U, T> {
    ///Expected variant
    Val(T),
    ///Raw bits
    Res(U),
}
