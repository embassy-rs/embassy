#[doc = "Register `TCR` reader"]
pub type R = crate::R<TcrSpec>;
#[doc = "Register `TCR` writer"]
pub type W = crate::W<TcrSpec>;
#[doc = "Counter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cen {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Cen> for bool {
    #[inline(always)]
    fn from(variant: Cen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CEN` reader - Counter Enable"]
pub type CenR = crate::BitReader<Cen>;
impl CenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cen {
        match self.bits {
            false => Cen::Disabled,
            true => Cen::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cen::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cen::Enabled
    }
}
#[doc = "Field `CEN` writer - Counter Enable"]
pub type CenW<'a, REG> = crate::BitWriter<'a, REG, Cen>;
impl<'a, REG> CenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cen::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cen::Enabled)
    }
}
#[doc = "Counter Reset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crst {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Crst> for bool {
    #[inline(always)]
    fn from(variant: Crst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRST` reader - Counter Reset Enable"]
pub type CrstR = crate::BitReader<Crst>;
impl CrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crst {
        match self.bits {
            false => Crst::Disabled,
            true => Crst::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Crst::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Crst::Enabled
    }
}
#[doc = "Field `CRST` writer - Counter Reset Enable"]
pub type CrstW<'a, REG> = crate::BitWriter<'a, REG, Crst>;
impl<'a, REG> CrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Crst::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Crst::Enabled)
    }
}
#[doc = "Allow Global Count Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agcen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Agcen> for bool {
    #[inline(always)]
    fn from(variant: Agcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AGCEN` reader - Allow Global Count Enable"]
pub type AgcenR = crate::BitReader<Agcen>;
impl AgcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Agcen {
        match self.bits {
            false => Agcen::Disable,
            true => Agcen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Agcen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Agcen::Enable
    }
}
#[doc = "Field `AGCEN` writer - Allow Global Count Enable"]
pub type AgcenW<'a, REG> = crate::BitWriter<'a, REG, Agcen>;
impl<'a, REG> AgcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Agcen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Agcen::Enable)
    }
}
#[doc = "Allow Trigger Count Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Atcen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Atcen> for bool {
    #[inline(always)]
    fn from(variant: Atcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ATCEN` reader - Allow Trigger Count Enable"]
pub type AtcenR = crate::BitReader<Atcen>;
impl AtcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Atcen {
        match self.bits {
            false => Atcen::Disable,
            true => Atcen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Atcen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Atcen::Enable
    }
}
#[doc = "Field `ATCEN` writer - Allow Trigger Count Enable"]
pub type AtcenW<'a, REG> = crate::BitWriter<'a, REG, Atcen>;
impl<'a, REG> AtcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Atcen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Atcen::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Counter Enable"]
    #[inline(always)]
    pub fn cen(&self) -> CenR {
        CenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Counter Reset Enable"]
    #[inline(always)]
    pub fn crst(&self) -> CrstR {
        CrstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - Allow Global Count Enable"]
    #[inline(always)]
    pub fn agcen(&self) -> AgcenR {
        AgcenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Allow Trigger Count Enable"]
    #[inline(always)]
    pub fn atcen(&self) -> AtcenR {
        AtcenR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Counter Enable"]
    #[inline(always)]
    pub fn cen(&mut self) -> CenW<TcrSpec> {
        CenW::new(self, 0)
    }
    #[doc = "Bit 1 - Counter Reset Enable"]
    #[inline(always)]
    pub fn crst(&mut self) -> CrstW<TcrSpec> {
        CrstW::new(self, 1)
    }
    #[doc = "Bit 4 - Allow Global Count Enable"]
    #[inline(always)]
    pub fn agcen(&mut self) -> AgcenW<TcrSpec> {
        AgcenW::new(self, 4)
    }
    #[doc = "Bit 5 - Allow Trigger Count Enable"]
    #[inline(always)]
    pub fn atcen(&mut self) -> AtcenW<TcrSpec> {
        AtcenW::new(self, 5)
    }
}
#[doc = "Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcrSpec;
impl crate::RegisterSpec for TcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcr::R`](R) reader structure"]
impl crate::Readable for TcrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcr::W`](W) writer structure"]
impl crate::Writable for TcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCR to value 0"]
impl crate::Resettable for TcrSpec {}
