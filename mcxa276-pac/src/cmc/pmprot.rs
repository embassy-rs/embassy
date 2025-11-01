#[doc = "Register `PMPROT` reader"]
pub type R = crate::R<PmprotSpec>;
#[doc = "Register `PMPROT` writer"]
pub type W = crate::W<PmprotSpec>;
#[doc = "Low-Power Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Lpmode {
    #[doc = "0: Not allowed"]
    Disabled = 0,
    #[doc = "1: Allowed"]
    En = 1,
    #[doc = "2: Allowed"]
    En1 = 2,
    #[doc = "3: Allowed"]
    En2 = 3,
    #[doc = "4: Allowed"]
    En3 = 4,
    #[doc = "5: Allowed"]
    En4 = 5,
    #[doc = "6: Allowed"]
    En5 = 6,
    #[doc = "7: Allowed"]
    En6 = 7,
    #[doc = "8: Allowed"]
    En7 = 8,
    #[doc = "9: Allowed"]
    En8 = 9,
    #[doc = "10: Allowed"]
    En9 = 10,
    #[doc = "11: Allowed"]
    En10 = 11,
    #[doc = "12: Allowed"]
    En11 = 12,
    #[doc = "13: Allowed"]
    En12 = 13,
    #[doc = "14: Allowed"]
    En13 = 14,
    #[doc = "15: Allowed"]
    En14 = 15,
}
impl From<Lpmode> for u8 {
    #[inline(always)]
    fn from(variant: Lpmode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Lpmode {
    type Ux = u8;
}
impl crate::IsEnum for Lpmode {}
#[doc = "Field `LPMODE` reader - Low-Power Mode"]
pub type LpmodeR = crate::FieldReader<Lpmode>;
impl LpmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpmode {
        match self.bits {
            0 => Lpmode::Disabled,
            1 => Lpmode::En,
            2 => Lpmode::En1,
            3 => Lpmode::En2,
            4 => Lpmode::En3,
            5 => Lpmode::En4,
            6 => Lpmode::En5,
            7 => Lpmode::En6,
            8 => Lpmode::En7,
            9 => Lpmode::En8,
            10 => Lpmode::En9,
            11 => Lpmode::En10,
            12 => Lpmode::En11,
            13 => Lpmode::En12,
            14 => Lpmode::En13,
            15 => Lpmode::En14,
            _ => unreachable!(),
        }
    }
    #[doc = "Not allowed"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpmode::Disabled
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en(&self) -> bool {
        *self == Lpmode::En
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en1(&self) -> bool {
        *self == Lpmode::En1
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en2(&self) -> bool {
        *self == Lpmode::En2
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en3(&self) -> bool {
        *self == Lpmode::En3
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en4(&self) -> bool {
        *self == Lpmode::En4
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en5(&self) -> bool {
        *self == Lpmode::En5
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en6(&self) -> bool {
        *self == Lpmode::En6
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en7(&self) -> bool {
        *self == Lpmode::En7
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en8(&self) -> bool {
        *self == Lpmode::En8
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en9(&self) -> bool {
        *self == Lpmode::En9
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en10(&self) -> bool {
        *self == Lpmode::En10
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en11(&self) -> bool {
        *self == Lpmode::En11
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en12(&self) -> bool {
        *self == Lpmode::En12
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en13(&self) -> bool {
        *self == Lpmode::En13
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn is_en14(&self) -> bool {
        *self == Lpmode::En14
    }
}
#[doc = "Field `LPMODE` writer - Low-Power Mode"]
pub type LpmodeW<'a, REG> = crate::FieldWriter<'a, REG, 4, Lpmode, crate::Safe>;
impl<'a, REG> LpmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Not allowed"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::Disabled)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en1(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En1)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en2(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En2)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en3(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En3)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en4(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En4)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en5(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En5)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en6(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En6)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en7(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En7)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en8(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En8)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en9(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En9)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en10(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En10)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en11(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En11)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en12(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En12)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en13(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En13)
    }
    #[doc = "Allowed"]
    #[inline(always)]
    pub fn en14(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::En14)
    }
}
#[doc = "Lock Register\n\nValue on reset: 0"]
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
#[doc = "Field `LOCK` reader - Lock Register"]
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
#[doc = "Field `LOCK` writer - Lock Register"]
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
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&self) -> LpmodeR {
        LpmodeR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 31 - Lock Register"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&mut self) -> LpmodeW<PmprotSpec> {
        LpmodeW::new(self, 0)
    }
    #[doc = "Bit 31 - Lock Register"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<PmprotSpec> {
        LockW::new(self, 31)
    }
}
#[doc = "Power Mode Protection\n\nYou can [`read`](crate::Reg::read) this register and get [`pmprot::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pmprot::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PmprotSpec;
impl crate::RegisterSpec for PmprotSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pmprot::R`](R) reader structure"]
impl crate::Readable for PmprotSpec {}
#[doc = "`write(|w| ..)` method takes [`pmprot::W`](W) writer structure"]
impl crate::Writable for PmprotSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PMPROT to value 0"]
impl crate::Resettable for PmprotSpec {}
