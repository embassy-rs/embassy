#[doc = "Register `SHIFTSIEN` reader"]
pub type R = crate::R<ShiftsienSpec>;
#[doc = "Register `SHIFTSIEN` writer"]
pub type W = crate::W<ShiftsienSpec>;
#[doc = "Field `SSIE` reader - Shifter Status Interrupt Enable"]
pub type SsieR = crate::FieldReader;
#[doc = "Field `SSIE` writer - Shifter Status Interrupt Enable"]
pub type SsieW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Shifter Status Interrupt Enable"]
    #[inline(always)]
    pub fn ssie(&self) -> SsieR {
        SsieR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Shifter Status Interrupt Enable"]
    #[inline(always)]
    pub fn ssie(&mut self) -> SsieW<ShiftsienSpec> {
        SsieW::new(self, 0)
    }
}
#[doc = "Shifter Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftsien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftsien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftsienSpec;
impl crate::RegisterSpec for ShiftsienSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftsien::R`](R) reader structure"]
impl crate::Readable for ShiftsienSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftsien::W`](W) writer structure"]
impl crate::Writable for ShiftsienSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTSIEN to value 0"]
impl crate::Resettable for ShiftsienSpec {}
