#[doc = "Register `ARM2EZH` reader"]
pub type R = crate::R<Arm2ezhSpec>;
#[doc = "Register `ARM2EZH` writer"]
pub type W = crate::W<Arm2ezhSpec>;
#[doc = "Field `IE` reader - Interrupt Enable"]
pub type IeR = crate::FieldReader;
#[doc = "Field `IE` writer - Interrupt Enable"]
pub type IeW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `GP` reader - General purpose register bits"]
pub type GpR = crate::FieldReader<u32>;
#[doc = "Field `GP` writer - General purpose register bits"]
pub type GpW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
impl R {
    #[doc = "Bits 0:1 - Interrupt Enable"]
    #[inline(always)]
    pub fn ie(&self) -> IeR {
        IeR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:31 - General purpose register bits"]
    #[inline(always)]
    pub fn gp(&self) -> GpR {
        GpR::new((self.bits >> 2) & 0x3fff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:1 - Interrupt Enable"]
    #[inline(always)]
    pub fn ie(&mut self) -> IeW<Arm2ezhSpec> {
        IeW::new(self, 0)
    }
    #[doc = "Bits 2:31 - General purpose register bits"]
    #[inline(always)]
    pub fn gp(&mut self) -> GpW<Arm2ezhSpec> {
        GpW::new(self, 2)
    }
}
#[doc = "ARM to EZH Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`arm2ezh::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`arm2ezh::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Arm2ezhSpec;
impl crate::RegisterSpec for Arm2ezhSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`arm2ezh::R`](R) reader structure"]
impl crate::Readable for Arm2ezhSpec {}
#[doc = "`write(|w| ..)` method takes [`arm2ezh::W`](W) writer structure"]
impl crate::Writable for Arm2ezhSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARM2EZH to value 0"]
impl crate::Resettable for Arm2ezhSpec {}
