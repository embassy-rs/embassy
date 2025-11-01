#[doc = "Register `EMR` reader"]
pub type R = crate::R<EmrSpec>;
#[doc = "Register `EMR` writer"]
pub type W = crate::W<EmrSpec>;
#[doc = "External Match 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Em0 {
    #[doc = "0: Low"]
    Clear = 0,
    #[doc = "1: High"]
    Set = 1,
}
impl From<Em0> for bool {
    #[inline(always)]
    fn from(variant: Em0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EM0` reader - External Match 0"]
pub type Em0R = crate::BitReader<Em0>;
impl Em0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Em0 {
        match self.bits {
            false => Em0::Clear,
            true => Em0::Set,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Em0::Clear
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Em0::Set
    }
}
#[doc = "Field `EM0` writer - External Match 0"]
pub type Em0W<'a, REG> = crate::BitWriter<'a, REG, Em0>;
impl<'a, REG> Em0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Em0::Clear)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Em0::Set)
    }
}
#[doc = "External Match 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Em1 {
    #[doc = "0: Low"]
    Clear = 0,
    #[doc = "1: High"]
    Set = 1,
}
impl From<Em1> for bool {
    #[inline(always)]
    fn from(variant: Em1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EM1` reader - External Match 1"]
pub type Em1R = crate::BitReader<Em1>;
impl Em1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Em1 {
        match self.bits {
            false => Em1::Clear,
            true => Em1::Set,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Em1::Clear
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Em1::Set
    }
}
#[doc = "Field `EM1` writer - External Match 1"]
pub type Em1W<'a, REG> = crate::BitWriter<'a, REG, Em1>;
impl<'a, REG> Em1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Em1::Clear)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Em1::Set)
    }
}
#[doc = "External Match 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Em2 {
    #[doc = "0: Low"]
    Clear = 0,
    #[doc = "1: High"]
    Set = 1,
}
impl From<Em2> for bool {
    #[inline(always)]
    fn from(variant: Em2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EM2` reader - External Match 2"]
pub type Em2R = crate::BitReader<Em2>;
impl Em2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Em2 {
        match self.bits {
            false => Em2::Clear,
            true => Em2::Set,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Em2::Clear
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Em2::Set
    }
}
#[doc = "Field `EM2` writer - External Match 2"]
pub type Em2W<'a, REG> = crate::BitWriter<'a, REG, Em2>;
impl<'a, REG> Em2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Em2::Clear)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Em2::Set)
    }
}
#[doc = "External Match 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Em3 {
    #[doc = "0: Low"]
    Clear = 0,
    #[doc = "1: High"]
    Set = 1,
}
impl From<Em3> for bool {
    #[inline(always)]
    fn from(variant: Em3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EM3` reader - External Match 3"]
pub type Em3R = crate::BitReader<Em3>;
impl Em3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Em3 {
        match self.bits {
            false => Em3::Clear,
            true => Em3::Set,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Em3::Clear
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Em3::Set
    }
}
#[doc = "Field `EM3` writer - External Match 3"]
pub type Em3W<'a, REG> = crate::BitWriter<'a, REG, Em3>;
impl<'a, REG> Em3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Em3::Clear)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Em3::Set)
    }
}
#[doc = "External Match Control 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Emc0 {
    #[doc = "0: Does nothing"]
    DoNothing = 0,
    #[doc = "1: Goes low"]
    Clear = 1,
    #[doc = "2: Goes high"]
    Set = 2,
    #[doc = "3: Toggles"]
    Toggle = 3,
}
impl From<Emc0> for u8 {
    #[inline(always)]
    fn from(variant: Emc0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Emc0 {
    type Ux = u8;
}
impl crate::IsEnum for Emc0 {}
#[doc = "Field `EMC0` reader - External Match Control 0"]
pub type Emc0R = crate::FieldReader<Emc0>;
impl Emc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emc0 {
        match self.bits {
            0 => Emc0::DoNothing,
            1 => Emc0::Clear,
            2 => Emc0::Set,
            3 => Emc0::Toggle,
            _ => unreachable!(),
        }
    }
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn is_do_nothing(&self) -> bool {
        *self == Emc0::DoNothing
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Emc0::Clear
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Emc0::Set
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn is_toggle(&self) -> bool {
        *self == Emc0::Toggle
    }
}
#[doc = "Field `EMC0` writer - External Match Control 0"]
pub type Emc0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Emc0, crate::Safe>;
impl<'a, REG> Emc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn do_nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Emc0::DoNothing)
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Emc0::Clear)
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Emc0::Set)
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn toggle(self) -> &'a mut crate::W<REG> {
        self.variant(Emc0::Toggle)
    }
}
#[doc = "External Match Control 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Emc1 {
    #[doc = "0: Does nothing"]
    DoNothing = 0,
    #[doc = "1: Goes low"]
    Clear = 1,
    #[doc = "2: Goes high"]
    Set = 2,
    #[doc = "3: Toggles"]
    Toggle = 3,
}
impl From<Emc1> for u8 {
    #[inline(always)]
    fn from(variant: Emc1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Emc1 {
    type Ux = u8;
}
impl crate::IsEnum for Emc1 {}
#[doc = "Field `EMC1` reader - External Match Control 1"]
pub type Emc1R = crate::FieldReader<Emc1>;
impl Emc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emc1 {
        match self.bits {
            0 => Emc1::DoNothing,
            1 => Emc1::Clear,
            2 => Emc1::Set,
            3 => Emc1::Toggle,
            _ => unreachable!(),
        }
    }
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn is_do_nothing(&self) -> bool {
        *self == Emc1::DoNothing
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Emc1::Clear
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Emc1::Set
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn is_toggle(&self) -> bool {
        *self == Emc1::Toggle
    }
}
#[doc = "Field `EMC1` writer - External Match Control 1"]
pub type Emc1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Emc1, crate::Safe>;
impl<'a, REG> Emc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn do_nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Emc1::DoNothing)
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Emc1::Clear)
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Emc1::Set)
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn toggle(self) -> &'a mut crate::W<REG> {
        self.variant(Emc1::Toggle)
    }
}
#[doc = "External Match Control 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Emc2 {
    #[doc = "0: Does nothing"]
    DoNothing = 0,
    #[doc = "1: Goes low"]
    Clear = 1,
    #[doc = "2: Goes high"]
    Set = 2,
    #[doc = "3: Toggles"]
    Toggle = 3,
}
impl From<Emc2> for u8 {
    #[inline(always)]
    fn from(variant: Emc2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Emc2 {
    type Ux = u8;
}
impl crate::IsEnum for Emc2 {}
#[doc = "Field `EMC2` reader - External Match Control 2"]
pub type Emc2R = crate::FieldReader<Emc2>;
impl Emc2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emc2 {
        match self.bits {
            0 => Emc2::DoNothing,
            1 => Emc2::Clear,
            2 => Emc2::Set,
            3 => Emc2::Toggle,
            _ => unreachable!(),
        }
    }
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn is_do_nothing(&self) -> bool {
        *self == Emc2::DoNothing
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Emc2::Clear
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Emc2::Set
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn is_toggle(&self) -> bool {
        *self == Emc2::Toggle
    }
}
#[doc = "Field `EMC2` writer - External Match Control 2"]
pub type Emc2W<'a, REG> = crate::FieldWriter<'a, REG, 2, Emc2, crate::Safe>;
impl<'a, REG> Emc2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn do_nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Emc2::DoNothing)
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Emc2::Clear)
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Emc2::Set)
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn toggle(self) -> &'a mut crate::W<REG> {
        self.variant(Emc2::Toggle)
    }
}
#[doc = "External Match Control 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Emc3 {
    #[doc = "0: Does nothing"]
    DoNothing = 0,
    #[doc = "1: Goes low"]
    Clear = 1,
    #[doc = "2: Goes high"]
    Set = 2,
    #[doc = "3: Toggles"]
    Toggle = 3,
}
impl From<Emc3> for u8 {
    #[inline(always)]
    fn from(variant: Emc3) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Emc3 {
    type Ux = u8;
}
impl crate::IsEnum for Emc3 {}
#[doc = "Field `EMC3` reader - External Match Control 3"]
pub type Emc3R = crate::FieldReader<Emc3>;
impl Emc3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emc3 {
        match self.bits {
            0 => Emc3::DoNothing,
            1 => Emc3::Clear,
            2 => Emc3::Set,
            3 => Emc3::Toggle,
            _ => unreachable!(),
        }
    }
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn is_do_nothing(&self) -> bool {
        *self == Emc3::DoNothing
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Emc3::Clear
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Emc3::Set
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn is_toggle(&self) -> bool {
        *self == Emc3::Toggle
    }
}
#[doc = "Field `EMC3` writer - External Match Control 3"]
pub type Emc3W<'a, REG> = crate::FieldWriter<'a, REG, 2, Emc3, crate::Safe>;
impl<'a, REG> Emc3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn do_nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Emc3::DoNothing)
    }
    #[doc = "Goes low"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Emc3::Clear)
    }
    #[doc = "Goes high"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Emc3::Set)
    }
    #[doc = "Toggles"]
    #[inline(always)]
    pub fn toggle(self) -> &'a mut crate::W<REG> {
        self.variant(Emc3::Toggle)
    }
}
impl R {
    #[doc = "Bit 0 - External Match 0"]
    #[inline(always)]
    pub fn em0(&self) -> Em0R {
        Em0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - External Match 1"]
    #[inline(always)]
    pub fn em1(&self) -> Em1R {
        Em1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - External Match 2"]
    #[inline(always)]
    pub fn em2(&self) -> Em2R {
        Em2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - External Match 3"]
    #[inline(always)]
    pub fn em3(&self) -> Em3R {
        Em3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:5 - External Match Control 0"]
    #[inline(always)]
    pub fn emc0(&self) -> Emc0R {
        Emc0R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - External Match Control 1"]
    #[inline(always)]
    pub fn emc1(&self) -> Emc1R {
        Emc1R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - External Match Control 2"]
    #[inline(always)]
    pub fn emc2(&self) -> Emc2R {
        Emc2R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - External Match Control 3"]
    #[inline(always)]
    pub fn emc3(&self) -> Emc3R {
        Emc3R::new(((self.bits >> 10) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - External Match 0"]
    #[inline(always)]
    pub fn em0(&mut self) -> Em0W<EmrSpec> {
        Em0W::new(self, 0)
    }
    #[doc = "Bit 1 - External Match 1"]
    #[inline(always)]
    pub fn em1(&mut self) -> Em1W<EmrSpec> {
        Em1W::new(self, 1)
    }
    #[doc = "Bit 2 - External Match 2"]
    #[inline(always)]
    pub fn em2(&mut self) -> Em2W<EmrSpec> {
        Em2W::new(self, 2)
    }
    #[doc = "Bit 3 - External Match 3"]
    #[inline(always)]
    pub fn em3(&mut self) -> Em3W<EmrSpec> {
        Em3W::new(self, 3)
    }
    #[doc = "Bits 4:5 - External Match Control 0"]
    #[inline(always)]
    pub fn emc0(&mut self) -> Emc0W<EmrSpec> {
        Emc0W::new(self, 4)
    }
    #[doc = "Bits 6:7 - External Match Control 1"]
    #[inline(always)]
    pub fn emc1(&mut self) -> Emc1W<EmrSpec> {
        Emc1W::new(self, 6)
    }
    #[doc = "Bits 8:9 - External Match Control 2"]
    #[inline(always)]
    pub fn emc2(&mut self) -> Emc2W<EmrSpec> {
        Emc2W::new(self, 8)
    }
    #[doc = "Bits 10:11 - External Match Control 3"]
    #[inline(always)]
    pub fn emc3(&mut self) -> Emc3W<EmrSpec> {
        Emc3W::new(self, 10)
    }
}
#[doc = "External Match\n\nYou can [`read`](crate::Reg::read) this register and get [`emr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EmrSpec;
impl crate::RegisterSpec for EmrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`emr::R`](R) reader structure"]
impl crate::Readable for EmrSpec {}
#[doc = "`write(|w| ..)` method takes [`emr::W`](W) writer structure"]
impl crate::Writable for EmrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EMR to value 0"]
impl crate::Resettable for EmrSpec {}
