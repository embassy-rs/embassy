#[doc = "Register `CONTROL` reader"]
pub type R = crate::R<ControlSpec>;
#[doc = "Register `CONTROL` writer"]
pub type W = crate::W<ControlSpec>;
#[doc = "VBUS Monitoring Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VbusSourceSel {
    #[doc = "1: Resistive divider attached to a GPIO pin"]
    Resistive = 1,
}
impl From<VbusSourceSel> for bool {
    #[inline(always)]
    fn from(variant: VbusSourceSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VBUS_SOURCE_SEL` reader - VBUS Monitoring Source Select"]
pub type VbusSourceSelR = crate::BitReader<VbusSourceSel>;
impl VbusSourceSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<VbusSourceSel> {
        match self.bits {
            true => Some(VbusSourceSel::Resistive),
            _ => None,
        }
    }
    #[doc = "Resistive divider attached to a GPIO pin"]
    #[inline(always)]
    pub fn is_resistive(&self) -> bool {
        *self == VbusSourceSel::Resistive
    }
}
#[doc = "Field `VBUS_SOURCE_SEL` writer - VBUS Monitoring Source Select"]
pub type VbusSourceSelW<'a, REG> = crate::BitWriter<'a, REG, VbusSourceSel>;
impl<'a, REG> VbusSourceSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Resistive divider attached to a GPIO pin"]
    #[inline(always)]
    pub fn resistive(self) -> &'a mut crate::W<REG> {
        self.variant(VbusSourceSel::Resistive)
    }
}
#[doc = "VBUS Session Valid status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessVld {
    #[doc = "0: Below"]
    SessVldLow = 0,
    #[doc = "1: Above"]
    SessVldHigh = 1,
}
impl From<SessVld> for bool {
    #[inline(always)]
    fn from(variant: SessVld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SESS_VLD` reader - VBUS Session Valid status"]
pub type SessVldR = crate::BitReader<SessVld>;
impl SessVldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SessVld {
        match self.bits {
            false => SessVld::SessVldLow,
            true => SessVld::SessVldHigh,
        }
    }
    #[doc = "Below"]
    #[inline(always)]
    pub fn is_sess_vld_low(&self) -> bool {
        *self == SessVld::SessVldLow
    }
    #[doc = "Above"]
    #[inline(always)]
    pub fn is_sess_vld_high(&self) -> bool {
        *self == SessVld::SessVldHigh
    }
}
#[doc = "DP Pullup in Non-OTG Device Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dppullupnonotg {
    #[doc = "0: Disable"]
    DisDeviceDpPu = 0,
    #[doc = "1: Enabled"]
    EnDeviceDpPu = 1,
}
impl From<Dppullupnonotg> for bool {
    #[inline(always)]
    fn from(variant: Dppullupnonotg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPPULLUPNONOTG` reader - DP Pullup in Non-OTG Device Mode"]
pub type DppullupnonotgR = crate::BitReader<Dppullupnonotg>;
impl DppullupnonotgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dppullupnonotg {
        match self.bits {
            false => Dppullupnonotg::DisDeviceDpPu,
            true => Dppullupnonotg::EnDeviceDpPu,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_device_dp_pu(&self) -> bool {
        *self == Dppullupnonotg::DisDeviceDpPu
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_en_device_dp_pu(&self) -> bool {
        *self == Dppullupnonotg::EnDeviceDpPu
    }
}
#[doc = "Field `DPPULLUPNONOTG` writer - DP Pullup in Non-OTG Device Mode"]
pub type DppullupnonotgW<'a, REG> = crate::BitWriter<'a, REG, Dppullupnonotg>;
impl<'a, REG> DppullupnonotgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_device_dp_pu(self) -> &'a mut crate::W<REG> {
        self.variant(Dppullupnonotg::DisDeviceDpPu)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn en_device_dp_pu(self) -> &'a mut crate::W<REG> {
        self.variant(Dppullupnonotg::EnDeviceDpPu)
    }
}
impl R {
    #[doc = "Bit 0 - VBUS Monitoring Source Select"]
    #[inline(always)]
    pub fn vbus_source_sel(&self) -> VbusSourceSelR {
        VbusSourceSelR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - VBUS Session Valid status"]
    #[inline(always)]
    pub fn sess_vld(&self) -> SessVldR {
        SessVldR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - DP Pullup in Non-OTG Device Mode"]
    #[inline(always)]
    pub fn dppullupnonotg(&self) -> DppullupnonotgR {
        DppullupnonotgR::new(((self.bits >> 4) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - VBUS Monitoring Source Select"]
    #[inline(always)]
    pub fn vbus_source_sel(&mut self) -> VbusSourceSelW<ControlSpec> {
        VbusSourceSelW::new(self, 0)
    }
    #[doc = "Bit 4 - DP Pullup in Non-OTG Device Mode"]
    #[inline(always)]
    pub fn dppullupnonotg(&mut self) -> DppullupnonotgW<ControlSpec> {
        DppullupnonotgW::new(self, 4)
    }
}
#[doc = "USB OTG Control\n\nYou can [`read`](crate::Reg::read) this register and get [`control::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ControlSpec;
impl crate::RegisterSpec for ControlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`control::R`](R) reader structure"]
impl crate::Readable for ControlSpec {}
#[doc = "`write(|w| ..)` method takes [`control::W`](W) writer structure"]
impl crate::Writable for ControlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONTROL to value 0"]
impl crate::Resettable for ControlSpec {}
