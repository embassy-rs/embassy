#[doc = "Register `LDOCSR` reader"]
pub type R = crate::R<LdocsrSpec>;
#[doc = "Register `LDOCSR` writer"]
pub type W = crate::W<LdocsrSpec>;
#[doc = "LDO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldoen {
    #[doc = "0: LDO is disabled"]
    Disabled = 0,
    #[doc = "1: LDO is enabled"]
    Enabled = 1,
}
impl From<Ldoen> for bool {
    #[inline(always)]
    fn from(variant: Ldoen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LDOEN` reader - LDO Enable"]
pub type LdoenR = crate::BitReader<Ldoen>;
impl LdoenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldoen {
        match self.bits {
            false => Ldoen::Disabled,
            true => Ldoen::Enabled,
        }
    }
    #[doc = "LDO is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ldoen::Disabled
    }
    #[doc = "LDO is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ldoen::Enabled
    }
}
#[doc = "Field `LDOEN` writer - LDO Enable"]
pub type LdoenW<'a, REG> = crate::BitWriter<'a, REG, Ldoen>;
impl<'a, REG> LdoenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "LDO is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldoen::Disabled)
    }
    #[doc = "LDO is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldoen::Enabled)
    }
}
#[doc = "LDO output voltage select\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VoutSel {
    #[doc = "0: VOUT = 1V"]
    Vout1v1 = 0,
    #[doc = "1: VOUT = 1V"]
    Vout1v2 = 1,
    #[doc = "2: VOUT = 1V"]
    Vout1v3 = 2,
    #[doc = "3: VOUT = 1.05V"]
    Vout105v = 3,
    #[doc = "4: VOUT = 1.1V"]
    Vout11v = 4,
    #[doc = "5: VOUT = 1.15V"]
    Vout115v = 5,
    #[doc = "6: VOUT = 1.2V"]
    Vout12v = 6,
    #[doc = "7: VOUT = 1.25V"]
    Vout125v = 7,
}
impl From<VoutSel> for u8 {
    #[inline(always)]
    fn from(variant: VoutSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for VoutSel {
    type Ux = u8;
}
impl crate::IsEnum for VoutSel {}
#[doc = "Field `VOUT_SEL` reader - LDO output voltage select"]
pub type VoutSelR = crate::FieldReader<VoutSel>;
impl VoutSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> VoutSel {
        match self.bits {
            0 => VoutSel::Vout1v1,
            1 => VoutSel::Vout1v2,
            2 => VoutSel::Vout1v3,
            3 => VoutSel::Vout105v,
            4 => VoutSel::Vout11v,
            5 => VoutSel::Vout115v,
            6 => VoutSel::Vout12v,
            7 => VoutSel::Vout125v,
            _ => unreachable!(),
        }
    }
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn is_vout_1v_1(&self) -> bool {
        *self == VoutSel::Vout1v1
    }
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn is_vout_1v_2(&self) -> bool {
        *self == VoutSel::Vout1v2
    }
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn is_vout_1v_3(&self) -> bool {
        *self == VoutSel::Vout1v3
    }
    #[doc = "VOUT = 1.05V"]
    #[inline(always)]
    pub fn is_vout_105v(&self) -> bool {
        *self == VoutSel::Vout105v
    }
    #[doc = "VOUT = 1.1V"]
    #[inline(always)]
    pub fn is_vout_11v(&self) -> bool {
        *self == VoutSel::Vout11v
    }
    #[doc = "VOUT = 1.15V"]
    #[inline(always)]
    pub fn is_vout_115v(&self) -> bool {
        *self == VoutSel::Vout115v
    }
    #[doc = "VOUT = 1.2V"]
    #[inline(always)]
    pub fn is_vout_12v(&self) -> bool {
        *self == VoutSel::Vout12v
    }
    #[doc = "VOUT = 1.25V"]
    #[inline(always)]
    pub fn is_vout_125v(&self) -> bool {
        *self == VoutSel::Vout125v
    }
}
#[doc = "Field `VOUT_SEL` writer - LDO output voltage select"]
pub type VoutSelW<'a, REG> = crate::FieldWriter<'a, REG, 3, VoutSel, crate::Safe>;
impl<'a, REG> VoutSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn vout_1v_1(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout1v1)
    }
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn vout_1v_2(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout1v2)
    }
    #[doc = "VOUT = 1V"]
    #[inline(always)]
    pub fn vout_1v_3(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout1v3)
    }
    #[doc = "VOUT = 1.05V"]
    #[inline(always)]
    pub fn vout_105v(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout105v)
    }
    #[doc = "VOUT = 1.1V"]
    #[inline(always)]
    pub fn vout_11v(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout11v)
    }
    #[doc = "VOUT = 1.15V"]
    #[inline(always)]
    pub fn vout_115v(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout115v)
    }
    #[doc = "VOUT = 1.2V"]
    #[inline(always)]
    pub fn vout_12v(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout12v)
    }
    #[doc = "VOUT = 1.25V"]
    #[inline(always)]
    pub fn vout_125v(self) -> &'a mut crate::W<REG> {
        self.variant(VoutSel::Vout125v)
    }
}
#[doc = "LDO Bypass\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldobypass {
    #[doc = "0: LDO is not bypassed"]
    Disabled = 0,
    #[doc = "1: LDO is bypassed"]
    Enabled = 1,
}
impl From<Ldobypass> for bool {
    #[inline(always)]
    fn from(variant: Ldobypass) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LDOBYPASS` reader - LDO Bypass"]
pub type LdobypassR = crate::BitReader<Ldobypass>;
impl LdobypassR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldobypass {
        match self.bits {
            false => Ldobypass::Disabled,
            true => Ldobypass::Enabled,
        }
    }
    #[doc = "LDO is not bypassed"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ldobypass::Disabled
    }
    #[doc = "LDO is bypassed"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ldobypass::Enabled
    }
}
#[doc = "Field `LDOBYPASS` writer - LDO Bypass"]
pub type LdobypassW<'a, REG> = crate::BitWriter<'a, REG, Ldobypass>;
impl<'a, REG> LdobypassW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "LDO is not bypassed"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldobypass::Disabled)
    }
    #[doc = "LDO is bypassed"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldobypass::Enabled)
    }
}
#[doc = "LDO VOUT OK Inform.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoutOk {
    #[doc = "0: LDO output VOUT is not OK"]
    Disabled = 0,
    #[doc = "1: LDO output VOUT is OK"]
    Enabled = 1,
}
impl From<VoutOk> for bool {
    #[inline(always)]
    fn from(variant: VoutOk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VOUT_OK` reader - LDO VOUT OK Inform."]
pub type VoutOkR = crate::BitReader<VoutOk>;
impl VoutOkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> VoutOk {
        match self.bits {
            false => VoutOk::Disabled,
            true => VoutOk::Enabled,
        }
    }
    #[doc = "LDO output VOUT is not OK"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == VoutOk::Disabled
    }
    #[doc = "LDO output VOUT is OK"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == VoutOk::Enabled
    }
}
impl R {
    #[doc = "Bit 0 - LDO Enable"]
    #[inline(always)]
    pub fn ldoen(&self) -> LdoenR {
        LdoenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:3 - LDO output voltage select"]
    #[inline(always)]
    pub fn vout_sel(&self) -> VoutSelR {
        VoutSelR::new(((self.bits >> 1) & 7) as u8)
    }
    #[doc = "Bit 4 - LDO Bypass"]
    #[inline(always)]
    pub fn ldobypass(&self) -> LdobypassR {
        LdobypassR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 31 - LDO VOUT OK Inform."]
    #[inline(always)]
    pub fn vout_ok(&self) -> VoutOkR {
        VoutOkR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LDO Enable"]
    #[inline(always)]
    pub fn ldoen(&mut self) -> LdoenW<LdocsrSpec> {
        LdoenW::new(self, 0)
    }
    #[doc = "Bits 1:3 - LDO output voltage select"]
    #[inline(always)]
    pub fn vout_sel(&mut self) -> VoutSelW<LdocsrSpec> {
        VoutSelW::new(self, 1)
    }
    #[doc = "Bit 4 - LDO Bypass"]
    #[inline(always)]
    pub fn ldobypass(&mut self) -> LdobypassW<LdocsrSpec> {
        LdobypassW::new(self, 4)
    }
}
#[doc = "LDO Control and Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ldocsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ldocsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LdocsrSpec;
impl crate::RegisterSpec for LdocsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ldocsr::R`](R) reader structure"]
impl crate::Readable for LdocsrSpec {}
#[doc = "`write(|w| ..)` method takes [`ldocsr::W`](W) writer structure"]
impl crate::Writable for LdocsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LDOCSR to value 0x08"]
impl crate::Resettable for LdocsrSpec {
    const RESET_VALUE: u32 = 0x08;
}
