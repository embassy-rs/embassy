#[doc = "Register `OTGCTL` reader"]
pub type R = crate::R<OtgctlSpec>;
#[doc = "Register `OTGCTL` writer"]
pub type W = crate::W<OtgctlSpec>;
#[doc = "On-The-Go Pullup and Pulldown Resistor Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Otgen {
    #[doc = "0: If USBENSOFEN is 1 and HOSTMODEEN is 0 in the Control Register (CTL), then the D+ Data line pullup resistors are enabled. If HOSTMODEEN is 1, then the D+ and D- Data line pulldown resistors are engaged."]
    ConfigResistorsCtl = 0,
    #[doc = "1: Uses the pullup and pulldown controls in this register."]
    ConfigResistorsHere = 1,
}
impl From<Otgen> for bool {
    #[inline(always)]
    fn from(variant: Otgen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OTGEN` reader - On-The-Go Pullup and Pulldown Resistor Enable"]
pub type OtgenR = crate::BitReader<Otgen>;
impl OtgenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Otgen {
        match self.bits {
            false => Otgen::ConfigResistorsCtl,
            true => Otgen::ConfigResistorsHere,
        }
    }
    #[doc = "If USBENSOFEN is 1 and HOSTMODEEN is 0 in the Control Register (CTL), then the D+ Data line pullup resistors are enabled. If HOSTMODEEN is 1, then the D+ and D- Data line pulldown resistors are engaged."]
    #[inline(always)]
    pub fn is_config_resistors_ctl(&self) -> bool {
        *self == Otgen::ConfigResistorsCtl
    }
    #[doc = "Uses the pullup and pulldown controls in this register."]
    #[inline(always)]
    pub fn is_config_resistors_here(&self) -> bool {
        *self == Otgen::ConfigResistorsHere
    }
}
#[doc = "Field `OTGEN` writer - On-The-Go Pullup and Pulldown Resistor Enable"]
pub type OtgenW<'a, REG> = crate::BitWriter<'a, REG, Otgen>;
impl<'a, REG> OtgenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "If USBENSOFEN is 1 and HOSTMODEEN is 0 in the Control Register (CTL), then the D+ Data line pullup resistors are enabled. If HOSTMODEEN is 1, then the D+ and D- Data line pulldown resistors are engaged."]
    #[inline(always)]
    pub fn config_resistors_ctl(self) -> &'a mut crate::W<REG> {
        self.variant(Otgen::ConfigResistorsCtl)
    }
    #[doc = "Uses the pullup and pulldown controls in this register."]
    #[inline(always)]
    pub fn config_resistors_here(self) -> &'a mut crate::W<REG> {
        self.variant(Otgen::ConfigResistorsHere)
    }
}
#[doc = "D- Data Line Pulldown Resistor Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmlow {
    #[doc = "0: Disable"]
    DisDmPulldown = 0,
    #[doc = "1: Enable"]
    EnDmPulldown = 1,
}
impl From<Dmlow> for bool {
    #[inline(always)]
    fn from(variant: Dmlow) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMLOW` reader - D- Data Line Pulldown Resistor Enable"]
pub type DmlowR = crate::BitReader<Dmlow>;
impl DmlowR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmlow {
        match self.bits {
            false => Dmlow::DisDmPulldown,
            true => Dmlow::EnDmPulldown,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_dm_pulldown(&self) -> bool {
        *self == Dmlow::DisDmPulldown
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_dm_pulldown(&self) -> bool {
        *self == Dmlow::EnDmPulldown
    }
}
#[doc = "Field `DMLOW` writer - D- Data Line Pulldown Resistor Enable"]
pub type DmlowW<'a, REG> = crate::BitWriter<'a, REG, Dmlow>;
impl<'a, REG> DmlowW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_dm_pulldown(self) -> &'a mut crate::W<REG> {
        self.variant(Dmlow::DisDmPulldown)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_dm_pulldown(self) -> &'a mut crate::W<REG> {
        self.variant(Dmlow::EnDmPulldown)
    }
}
#[doc = "D+ Data Line pulldown Resistor Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dplow {
    #[doc = "0: Disable"]
    DisDpPulldown = 0,
    #[doc = "1: Enable"]
    EnDpPulldown = 1,
}
impl From<Dplow> for bool {
    #[inline(always)]
    fn from(variant: Dplow) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPLOW` reader - D+ Data Line pulldown Resistor Enable"]
pub type DplowR = crate::BitReader<Dplow>;
impl DplowR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dplow {
        match self.bits {
            false => Dplow::DisDpPulldown,
            true => Dplow::EnDpPulldown,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_dp_pulldown(&self) -> bool {
        *self == Dplow::DisDpPulldown
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_dp_pulldown(&self) -> bool {
        *self == Dplow::EnDpPulldown
    }
}
#[doc = "Field `DPLOW` writer - D+ Data Line pulldown Resistor Enable"]
pub type DplowW<'a, REG> = crate::BitWriter<'a, REG, Dplow>;
impl<'a, REG> DplowW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_dp_pulldown(self) -> &'a mut crate::W<REG> {
        self.variant(Dplow::DisDpPulldown)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_dp_pulldown(self) -> &'a mut crate::W<REG> {
        self.variant(Dplow::EnDpPulldown)
    }
}
#[doc = "D+ Data Line Pullup Resistor Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dphigh {
    #[doc = "0: Disable"]
    DisDpPullup = 0,
    #[doc = "1: Enable"]
    EnDpPullup = 1,
}
impl From<Dphigh> for bool {
    #[inline(always)]
    fn from(variant: Dphigh) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPHIGH` reader - D+ Data Line Pullup Resistor Enable"]
pub type DphighR = crate::BitReader<Dphigh>;
impl DphighR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dphigh {
        match self.bits {
            false => Dphigh::DisDpPullup,
            true => Dphigh::EnDpPullup,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_dp_pullup(&self) -> bool {
        *self == Dphigh::DisDpPullup
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_dp_pullup(&self) -> bool {
        *self == Dphigh::EnDpPullup
    }
}
#[doc = "Field `DPHIGH` writer - D+ Data Line Pullup Resistor Enable"]
pub type DphighW<'a, REG> = crate::BitWriter<'a, REG, Dphigh>;
impl<'a, REG> DphighW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_dp_pullup(self) -> &'a mut crate::W<REG> {
        self.variant(Dphigh::DisDpPullup)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_dp_pullup(self) -> &'a mut crate::W<REG> {
        self.variant(Dphigh::EnDpPullup)
    }
}
impl R {
    #[doc = "Bit 2 - On-The-Go Pullup and Pulldown Resistor Enable"]
    #[inline(always)]
    pub fn otgen(&self) -> OtgenR {
        OtgenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - D- Data Line Pulldown Resistor Enable"]
    #[inline(always)]
    pub fn dmlow(&self) -> DmlowR {
        DmlowR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - D+ Data Line pulldown Resistor Enable"]
    #[inline(always)]
    pub fn dplow(&self) -> DplowR {
        DplowR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 7 - D+ Data Line Pullup Resistor Enable"]
    #[inline(always)]
    pub fn dphigh(&self) -> DphighR {
        DphighR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 2 - On-The-Go Pullup and Pulldown Resistor Enable"]
    #[inline(always)]
    pub fn otgen(&mut self) -> OtgenW<OtgctlSpec> {
        OtgenW::new(self, 2)
    }
    #[doc = "Bit 4 - D- Data Line Pulldown Resistor Enable"]
    #[inline(always)]
    pub fn dmlow(&mut self) -> DmlowW<OtgctlSpec> {
        DmlowW::new(self, 4)
    }
    #[doc = "Bit 5 - D+ Data Line pulldown Resistor Enable"]
    #[inline(always)]
    pub fn dplow(&mut self) -> DplowW<OtgctlSpec> {
        DplowW::new(self, 5)
    }
    #[doc = "Bit 7 - D+ Data Line Pullup Resistor Enable"]
    #[inline(always)]
    pub fn dphigh(&mut self) -> DphighW<OtgctlSpec> {
        DphighW::new(self, 7)
    }
}
#[doc = "OTG Control\n\nYou can [`read`](crate::Reg::read) this register and get [`otgctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OtgctlSpec;
impl crate::RegisterSpec for OtgctlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`otgctl::R`](R) reader structure"]
impl crate::Readable for OtgctlSpec {}
#[doc = "`write(|w| ..)` method takes [`otgctl::W`](W) writer structure"]
impl crate::Writable for OtgctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OTGCTL to value 0"]
impl crate::Resettable for OtgctlSpec {}
