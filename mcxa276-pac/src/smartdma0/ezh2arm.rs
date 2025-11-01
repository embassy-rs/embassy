#[doc = "Register `EZH2ARM` reader"]
pub type R = crate::R<Ezh2armSpec>;
#[doc = "Register `EZH2ARM` writer"]
pub type W = crate::W<Ezh2armSpec>;
#[doc = "Field `GP` reader - General purpose register bits Writing to EZH2ARM triggers the ARM interrupt when ARM2EZH \\[1:0\\] == 2h"]
pub type GpR = crate::FieldReader<u32>;
#[doc = "Field `GP` writer - General purpose register bits Writing to EZH2ARM triggers the ARM interrupt when ARM2EZH \\[1:0\\] == 2h"]
pub type GpW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - General purpose register bits Writing to EZH2ARM triggers the ARM interrupt when ARM2EZH \\[1:0\\] == 2h"]
    #[inline(always)]
    pub fn gp(&self) -> GpR {
        GpR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - General purpose register bits Writing to EZH2ARM triggers the ARM interrupt when ARM2EZH \\[1:0\\] == 2h"]
    #[inline(always)]
    pub fn gp(&mut self) -> GpW<Ezh2armSpec> {
        GpW::new(self, 0)
    }
}
#[doc = "EZH to ARM Trigger\n\nYou can [`read`](crate::Reg::read) this register and get [`ezh2arm::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ezh2arm::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ezh2armSpec;
impl crate::RegisterSpec for Ezh2armSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ezh2arm::R`](R) reader structure"]
impl crate::Readable for Ezh2armSpec {}
#[doc = "`write(|w| ..)` method takes [`ezh2arm::W`](W) writer structure"]
impl crate::Writable for Ezh2armSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EZH2ARM to value 0"]
impl crate::Resettable for Ezh2armSpec {}
