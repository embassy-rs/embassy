#[doc = "Register `SRAM_XEN_DP` reader"]
pub type R = crate::R<SramXenDpSpec>;
#[doc = "Register `SRAM_XEN_DP` writer"]
pub type W = crate::W<SramXenDpSpec>;
#[doc = "Field `RAMX0_XEN` reader - Refer to SRAM_XEN for more details."]
pub type Ramx0XenR = crate::BitReader;
#[doc = "Field `RAMX0_XEN` writer - Refer to SRAM_XEN for more details."]
pub type Ramx0XenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAMX1_XEN` reader - Refer to SRAM_XEN for more details."]
pub type Ramx1XenR = crate::BitReader;
#[doc = "Field `RAMX1_XEN` writer - Refer to SRAM_XEN for more details."]
pub type Ramx1XenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAMA0_XEN` reader - Refer to SRAM_XEN for more details."]
pub type Rama0XenR = crate::BitReader;
#[doc = "Field `RAMA0_XEN` writer - Refer to SRAM_XEN for more details."]
pub type Rama0XenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAMA1_XEN` reader - Refer to SRAM_XEN for more details."]
pub type Rama1XenR = crate::BitReader;
#[doc = "Field `RAMA1_XEN` writer - Refer to SRAM_XEN for more details."]
pub type Rama1XenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAMB_XEN` reader - Refer to SRAM_XEN for more details."]
pub type RambXenR = crate::BitReader;
#[doc = "Field `RAMB_XEN` writer - Refer to SRAM_XEN for more details."]
pub type RambXenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAMC_XEN` reader - Refer to SRAM_XEN for more details."]
pub type RamcXenR = crate::BitReader;
#[doc = "Field `RAMC_XEN` writer - Refer to SRAM_XEN for more details."]
pub type RamcXenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramx0_xen(&self) -> Ramx0XenR {
        Ramx0XenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramx1_xen(&self) -> Ramx1XenR {
        Ramx1XenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn rama0_xen(&self) -> Rama0XenR {
        Rama0XenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn rama1_xen(&self) -> Rama1XenR {
        Rama1XenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramb_xen(&self) -> RambXenR {
        RambXenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramc_xen(&self) -> RamcXenR {
        RamcXenR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramx0_xen(&mut self) -> Ramx0XenW<SramXenDpSpec> {
        Ramx0XenW::new(self, 0)
    }
    #[doc = "Bit 1 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramx1_xen(&mut self) -> Ramx1XenW<SramXenDpSpec> {
        Ramx1XenW::new(self, 1)
    }
    #[doc = "Bit 2 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn rama0_xen(&mut self) -> Rama0XenW<SramXenDpSpec> {
        Rama0XenW::new(self, 2)
    }
    #[doc = "Bit 3 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn rama1_xen(&mut self) -> Rama1XenW<SramXenDpSpec> {
        Rama1XenW::new(self, 3)
    }
    #[doc = "Bit 4 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramb_xen(&mut self) -> RambXenW<SramXenDpSpec> {
        RambXenW::new(self, 4)
    }
    #[doc = "Bit 5 - Refer to SRAM_XEN for more details."]
    #[inline(always)]
    pub fn ramc_xen(&mut self) -> RamcXenW<SramXenDpSpec> {
        RamcXenW::new(self, 5)
    }
}
#[doc = "RAM XEN Control (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`sram_xen_dp::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sram_xen_dp::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SramXenDpSpec;
impl crate::RegisterSpec for SramXenDpSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sram_xen_dp::R`](R) reader structure"]
impl crate::Readable for SramXenDpSpec {}
#[doc = "`write(|w| ..)` method takes [`sram_xen_dp::W`](W) writer structure"]
impl crate::Writable for SramXenDpSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SRAM_XEN_DP to value 0"]
impl crate::Resettable for SramXenDpSpec {}
