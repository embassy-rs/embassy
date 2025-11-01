#[doc = "Register `CKSTAT` reader"]
pub type R = crate::R<CkstatSpec>;
#[doc = "Register `CKSTAT` writer"]
pub type W = crate::W<CkstatSpec>;
#[doc = "Low Power Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ckmode {
    #[doc = "0: Core clock is on"]
    Ckmode0000 = 0,
    #[doc = "1: Core clock is off"]
    Ckmode0001 = 1,
    #[doc = "15: Core, platform, and peripheral clocks are off, and core enters Low-Power mode"]
    Ckmode1111 = 15,
}
impl From<Ckmode> for u8 {
    #[inline(always)]
    fn from(variant: Ckmode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ckmode {
    type Ux = u8;
}
impl crate::IsEnum for Ckmode {}
#[doc = "Field `CKMODE` reader - Low Power Status"]
pub type CkmodeR = crate::FieldReader<Ckmode>;
impl CkmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ckmode> {
        match self.bits {
            0 => Some(Ckmode::Ckmode0000),
            1 => Some(Ckmode::Ckmode0001),
            15 => Some(Ckmode::Ckmode1111),
            _ => None,
        }
    }
    #[doc = "Core clock is on"]
    #[inline(always)]
    pub fn is_ckmode0000(&self) -> bool {
        *self == Ckmode::Ckmode0000
    }
    #[doc = "Core clock is off"]
    #[inline(always)]
    pub fn is_ckmode0001(&self) -> bool {
        *self == Ckmode::Ckmode0001
    }
    #[doc = "Core, platform, and peripheral clocks are off, and core enters Low-Power mode"]
    #[inline(always)]
    pub fn is_ckmode1111(&self) -> bool {
        *self == Ckmode::Ckmode1111
    }
}
#[doc = "Field `WAKEUP` reader - Wake-up Source"]
pub type WakeupR = crate::FieldReader;
#[doc = "Clock Status Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valid {
    #[doc = "0: Core clock not gated"]
    Disabled = 0,
    #[doc = "1: Core clock was gated due to Low-Power mode entry"]
    Enabled = 1,
}
impl From<Valid> for bool {
    #[inline(always)]
    fn from(variant: Valid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VALID` reader - Clock Status Valid"]
pub type ValidR = crate::BitReader<Valid>;
impl ValidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Valid {
        match self.bits {
            false => Valid::Disabled,
            true => Valid::Enabled,
        }
    }
    #[doc = "Core clock not gated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Valid::Disabled
    }
    #[doc = "Core clock was gated due to Low-Power mode entry"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Valid::Enabled
    }
}
#[doc = "Field `VALID` writer - Clock Status Valid"]
pub type ValidW<'a, REG> = crate::BitWriter1C<'a, REG, Valid>;
impl<'a, REG> ValidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Core clock not gated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Valid::Disabled)
    }
    #[doc = "Core clock was gated due to Low-Power mode entry"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Valid::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:3 - Low Power Status"]
    #[inline(always)]
    pub fn ckmode(&self) -> CkmodeR {
        CkmodeR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 8:15 - Wake-up Source"]
    #[inline(always)]
    pub fn wakeup(&self) -> WakeupR {
        WakeupR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bit 31 - Clock Status Valid"]
    #[inline(always)]
    pub fn valid(&self) -> ValidR {
        ValidR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 31 - Clock Status Valid"]
    #[inline(always)]
    pub fn valid(&mut self) -> ValidW<CkstatSpec> {
        ValidW::new(self, 31)
    }
}
#[doc = "Clock Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ckstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ckstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CkstatSpec;
impl crate::RegisterSpec for CkstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ckstat::R`](R) reader structure"]
impl crate::Readable for CkstatSpec {}
#[doc = "`write(|w| ..)` method takes [`ckstat::W`](W) writer structure"]
impl crate::Writable for CkstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8000_0000;
}
#[doc = "`reset()` method sets CKSTAT to value 0"]
impl crate::Resettable for CkstatSpec {}
