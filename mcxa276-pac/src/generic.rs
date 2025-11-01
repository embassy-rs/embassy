use core::marker;
#[doc = " Generic peripheral accessor"]
pub struct Periph<RB, const A: usize> {
    _marker: marker::PhantomData<RB>,
}
unsafe impl<RB, const A: usize> Send for Periph<RB, A> {}
impl<RB, const A: usize> Periph<RB, A> {
    #[doc = "Pointer to the register block"]
    pub const PTR: *const RB = A as *const _;
    #[doc = "Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const RB {
        Self::PTR
    }
    #[doc = " Steal an instance of this peripheral"]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " Ensure that the new instance of the peripheral cannot be used in a way"]
    #[doc = " that may race with any existing instances, for example by only"]
    #[doc = " accessing read-only or write-only registers, or by consuming the"]
    #[doc = " original peripheral and using critical sections to coordinate"]
    #[doc = " access between multiple new instances."]
    #[doc = ""]
    #[doc = " Additionally, other software such as HALs may rely on only one"]
    #[doc = " peripheral instance existing to ensure memory safety; ensure"]
    #[doc = " no stolen instances are passed to such software."]
    pub unsafe fn steal() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}
impl<RB, const A: usize> core::ops::Deref for Periph<RB, A> {
    type Target = RB;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
#[doc = " Raw register type (`u8`, `u16`, `u32`, ...)"]
pub trait RawReg:
    Copy
    + From<bool>
    + core::ops::BitOr<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOrAssign
    + core::ops::BitAndAssign
    + core::ops::Not<Output = Self>
    + core::ops::Shl<u8, Output = Self>
{
    #[doc = " Mask for bits of width `WI`"]
    fn mask<const WI: u8>() -> Self;
    #[doc = " `0`"]
    const ZERO: Self;
    #[doc = " `1`"]
    const ONE: Self;
}
macro_rules! raw_reg {
    ($ U : ty , $ size : literal , $ mask : ident) => {
        impl RawReg for $U {
            #[inline(always)]
            fn mask<const WI: u8>() -> Self {
                $mask::<WI>()
            }
            const ZERO: Self = 0;
            const ONE: Self = 1;
        }
        const fn $mask<const WI: u8>() -> $U {
            <$U>::MAX >> ($size - WI)
        }
        impl FieldSpec for $U {
            type Ux = $U;
        }
    };
}
raw_reg!(u8, 8, mask_u8);
raw_reg!(u16, 16, mask_u16);
raw_reg!(u32, 32, mask_u32);
raw_reg!(u64, 64, mask_u64);
#[doc = " Raw register type"]
pub trait RegisterSpec {
    #[doc = " Raw register type (`u8`, `u16`, `u32`, ...)."]
    type Ux: RawReg;
}
#[doc = " Raw field type"]
pub trait FieldSpec: Sized {
    #[doc = " Raw field type (`u8`, `u16`, `u32`, ...)."]
    type Ux: Copy + core::fmt::Debug + PartialEq + From<Self>;
}
#[doc = " Marker for fields with fixed values"]
pub trait IsEnum: FieldSpec {}
#[doc = " Trait implemented by readable registers to enable the `read` method."]
#[doc = ""]
#[doc = " Registers marked with `Writable` can be also be `modify`'ed."]
pub trait Readable: RegisterSpec {}
#[doc = " Trait implemented by writeable registers."]
#[doc = ""]
#[doc = " This enables the  `write`, `write_with_zero` and `reset` methods."]
#[doc = ""]
#[doc = " Registers marked with `Readable` can be also be `modify`'ed."]
pub trait Writable: RegisterSpec {
    #[doc = " Is it safe to write any bits to register"]
    type Safety;
    #[doc = " Specifies the register bits that are not changed if you pass `1` and are changed if you pass `0`"]
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = Self::Ux::ZERO;
    #[doc = " Specifies the register bits that are not changed if you pass `0` and are changed if you pass `1`"]
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = Self::Ux::ZERO;
}
#[doc = " Reset value of the register."]
#[doc = ""]
#[doc = " This value is the initial value for the `write` method. It can also be directly written to the"]
#[doc = " register by using the `reset` method."]
pub trait Resettable: RegisterSpec {
    #[doc = " Reset value of the register."]
    const RESET_VALUE: Self::Ux = Self::Ux::ZERO;
    #[doc = " Reset value of the register."]
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        Self::RESET_VALUE
    }
}
#[doc(hidden)]
pub mod raw;
#[doc = " Register reader."]
#[doc = ""]
#[doc = " Result of the `read` methods of registers. Also used as a closure argument in the `modify`"]
#[doc = " method."]
pub type R<REG> = raw::R<REG>;
impl<REG: RegisterSpec> R<REG> {
    #[doc = " Reads raw bits from register."]
    #[inline(always)]
    pub const fn bits(&self) -> REG::Ux {
        self.bits
    }
}
impl<REG: RegisterSpec, FI> PartialEq<FI> for R<REG>
where
    REG::Ux: PartialEq,
    FI: Copy,
    REG::Ux: From<FI>,
{
    #[inline(always)]
    fn eq(&self, other: &FI) -> bool {
        self.bits.eq(&REG::Ux::from(*other))
    }
}
#[doc = " Register writer."]
#[doc = ""]
#[doc = " Used as an argument to the closures in the `write` and `modify` methods of the register."]
pub type W<REG> = raw::W<REG>;
impl<REG: Writable> W<REG> {
    #[doc = " Writes raw bits to the register."]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " Passing incorrect value can cause undefined behaviour. See reference manual"]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: REG::Ux) -> &mut Self {
        self.bits = bits;
        self
    }
}
impl<REG> W<REG>
where
    REG: Writable<Safety = Safe>,
{
    #[doc = " Writes raw bits to the register."]
    #[inline(always)]
    pub fn set(&mut self, bits: REG::Ux) -> &mut Self {
        self.bits = bits;
        self
    }
}
#[doc = " Field reader."]
#[doc = ""]
#[doc = " Result of the `read` methods of fields."]
pub type FieldReader<FI = u8> = raw::FieldReader<FI>;
#[doc = " Bit-wise field reader"]
pub type BitReader<FI = bool> = raw::BitReader<FI>;
impl<FI: FieldSpec> FieldReader<FI> {
    #[doc = " Reads raw bits from field."]
    #[inline(always)]
    pub const fn bits(&self) -> FI::Ux {
        self.bits
    }
}
impl<FI: FieldSpec> core::fmt::Debug for FieldReader<FI> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.bits, f)
    }
}
impl<FI> PartialEq<FI> for FieldReader<FI>
where
    FI: FieldSpec + Copy,
{
    #[inline(always)]
    fn eq(&self, other: &FI) -> bool {
        self.bits.eq(&FI::Ux::from(*other))
    }
}
impl<FI> PartialEq<FI> for BitReader<FI>
where
    FI: Copy,
    bool: From<FI>,
{
    #[inline(always)]
    fn eq(&self, other: &FI) -> bool {
        self.bits.eq(&bool::from(*other))
    }
}
impl<FI> BitReader<FI> {
    #[doc = " Value of the field as raw bits."]
    #[inline(always)]
    pub const fn bit(&self) -> bool {
        self.bits
    }
    #[doc = " Returns `true` if the bit is clear (0)."]
    #[inline(always)]
    pub const fn bit_is_clear(&self) -> bool {
        !self.bit()
    }
    #[doc = " Returns `true` if the bit is set (1)."]
    #[inline(always)]
    pub const fn bit_is_set(&self) -> bool {
        self.bit()
    }
}
impl<FI> core::fmt::Debug for BitReader<FI> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.bits, f)
    }
}
#[doc = " Marker for register/field writers which can take any value of specified width"]
pub struct Safe;
#[doc = " You should check that value is allowed to pass to register/field writer marked with this"]
pub struct Unsafe;
#[doc = " Marker for field writers are safe to write in specified inclusive range"]
pub struct Range<const MIN: u64, const MAX: u64>;
#[doc = " Marker for field writers are safe to write in specified inclusive range"]
pub struct RangeFrom<const MIN: u64>;
#[doc = " Marker for field writers are safe to write in specified inclusive range"]
pub struct RangeTo<const MAX: u64>;
#[doc = " Write field Proxy"]
pub type FieldWriter<'a, REG, const WI: u8, FI = u8, Safety = Unsafe> =
    raw::FieldWriter<'a, REG, WI, FI, Safety>;
impl<REG, const WI: u8, FI, Safety> FieldWriter<'_, REG, WI, FI, Safety>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
{
    #[doc = " Field width"]
    pub const WIDTH: u8 = WI;
    #[doc = " Field width"]
    #[inline(always)]
    pub const fn width(&self) -> u8 {
        WI
    }
    #[doc = " Field offset"]
    #[inline(always)]
    pub const fn offset(&self) -> u8 {
        self.o
    }
}
impl<'a, REG, const WI: u8, FI, Safety> FieldWriter<'a, REG, WI, FI, Safety>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
{
    #[doc = " Writes raw bits to the field"]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " Passing incorrect value can cause undefined behaviour. See reference manual"]
    #[inline(always)]
    pub unsafe fn bits(self, value: FI::Ux) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::mask::<WI>() << self.o);
        self.w.bits |= (REG::Ux::from(value) & REG::Ux::mask::<WI>()) << self.o;
        self.w
    }
}
impl<'a, REG, const WI: u8, FI> FieldWriter<'a, REG, WI, FI, Safe>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
{
    #[doc = " Writes raw bits to the field"]
    #[inline(always)]
    pub fn set(self, value: FI::Ux) -> &'a mut W<REG> {
        unsafe { self.bits(value) }
    }
}
impl<'a, REG, const WI: u8, FI, const MIN: u64, const MAX: u64>
    FieldWriter<'a, REG, WI, FI, Range<MIN, MAX>>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
    u64: From<FI::Ux>,
{
    #[doc = " Writes raw bits to the field"]
    #[inline(always)]
    pub fn set(self, value: FI::Ux) -> &'a mut W<REG> {
        {
            let value = u64::from(value);
            assert!(value >= MIN && value <= MAX);
        }
        unsafe { self.bits(value) }
    }
}
impl<'a, REG, const WI: u8, FI, const MIN: u64> FieldWriter<'a, REG, WI, FI, RangeFrom<MIN>>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
    u64: From<FI::Ux>,
{
    #[doc = " Writes raw bits to the field"]
    #[inline(always)]
    pub fn set(self, value: FI::Ux) -> &'a mut W<REG> {
        {
            let value = u64::from(value);
            assert!(value >= MIN);
        }
        unsafe { self.bits(value) }
    }
}
impl<'a, REG, const WI: u8, FI, const MAX: u64> FieldWriter<'a, REG, WI, FI, RangeTo<MAX>>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
    u64: From<FI::Ux>,
{
    #[doc = " Writes raw bits to the field"]
    #[inline(always)]
    pub fn set(self, value: FI::Ux) -> &'a mut W<REG> {
        {
            let value = u64::from(value);
            assert!(value <= MAX);
        }
        unsafe { self.bits(value) }
    }
}
impl<'a, REG, const WI: u8, FI, Safety> FieldWriter<'a, REG, WI, FI, Safety>
where
    REG: Writable + RegisterSpec,
    FI: IsEnum,
    REG::Ux: From<FI::Ux>,
{
    #[doc = " Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: FI) -> &'a mut W<REG> {
        unsafe { self.bits(FI::Ux::from(variant)) }
    }
}
macro_rules! bit_proxy {
    ($ writer : ident , $ mwv : ident) => {
        #[doc(hidden)]
        pub struct $mwv;
        #[doc = " Bit-wise write field proxy"]
        pub type $writer<'a, REG, FI = bool> = raw::BitWriter<'a, REG, FI, $mwv>;
        impl<'a, REG, FI> $writer<'a, REG, FI>
        where
            REG: Writable + RegisterSpec,
            bool: From<FI>,
        {
            #[doc = " Field width"]
            pub const WIDTH: u8 = 1;
            #[doc = " Field width"]
            #[inline(always)]
            pub const fn width(&self) -> u8 {
                Self::WIDTH
            }
            #[doc = " Field offset"]
            #[inline(always)]
            pub const fn offset(&self) -> u8 {
                self.o
            }
            #[doc = " Writes bit to the field"]
            #[inline(always)]
            pub fn bit(self, value: bool) -> &'a mut W<REG> {
                self.w.bits &= !(REG::Ux::ONE << self.o);
                self.w.bits |= (REG::Ux::from(value) & REG::Ux::ONE) << self.o;
                self.w
            }
            #[doc = " Writes `variant` to the field"]
            #[inline(always)]
            pub fn variant(self, variant: FI) -> &'a mut W<REG> {
                self.bit(bool::from(variant))
            }
        }
    };
}
bit_proxy!(BitWriter, BitM);
bit_proxy!(BitWriter1S, Bit1S);
bit_proxy!(BitWriter0C, Bit0C);
bit_proxy!(BitWriter1C, Bit1C);
bit_proxy!(BitWriter0S, Bit0S);
bit_proxy!(BitWriter1T, Bit1T);
bit_proxy!(BitWriter0T, Bit0T);
impl<'a, REG, FI> BitWriter<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = " Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::ONE << self.o;
        self.w
    }
    #[doc = " Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::ONE << self.o);
        self.w
    }
}
impl<'a, REG, FI> BitWriter1S<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = " Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::ONE << self.o;
        self.w
    }
}
impl<'a, REG, FI> BitWriter0C<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = " Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::ONE << self.o);
        self.w
    }
}
impl<'a, REG, FI> BitWriter1C<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = "Clears the field bit by passing one"]
    #[inline(always)]
    pub fn clear_bit_by_one(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::ONE << self.o;
        self.w
    }
}
impl<'a, REG, FI> BitWriter0S<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = "Sets the field bit by passing zero"]
    #[inline(always)]
    pub fn set_bit_by_zero(self) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::ONE << self.o);
        self.w
    }
}
impl<'a, REG, FI> BitWriter1T<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = "Toggle the field bit by passing one"]
    #[inline(always)]
    pub fn toggle_bit(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::ONE << self.o;
        self.w
    }
}
impl<'a, REG, FI> BitWriter0T<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    #[doc = "Toggle the field bit by passing zero"]
    #[inline(always)]
    pub fn toggle_bit(self) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::ONE << self.o);
        self.w
    }
}
#[doc = " This structure provides volatile access to registers."]
#[repr(transparent)]
pub struct Reg<REG: RegisterSpec> {
    register: vcell::VolatileCell<REG::Ux>,
    _marker: marker::PhantomData<REG>,
}
unsafe impl<REG: RegisterSpec> Send for Reg<REG> where REG::Ux: Send {}
impl<REG: RegisterSpec> Reg<REG> {
    #[doc = " Returns the underlying memory address of register."]
    #[doc = ""]
    #[doc = " ```ignore"]
    #[doc = " let reg_ptr = periph.reg.as_ptr();"]
    #[doc = " ```"]
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut REG::Ux {
        self.register.as_ptr()
    }
}
impl<REG: Readable> Reg<REG> {
    #[doc = " Reads the contents of a `Readable` register."]
    #[doc = ""]
    #[doc = " You can read the raw contents of a register by using `bits`:"]
    #[doc = " ```ignore"]
    #[doc = " let bits = periph.reg.read().bits();"]
    #[doc = " ```"]
    #[doc = " or get the content of a particular field of a register:"]
    #[doc = " ```ignore"]
    #[doc = " let reader = periph.reg.read();"]
    #[doc = " let bits = reader.field1().bits();"]
    #[doc = " let flag = reader.field2().bit_is_set();"]
    #[doc = " ```"]
    #[inline(always)]
    pub fn read(&self) -> R<REG> {
        R {
            bits: self.register.get(),
            _reg: marker::PhantomData,
        }
    }
}
impl<REG: Resettable + Writable> Reg<REG> {
    #[doc = " Writes the reset value to `Writable` register."]
    #[doc = ""]
    #[doc = " Resets the register to its initial state."]
    #[inline(always)]
    pub fn reset(&self) {
        self.register.set(REG::RESET_VALUE)
    }
    #[doc = " Writes bits to a `Writable` register."]
    #[doc = ""]
    #[doc = " You can write raw bits into a register:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write(|w| unsafe { w.bits(rawbits) });"]
    #[doc = " ```"]
    #[doc = " or write only the fields you need:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write(|w| w"]
    #[doc = "     .field1().bits(newfield1bits)"]
    #[doc = "     .field2().set_bit()"]
    #[doc = "     .field3().variant(VARIANT)"]
    #[doc = " );"]
    #[doc = " ```"]
    #[doc = " or an alternative way of saying the same:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write(|w| {"]
    #[doc = "     w.field1().bits(newfield1bits);"]
    #[doc = "     w.field2().set_bit();"]
    #[doc = "     w.field3().variant(VARIANT)"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " In the latter case, other fields will be set to their reset value."]
    #[inline(always)]
    pub fn write<F>(&self, f: F) -> REG::Ux
    where
        F: FnOnce(&mut W<REG>) -> &mut W<REG>,
    {
        let value = f(&mut W {
            bits: REG::RESET_VALUE & !REG::ONE_TO_MODIFY_FIELDS_BITMAP
                | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
            _reg: marker::PhantomData,
        })
        .bits;
        self.register.set(value);
        value
    }
    #[doc = " Writes bits to a `Writable` register and produce a value."]
    #[doc = ""]
    #[doc = " You can write raw bits into a register:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write_and(|w| unsafe { w.bits(rawbits); });"]
    #[doc = " ```"]
    #[doc = " or write only the fields you need:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write_and(|w| {"]
    #[doc = "     w.field1().bits(newfield1bits)"]
    #[doc = "         .field2().set_bit()"]
    #[doc = "         .field3().variant(VARIANT);"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " or an alternative way of saying the same:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.write_and(|w| {"]
    #[doc = "     w.field1().bits(newfield1bits);"]
    #[doc = "     w.field2().set_bit();"]
    #[doc = "     w.field3().variant(VARIANT);"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " In the latter case, other fields will be set to their reset value."]
    #[doc = ""]
    #[doc = " Values can be returned from the closure:"]
    #[doc = " ```ignore"]
    #[doc = " let state = periph.reg.write_and(|w| State::set(w.field1()));"]
    #[doc = " ```"]
    #[inline(always)]
    pub fn from_write<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut W<REG>) -> T,
    {
        let mut writer = W {
            bits: REG::RESET_VALUE & !REG::ONE_TO_MODIFY_FIELDS_BITMAP
                | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
            _reg: marker::PhantomData,
        };
        let result = f(&mut writer);
        self.register.set(writer.bits);
        result
    }
}
impl<REG: Writable> Reg<REG> {
    #[doc = " Writes 0 to a `Writable` register."]
    #[doc = ""]
    #[doc = " Similar to `write`, but unused bits will contain 0."]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " Unsafe to use with registers which don't allow to write 0."]
    #[inline(always)]
    pub unsafe fn write_with_zero<F>(&self, f: F) -> REG::Ux
    where
        F: FnOnce(&mut W<REG>) -> &mut W<REG>,
    {
        let value = f(&mut W {
            bits: REG::Ux::ZERO,
            _reg: marker::PhantomData,
        })
        .bits;
        self.register.set(value);
        value
    }
    #[doc = " Writes 0 to a `Writable` register and produces a value."]
    #[doc = ""]
    #[doc = " Similar to `write`, but unused bits will contain 0."]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " Unsafe to use with registers which don't allow to write 0."]
    #[inline(always)]
    pub unsafe fn from_write_with_zero<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut W<REG>) -> T,
    {
        let mut writer = W {
            bits: REG::Ux::ZERO,
            _reg: marker::PhantomData,
        };
        let result = f(&mut writer);
        self.register.set(writer.bits);
        result
    }
}
impl<REG: Readable + Writable> Reg<REG> {
    #[doc = " Modifies the contents of the register by reading and then writing it."]
    #[doc = ""]
    #[doc = " E.g. to do a read-modify-write sequence to change parts of a register:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.modify(|r, w| unsafe { w.bits("]
    #[doc = "    r.bits() | 3"]
    #[doc = " ) });"]
    #[doc = " ```"]
    #[doc = " or"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.modify(|_, w| w"]
    #[doc = "     .field1().bits(newfield1bits)"]
    #[doc = "     .field2().set_bit()"]
    #[doc = "     .field3().variant(VARIANT)"]
    #[doc = " );"]
    #[doc = " ```"]
    #[doc = " or an alternative way of saying the same:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.modify(|_, w| {"]
    #[doc = "     w.field1().bits(newfield1bits);"]
    #[doc = "     w.field2().set_bit();"]
    #[doc = "     w.field3().variant(VARIANT)"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " Other fields will have the value they had before the call to `modify`."]
    #[inline(always)]
    pub fn modify<F>(&self, f: F) -> REG::Ux
    where
        for<'w> F: FnOnce(&R<REG>, &'w mut W<REG>) -> &'w mut W<REG>,
    {
        let bits = self.register.get();
        let value = f(
            &R {
                bits,
                _reg: marker::PhantomData,
            },
            &mut W {
                bits: bits & !REG::ONE_TO_MODIFY_FIELDS_BITMAP | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
                _reg: marker::PhantomData,
            },
        )
        .bits;
        self.register.set(value);
        value
    }
    #[doc = " Modifies the contents of the register by reading and then writing it"]
    #[doc = " and produces a value."]
    #[doc = ""]
    #[doc = " E.g. to do a read-modify-write sequence to change parts of a register:"]
    #[doc = " ```ignore"]
    #[doc = " let bits = periph.reg.modify(|r, w| {"]
    #[doc = "     let new_bits = r.bits() | 3;"]
    #[doc = "     unsafe {"]
    #[doc = "         w.bits(new_bits);"]
    #[doc = "     }"]
    #[doc = ""]
    #[doc = "     new_bits"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " or"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.modify(|_, w| {"]
    #[doc = "     w.field1().bits(newfield1bits)"]
    #[doc = "         .field2().set_bit()"]
    #[doc = "         .field3().variant(VARIANT);"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " or an alternative way of saying the same:"]
    #[doc = " ```ignore"]
    #[doc = " periph.reg.modify(|_, w| {"]
    #[doc = "     w.field1().bits(newfield1bits);"]
    #[doc = "     w.field2().set_bit();"]
    #[doc = "     w.field3().variant(VARIANT);"]
    #[doc = " });"]
    #[doc = " ```"]
    #[doc = " Other fields will have the value they had before the call to `modify`."]
    #[inline(always)]
    pub fn from_modify<F, T>(&self, f: F) -> T
    where
        for<'w> F: FnOnce(&R<REG>, &'w mut W<REG>) -> T,
    {
        let bits = self.register.get();
        let mut writer = W {
            bits: bits & !REG::ONE_TO_MODIFY_FIELDS_BITMAP | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
            _reg: marker::PhantomData,
        };
        let result = f(
            &R {
                bits,
                _reg: marker::PhantomData,
            },
            &mut writer,
        );
        self.register.set(writer.bits);
        result
    }
}
impl<REG: Readable> core::fmt::Debug for crate::generic::Reg<REG>
where
    R<REG>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.read(), f)
    }
}
