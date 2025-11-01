#[doc = "Register `CKCTRL` reader"]
pub type R = crate::R<CkctrlSpec>;
#[doc = "Register `CKCTRL` writer"]
pub type W = crate::W<CkctrlSpec>;
#[doc = "Clocking Mode\n\nValue on reset: 0"]
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
#[doc = "Field `CKMODE` reader - Clocking Mode"]
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
#[doc = "Field `CKMODE` writer - Clocking Mode"]
pub type CkmodeW<'a, REG> = crate::FieldWriter<'a, REG, 4, Ckmode>;
impl<'a, REG> CkmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Core clock is on"]
    #[inline(always)]
    pub fn ckmode0000(self) -> &'a mut crate::W<REG> {
        self.variant(Ckmode::Ckmode0000)
    }
    #[doc = "Core clock is off"]
    #[inline(always)]
    pub fn ckmode0001(self) -> &'a mut crate::W<REG> {
        self.variant(Ckmode::Ckmode0001)
    }
    #[doc = "Core, platform, and peripheral clocks are off, and core enters Low-Power mode"]
    #[inline(always)]
    pub fn ckmode1111(self) -> &'a mut crate::W<REG> {
        self.variant(Ckmode::Ckmode1111)
    }
}
#[doc = "Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: Allowed"]
    Disabled = 0,
    #[doc = "1: Blocked"]
    Enabled = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - Lock"]
pub type LockR = crate::BitReader<Lock>;
impl LockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lock {
        match self.bits {
            false => Lock::Disabled,
            true => Lock::Enabled,
        }
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lock::Disabled
    }
    #[doc = "Blocked"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lock::Enabled
    }
}
#[doc = "Field `LOCK` writer - Lock"]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Disabled)
    }
    #[doc = "Blocked"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:3 - Clocking Mode"]
    #[inline(always)]
    pub fn ckmode(&self) -> CkmodeR {
        CkmodeR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 31 - Lock"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:3 - Clocking Mode"]
    #[inline(always)]
    pub fn ckmode(&mut self) -> CkmodeW<CkctrlSpec> {
        CkmodeW::new(self, 0)
    }
    #[doc = "Bit 31 - Lock"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<CkctrlSpec> {
        LockW::new(self, 31)
    }
}
#[doc = "Clock Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ckctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ckctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CkctrlSpec;
impl crate::RegisterSpec for CkctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ckctrl::R`](R) reader structure"]
impl crate::Readable for CkctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ckctrl::W`](W) writer structure"]
impl crate::Writable for CkctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CKCTRL to value 0"]
impl crate::Resettable for CkctrlSpec {}
