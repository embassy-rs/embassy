#[doc = "Register `SHIFTEIEN` reader"]
pub type R = crate::R<ShifteienSpec>;
#[doc = "Register `SHIFTEIEN` writer"]
pub type W = crate::W<ShifteienSpec>;
#[doc = "Field `SEIE` reader - Shifter Error Interrupt Enable"]
pub type SeieR = crate::FieldReader;
#[doc = "Field `SEIE` writer - Shifter Error Interrupt Enable"]
pub type SeieW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Shifter Error Interrupt Enable"]
    #[inline(always)]
    pub fn seie(&self) -> SeieR {
        SeieR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Shifter Error Interrupt Enable"]
    #[inline(always)]
    pub fn seie(&mut self) -> SeieW<ShifteienSpec> {
        SeieW::new(self, 0)
    }
}
#[doc = "Shifter Error Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shifteien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shifteien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShifteienSpec;
impl crate::RegisterSpec for ShifteienSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shifteien::R`](R) reader structure"]
impl crate::Readable for ShifteienSpec {}
#[doc = "`write(|w| ..)` method takes [`shifteien::W`](W) writer structure"]
impl crate::Writable for ShifteienSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTEIEN to value 0"]
impl crate::Resettable for ShifteienSpec {}
