#[doc = "Register `MSFCFG` reader"]
pub type R = crate::R<MsfcfgSpec>;
#[doc = "Register `MSFCFG` writer"]
pub type W = crate::W<MsfcfgSpec>;
#[doc = "user IFR sector 0 erase control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfrEraseDis0 {
    #[doc = "0: Enable IFR sector erase."]
    Enable = 0,
    #[doc = "1: Disable IFR sector erase, write one lock until a system reset."]
    Disable = 1,
}
impl From<IfrEraseDis0> for bool {
    #[inline(always)]
    fn from(variant: IfrEraseDis0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IFR_ERASE_DIS0` reader - user IFR sector 0 erase control"]
pub type IfrEraseDis0R = crate::BitReader<IfrEraseDis0>;
impl IfrEraseDis0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IfrEraseDis0 {
        match self.bits {
            false => IfrEraseDis0::Enable,
            true => IfrEraseDis0::Disable,
        }
    }
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IfrEraseDis0::Enable
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IfrEraseDis0::Disable
    }
}
#[doc = "Field `IFR_ERASE_DIS0` writer - user IFR sector 0 erase control"]
pub type IfrEraseDis0W<'a, REG> = crate::BitWriter<'a, REG, IfrEraseDis0>;
impl<'a, REG> IfrEraseDis0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis0::Enable)
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis0::Disable)
    }
}
#[doc = "user IFR sector 1 erase control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfrEraseDis1 {
    #[doc = "0: Enable IFR sector erase."]
    Enable = 0,
    #[doc = "1: Disable IFR sector erase, write one lock until a system reset."]
    Disable = 1,
}
impl From<IfrEraseDis1> for bool {
    #[inline(always)]
    fn from(variant: IfrEraseDis1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IFR_ERASE_DIS1` reader - user IFR sector 1 erase control"]
pub type IfrEraseDis1R = crate::BitReader<IfrEraseDis1>;
impl IfrEraseDis1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IfrEraseDis1 {
        match self.bits {
            false => IfrEraseDis1::Enable,
            true => IfrEraseDis1::Disable,
        }
    }
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IfrEraseDis1::Enable
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IfrEraseDis1::Disable
    }
}
#[doc = "Field `IFR_ERASE_DIS1` writer - user IFR sector 1 erase control"]
pub type IfrEraseDis1W<'a, REG> = crate::BitWriter<'a, REG, IfrEraseDis1>;
impl<'a, REG> IfrEraseDis1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis1::Enable)
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis1::Disable)
    }
}
#[doc = "user IFR sector 2 erase control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfrEraseDis2 {
    #[doc = "0: Enable IFR sector erase."]
    Enable = 0,
    #[doc = "1: Disable IFR sector erase, write one lock until a system reset."]
    Disable = 1,
}
impl From<IfrEraseDis2> for bool {
    #[inline(always)]
    fn from(variant: IfrEraseDis2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IFR_ERASE_DIS2` reader - user IFR sector 2 erase control"]
pub type IfrEraseDis2R = crate::BitReader<IfrEraseDis2>;
impl IfrEraseDis2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IfrEraseDis2 {
        match self.bits {
            false => IfrEraseDis2::Enable,
            true => IfrEraseDis2::Disable,
        }
    }
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IfrEraseDis2::Enable
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IfrEraseDis2::Disable
    }
}
#[doc = "Field `IFR_ERASE_DIS2` writer - user IFR sector 2 erase control"]
pub type IfrEraseDis2W<'a, REG> = crate::BitWriter<'a, REG, IfrEraseDis2>;
impl<'a, REG> IfrEraseDis2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis2::Enable)
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis2::Disable)
    }
}
#[doc = "user IFR sector 3 erase control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfrEraseDis3 {
    #[doc = "0: Enable IFR sector erase."]
    Enable = 0,
    #[doc = "1: Disable IFR sector erase, write one lock until a system reset."]
    Disable = 1,
}
impl From<IfrEraseDis3> for bool {
    #[inline(always)]
    fn from(variant: IfrEraseDis3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IFR_ERASE_DIS3` reader - user IFR sector 3 erase control"]
pub type IfrEraseDis3R = crate::BitReader<IfrEraseDis3>;
impl IfrEraseDis3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IfrEraseDis3 {
        match self.bits {
            false => IfrEraseDis3::Enable,
            true => IfrEraseDis3::Disable,
        }
    }
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IfrEraseDis3::Enable
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IfrEraseDis3::Disable
    }
}
#[doc = "Field `IFR_ERASE_DIS3` writer - user IFR sector 3 erase control"]
pub type IfrEraseDis3W<'a, REG> = crate::BitWriter<'a, REG, IfrEraseDis3>;
impl<'a, REG> IfrEraseDis3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable IFR sector erase."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis3::Enable)
    }
    #[doc = "Disable IFR sector erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IfrEraseDis3::Disable)
    }
}
#[doc = "Mass erase control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassEraseDis {
    #[doc = "0: Enables mass erase"]
    Enable = 0,
    #[doc = "1: Disables mass erase, write one lock until a system reset."]
    Disable = 1,
}
impl From<MassEraseDis> for bool {
    #[inline(always)]
    fn from(variant: MassEraseDis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MASS_ERASE_DIS` reader - Mass erase control"]
pub type MassEraseDisR = crate::BitReader<MassEraseDis>;
impl MassEraseDisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> MassEraseDis {
        match self.bits {
            false => MassEraseDis::Enable,
            true => MassEraseDis::Disable,
        }
    }
    #[doc = "Enables mass erase"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == MassEraseDis::Enable
    }
    #[doc = "Disables mass erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == MassEraseDis::Disable
    }
}
#[doc = "Field `MASS_ERASE_DIS` writer - Mass erase control"]
pub type MassEraseDisW<'a, REG> = crate::BitWriter<'a, REG, MassEraseDis>;
impl<'a, REG> MassEraseDisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables mass erase"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(MassEraseDis::Enable)
    }
    #[doc = "Disables mass erase, write one lock until a system reset."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(MassEraseDis::Disable)
    }
}
impl R {
    #[doc = "Bit 0 - user IFR sector 0 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis0(&self) -> IfrEraseDis0R {
        IfrEraseDis0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - user IFR sector 1 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis1(&self) -> IfrEraseDis1R {
        IfrEraseDis1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - user IFR sector 2 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis2(&self) -> IfrEraseDis2R {
        IfrEraseDis2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - user IFR sector 3 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis3(&self) -> IfrEraseDis3R {
        IfrEraseDis3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Mass erase control"]
    #[inline(always)]
    pub fn mass_erase_dis(&self) -> MassEraseDisR {
        MassEraseDisR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - user IFR sector 0 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis0(&mut self) -> IfrEraseDis0W<MsfcfgSpec> {
        IfrEraseDis0W::new(self, 0)
    }
    #[doc = "Bit 1 - user IFR sector 1 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis1(&mut self) -> IfrEraseDis1W<MsfcfgSpec> {
        IfrEraseDis1W::new(self, 1)
    }
    #[doc = "Bit 2 - user IFR sector 2 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis2(&mut self) -> IfrEraseDis2W<MsfcfgSpec> {
        IfrEraseDis2W::new(self, 2)
    }
    #[doc = "Bit 3 - user IFR sector 3 erase control"]
    #[inline(always)]
    pub fn ifr_erase_dis3(&mut self) -> IfrEraseDis3W<MsfcfgSpec> {
        IfrEraseDis3W::new(self, 3)
    }
    #[doc = "Bit 8 - Mass erase control"]
    #[inline(always)]
    pub fn mass_erase_dis(&mut self) -> MassEraseDisW<MsfcfgSpec> {
        MassEraseDisW::new(self, 8)
    }
}
#[doc = "MSF Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`msfcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`msfcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MsfcfgSpec;
impl crate::RegisterSpec for MsfcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`msfcfg::R`](R) reader structure"]
impl crate::Readable for MsfcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`msfcfg::W`](W) writer structure"]
impl crate::Writable for MsfcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MSFCFG to value 0"]
impl crate::Resettable for MsfcfgSpec {}
