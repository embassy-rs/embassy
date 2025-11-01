#[doc = "Register `SHIFTSDEN` reader"]
pub type R = crate::R<ShiftsdenSpec>;
#[doc = "Register `SHIFTSDEN` writer"]
pub type W = crate::W<ShiftsdenSpec>;
#[doc = "Field `SSDE` reader - Shifter Status DMA Enable"]
pub type SsdeR = crate::FieldReader;
#[doc = "Field `SSDE` writer - Shifter Status DMA Enable"]
pub type SsdeW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Shifter Status DMA Enable"]
    #[inline(always)]
    pub fn ssde(&self) -> SsdeR {
        SsdeR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Shifter Status DMA Enable"]
    #[inline(always)]
    pub fn ssde(&mut self) -> SsdeW<ShiftsdenSpec> {
        SsdeW::new(self, 0)
    }
}
#[doc = "Shifter Status DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftsden::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftsden::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftsdenSpec;
impl crate::RegisterSpec for ShiftsdenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftsden::R`](R) reader structure"]
impl crate::Readable for ShiftsdenSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftsden::W`](W) writer structure"]
impl crate::Writable for ShiftsdenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTSDEN to value 0"]
impl crate::Resettable for ShiftsdenSpec {}
