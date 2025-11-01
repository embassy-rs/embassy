#[doc = "Register `CCR1` reader"]
pub type R = crate::R<Ccr1Spec>;
#[doc = "Register `CCR1` writer"]
pub type W = crate::W<Ccr1Spec>;
#[doc = "Field `SCKSET` reader - SCK Setup"]
pub type ScksetR = crate::FieldReader;
#[doc = "Field `SCKSET` writer - SCK Setup"]
pub type ScksetW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `SCKHLD` reader - SCK Hold"]
pub type SckhldR = crate::FieldReader;
#[doc = "Field `SCKHLD` writer - SCK Hold"]
pub type SckhldW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `PCSPCS` reader - PCS to PCS Delay"]
pub type PcspcsR = crate::FieldReader;
#[doc = "Field `PCSPCS` writer - PCS to PCS Delay"]
pub type PcspcsW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `SCKSCK` reader - SCK Inter-Frame Delay"]
pub type ScksckR = crate::FieldReader;
#[doc = "Field `SCKSCK` writer - SCK Inter-Frame Delay"]
pub type ScksckW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - SCK Setup"]
    #[inline(always)]
    pub fn sckset(&self) -> ScksetR {
        ScksetR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - SCK Hold"]
    #[inline(always)]
    pub fn sckhld(&self) -> SckhldR {
        SckhldR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - PCS to PCS Delay"]
    #[inline(always)]
    pub fn pcspcs(&self) -> PcspcsR {
        PcspcsR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - SCK Inter-Frame Delay"]
    #[inline(always)]
    pub fn scksck(&self) -> ScksckR {
        ScksckR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - SCK Setup"]
    #[inline(always)]
    pub fn sckset(&mut self) -> ScksetW<Ccr1Spec> {
        ScksetW::new(self, 0)
    }
    #[doc = "Bits 8:15 - SCK Hold"]
    #[inline(always)]
    pub fn sckhld(&mut self) -> SckhldW<Ccr1Spec> {
        SckhldW::new(self, 8)
    }
    #[doc = "Bits 16:23 - PCS to PCS Delay"]
    #[inline(always)]
    pub fn pcspcs(&mut self) -> PcspcsW<Ccr1Spec> {
        PcspcsW::new(self, 16)
    }
    #[doc = "Bits 24:31 - SCK Inter-Frame Delay"]
    #[inline(always)]
    pub fn scksck(&mut self) -> ScksckW<Ccr1Spec> {
        ScksckW::new(self, 24)
    }
}
#[doc = "Clock Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ccr1Spec;
impl crate::RegisterSpec for Ccr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr1::R`](R) reader structure"]
impl crate::Readable for Ccr1Spec {}
#[doc = "`write(|w| ..)` method takes [`ccr1::W`](W) writer structure"]
impl crate::Writable for Ccr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR1 to value 0"]
impl crate::Resettable for Ccr1Spec {}
