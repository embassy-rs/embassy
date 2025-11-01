#[doc = "Register `ROSCCSR` reader"]
pub type R = crate::R<RosccsrSpec>;
#[doc = "Register `ROSCCSR` writer"]
pub type W = crate::W<RosccsrSpec>;
#[doc = "Lock Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lk {
    #[doc = "0: Control Status Register can be written"]
    WriteEnabled = 0,
    #[doc = "1: Control Status Register cannot be written"]
    WriteDisabled = 1,
}
impl From<Lk> for bool {
    #[inline(always)]
    fn from(variant: Lk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LK` reader - Lock Register"]
pub type LkR = crate::BitReader<Lk>;
impl LkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lk {
        match self.bits {
            false => Lk::WriteEnabled,
            true => Lk::WriteDisabled,
        }
    }
    #[doc = "Control Status Register can be written"]
    #[inline(always)]
    pub fn is_write_enabled(&self) -> bool {
        *self == Lk::WriteEnabled
    }
    #[doc = "Control Status Register cannot be written"]
    #[inline(always)]
    pub fn is_write_disabled(&self) -> bool {
        *self == Lk::WriteDisabled
    }
}
#[doc = "Field `LK` writer - Lock Register"]
pub type LkW<'a, REG> = crate::BitWriter<'a, REG, Lk>;
impl<'a, REG> LkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Control Status Register can be written"]
    #[inline(always)]
    pub fn write_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteEnabled)
    }
    #[doc = "Control Status Register cannot be written"]
    #[inline(always)]
    pub fn write_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteDisabled)
    }
}
#[doc = "ROSC Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roscvld {
    #[doc = "0: ROSC is not enabled or clock is not valid"]
    DisabledOrNotValid = 0,
    #[doc = "1: ROSC is enabled and output clock is valid"]
    EnabledAndValid = 1,
}
impl From<Roscvld> for bool {
    #[inline(always)]
    fn from(variant: Roscvld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROSCVLD` reader - ROSC Valid"]
pub type RoscvldR = crate::BitReader<Roscvld>;
impl RoscvldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roscvld {
        match self.bits {
            false => Roscvld::DisabledOrNotValid,
            true => Roscvld::EnabledAndValid,
        }
    }
    #[doc = "ROSC is not enabled or clock is not valid"]
    #[inline(always)]
    pub fn is_disabled_or_not_valid(&self) -> bool {
        *self == Roscvld::DisabledOrNotValid
    }
    #[doc = "ROSC is enabled and output clock is valid"]
    #[inline(always)]
    pub fn is_enabled_and_valid(&self) -> bool {
        *self == Roscvld::EnabledAndValid
    }
}
#[doc = "ROSC Selected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roscsel {
    #[doc = "0: ROSC is not the system clock source"]
    NotRosc = 0,
    #[doc = "1: ROSC is the system clock source"]
    Rosc = 1,
}
impl From<Roscsel> for bool {
    #[inline(always)]
    fn from(variant: Roscsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROSCSEL` reader - ROSC Selected"]
pub type RoscselR = crate::BitReader<Roscsel>;
impl RoscselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roscsel {
        match self.bits {
            false => Roscsel::NotRosc,
            true => Roscsel::Rosc,
        }
    }
    #[doc = "ROSC is not the system clock source"]
    #[inline(always)]
    pub fn is_not_rosc(&self) -> bool {
        *self == Roscsel::NotRosc
    }
    #[doc = "ROSC is the system clock source"]
    #[inline(always)]
    pub fn is_rosc(&self) -> bool {
        *self == Roscsel::Rosc
    }
}
#[doc = "ROSC Clock Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roscerr {
    #[doc = "0: ROSC Clock has not detected an error"]
    DisabledOrNoError = 0,
    #[doc = "1: ROSC Clock has detected an error"]
    EnabledAndError = 1,
}
impl From<Roscerr> for bool {
    #[inline(always)]
    fn from(variant: Roscerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROSCERR` reader - ROSC Clock Error"]
pub type RoscerrR = crate::BitReader<Roscerr>;
impl RoscerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roscerr {
        match self.bits {
            false => Roscerr::DisabledOrNoError,
            true => Roscerr::EnabledAndError,
        }
    }
    #[doc = "ROSC Clock has not detected an error"]
    #[inline(always)]
    pub fn is_disabled_or_no_error(&self) -> bool {
        *self == Roscerr::DisabledOrNoError
    }
    #[doc = "ROSC Clock has detected an error"]
    #[inline(always)]
    pub fn is_enabled_and_error(&self) -> bool {
        *self == Roscerr::EnabledAndError
    }
}
#[doc = "Field `ROSCERR` writer - ROSC Clock Error"]
pub type RoscerrW<'a, REG> = crate::BitWriter1C<'a, REG, Roscerr>;
impl<'a, REG> RoscerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ROSC Clock has not detected an error"]
    #[inline(always)]
    pub fn disabled_or_no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Roscerr::DisabledOrNoError)
    }
    #[doc = "ROSC Clock has detected an error"]
    #[inline(always)]
    pub fn enabled_and_error(self) -> &'a mut crate::W<REG> {
        self.variant(Roscerr::EnabledAndError)
    }
}
impl R {
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - ROSC Valid"]
    #[inline(always)]
    pub fn roscvld(&self) -> RoscvldR {
        RoscvldR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - ROSC Selected"]
    #[inline(always)]
    pub fn roscsel(&self) -> RoscselR {
        RoscselR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - ROSC Clock Error"]
    #[inline(always)]
    pub fn roscerr(&self) -> RoscerrR {
        RoscerrR::new(((self.bits >> 26) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<RosccsrSpec> {
        LkW::new(self, 23)
    }
    #[doc = "Bit 26 - ROSC Clock Error"]
    #[inline(always)]
    pub fn roscerr(&mut self) -> RoscerrW<RosccsrSpec> {
        RoscerrW::new(self, 26)
    }
}
#[doc = "ROSC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rosccsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rosccsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RosccsrSpec;
impl crate::RegisterSpec for RosccsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rosccsr::R`](R) reader structure"]
impl crate::Readable for RosccsrSpec {}
#[doc = "`write(|w| ..)` method takes [`rosccsr::W`](W) writer structure"]
impl crate::Writable for RosccsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0400_0000;
}
#[doc = "`reset()` method sets ROSCCSR to value 0"]
impl crate::Resettable for RosccsrSpec {}
