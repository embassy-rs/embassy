#[doc = "Register `TRIGFIL_PRSC` reader"]
pub type R = crate::R<TrigfilPrscSpec>;
#[doc = "Register `TRIGFIL_PRSC` writer"]
pub type W = crate::W<TrigfilPrscSpec>;
#[doc = "Filter Prescaller Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FiltScaleVal {
    #[doc = "0: Bypass the clock"]
    Val0 = 0,
    #[doc = "1: Divide 2"]
    Val1 = 1,
    #[doc = "2: Divide 4"]
    Val2 = 2,
    #[doc = "3: Divide 8"]
    Val3 = 3,
}
impl From<FiltScaleVal> for u8 {
    #[inline(always)]
    fn from(variant: FiltScaleVal) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for FiltScaleVal {
    type Ux = u8;
}
impl crate::IsEnum for FiltScaleVal {}
#[doc = "Field `FILT_SCALE_VAL` reader - Filter Prescaller Value"]
pub type FiltScaleValR = crate::FieldReader<FiltScaleVal>;
impl FiltScaleValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FiltScaleVal {
        match self.bits {
            0 => FiltScaleVal::Val0,
            1 => FiltScaleVal::Val1,
            2 => FiltScaleVal::Val2,
            3 => FiltScaleVal::Val3,
            _ => unreachable!(),
        }
    }
    #[doc = "Bypass the clock"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == FiltScaleVal::Val0
    }
    #[doc = "Divide 2"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == FiltScaleVal::Val1
    }
    #[doc = "Divide 4"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == FiltScaleVal::Val2
    }
    #[doc = "Divide 8"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == FiltScaleVal::Val3
    }
}
#[doc = "Field `FILT_SCALE_VAL` writer - Filter Prescaller Value"]
pub type FiltScaleValW<'a, REG> = crate::FieldWriter<'a, REG, 2, FiltScaleVal, crate::Safe>;
impl<'a, REG> FiltScaleValW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Bypass the clock"]
    #[inline(always)]
    pub fn val0(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleVal::Val0)
    }
    #[doc = "Divide 2"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleVal::Val1)
    }
    #[doc = "Divide 4"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleVal::Val2)
    }
    #[doc = "Divide 8"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleVal::Val3)
    }
}
#[doc = "Enable trigger filter prescaller\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FiltScaleEn {
    #[doc = "0: Disable prescaller"]
    Val2 = 0,
    #[doc = "1: Enabled prescaller"]
    Val1 = 1,
}
impl From<FiltScaleEn> for bool {
    #[inline(always)]
    fn from(variant: FiltScaleEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILT_SCALE_EN` reader - Enable trigger filter prescaller"]
pub type FiltScaleEnR = crate::BitReader<FiltScaleEn>;
impl FiltScaleEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FiltScaleEn {
        match self.bits {
            false => FiltScaleEn::Val2,
            true => FiltScaleEn::Val1,
        }
    }
    #[doc = "Disable prescaller"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == FiltScaleEn::Val2
    }
    #[doc = "Enabled prescaller"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == FiltScaleEn::Val1
    }
}
#[doc = "Field `FILT_SCALE_EN` writer - Enable trigger filter prescaller"]
pub type FiltScaleEnW<'a, REG> = crate::BitWriter<'a, REG, FiltScaleEn>;
impl<'a, REG> FiltScaleEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable prescaller"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleEn::Val2)
    }
    #[doc = "Enabled prescaller"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(FiltScaleEn::Val1)
    }
}
impl R {
    #[doc = "Bits 0:1 - Filter Prescaller Value"]
    #[inline(always)]
    pub fn filt_scale_val(&self) -> FiltScaleValR {
        FiltScaleValR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 31 - Enable trigger filter prescaller"]
    #[inline(always)]
    pub fn filt_scale_en(&self) -> FiltScaleEnR {
        FiltScaleEnR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Filter Prescaller Value"]
    #[inline(always)]
    pub fn filt_scale_val(&mut self) -> FiltScaleValW<TrigfilPrscSpec> {
        FiltScaleValW::new(self, 0)
    }
    #[doc = "Bit 31 - Enable trigger filter prescaller"]
    #[inline(always)]
    pub fn filt_scale_en(&mut self) -> FiltScaleEnW<TrigfilPrscSpec> {
        FiltScaleEnW::new(self, 31)
    }
}
#[doc = "Trigger filter prescaller\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil_prsc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigfil_prsc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrigfilPrscSpec;
impl crate::RegisterSpec for TrigfilPrscSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trigfil_prsc::R`](R) reader structure"]
impl crate::Readable for TrigfilPrscSpec {}
#[doc = "`write(|w| ..)` method takes [`trigfil_prsc::W`](W) writer structure"]
impl crate::Writable for TrigfilPrscSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TRIGFIL_PRSC to value 0"]
impl crate::Resettable for TrigfilPrscSpec {}
