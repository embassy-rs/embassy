#[doc = "Register `MCFGR2` reader"]
pub type R = crate::R<Mcfgr2Spec>;
#[doc = "Register `MCFGR2` writer"]
pub type W = crate::W<Mcfgr2Spec>;
#[doc = "Field `BUSIDLE` reader - Bus Idle Timeout"]
pub type BusidleR = crate::FieldReader<u16>;
#[doc = "Field `BUSIDLE` writer - Bus Idle Timeout"]
pub type BusidleW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
#[doc = "Field `FILTSCL` reader - Glitch Filter SCL"]
pub type FiltsclR = crate::FieldReader;
#[doc = "Field `FILTSCL` writer - Glitch Filter SCL"]
pub type FiltsclW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `FILTSDA` reader - Glitch Filter SDA"]
pub type FiltsdaR = crate::FieldReader;
#[doc = "Field `FILTSDA` writer - Glitch Filter SDA"]
pub type FiltsdaW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:11 - Bus Idle Timeout"]
    #[inline(always)]
    pub fn busidle(&self) -> BusidleR {
        BusidleR::new((self.bits & 0x0fff) as u16)
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
    #[doc = "Bits 0:11 - Bus Idle Timeout"]
    #[inline(always)]
    pub fn busidle(&mut self) -> BusidleW<Mcfgr2Spec> {
        BusidleW::new(self, 0)
    }
    #[doc = "Bits 16:19 - Glitch Filter SCL"]
    #[inline(always)]
    pub fn filtscl(&mut self) -> FiltsclW<Mcfgr2Spec> {
        FiltsclW::new(self, 16)
    }
    #[doc = "Bits 24:27 - Glitch Filter SDA"]
    #[inline(always)]
    pub fn filtsda(&mut self) -> FiltsdaW<Mcfgr2Spec> {
        FiltsdaW::new(self, 24)
    }
}
#[doc = "Controller Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mcfgr2Spec;
impl crate::RegisterSpec for Mcfgr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcfgr2::R`](R) reader structure"]
impl crate::Readable for Mcfgr2Spec {}
#[doc = "`write(|w| ..)` method takes [`mcfgr2::W`](W) writer structure"]
impl crate::Writable for Mcfgr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCFGR2 to value 0"]
impl crate::Resettable for Mcfgr2Spec {}
