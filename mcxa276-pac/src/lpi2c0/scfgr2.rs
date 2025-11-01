#[doc = "Register `SCFGR2` reader"]
pub type R = crate::R<Scfgr2Spec>;
#[doc = "Register `SCFGR2` writer"]
pub type W = crate::W<Scfgr2Spec>;
#[doc = "Field `CLKHOLD` reader - Clock Hold Time"]
pub type ClkholdR = crate::FieldReader;
#[doc = "Field `CLKHOLD` writer - Clock Hold Time"]
pub type ClkholdW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `DATAVD` reader - Data Valid Delay"]
pub type DatavdR = crate::FieldReader;
#[doc = "Field `DATAVD` writer - Data Valid Delay"]
pub type DatavdW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `FILTSCL` reader - Glitch Filter SCL"]
pub type FiltsclR = crate::FieldReader;
#[doc = "Field `FILTSCL` writer - Glitch Filter SCL"]
pub type FiltsclW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `FILTSDA` reader - Glitch Filter SDA"]
pub type FiltsdaR = crate::FieldReader;
#[doc = "Field `FILTSDA` writer - Glitch Filter SDA"]
pub type FiltsdaW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Clock Hold Time"]
    #[inline(always)]
    pub fn clkhold(&self) -> ClkholdR {
        ClkholdR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 8:13 - Data Valid Delay"]
    #[inline(always)]
    pub fn datavd(&self) -> DatavdR {
        DatavdR::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:19 - Glitch Filter SCL"]
    #[inline(always)]
    pub fn filtscl(&self) -> FiltsclR {
        FiltsclR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 24:27 - Glitch Filter SDA"]
    #[inline(always)]
    pub fn filtsda(&self) -> FiltsdaR {
        FiltsdaR::new(((self.bits >> 24) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Clock Hold Time"]
    #[inline(always)]
    pub fn clkhold(&mut self) -> ClkholdW<Scfgr2Spec> {
        ClkholdW::new(self, 0)
    }
    #[doc = "Bits 8:13 - Data Valid Delay"]
    #[inline(always)]
    pub fn datavd(&mut self) -> DatavdW<Scfgr2Spec> {
        DatavdW::new(self, 8)
    }
    #[doc = "Bits 16:19 - Glitch Filter SCL"]
    #[inline(always)]
    pub fn filtscl(&mut self) -> FiltsclW<Scfgr2Spec> {
        FiltsclW::new(self, 16)
    }
    #[doc = "Bits 24:27 - Glitch Filter SDA"]
    #[inline(always)]
    pub fn filtsda(&mut self) -> FiltsdaW<Scfgr2Spec> {
        FiltsdaW::new(self, 24)
    }
}
#[doc = "Target Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scfgr2Spec;
impl crate::RegisterSpec for Scfgr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scfgr2::R`](R) reader structure"]
impl crate::Readable for Scfgr2Spec {}
#[doc = "`write(|w| ..)` method takes [`scfgr2::W`](W) writer structure"]
impl crate::Writable for Scfgr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCFGR2 to value 0"]
impl crate::Resettable for Scfgr2Spec {}
