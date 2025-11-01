#[doc = "Register `MR0` reader"]
pub type R = crate::R<Mr0Spec>;
#[doc = "Register `MR0` writer"]
pub type W = crate::W<Mr0Spec>;
#[doc = "Field `ISPMODE_n` reader - In System Programming Mode"]
pub type IspmodeNR = crate::BitReader;
#[doc = "Field `ISPMODE_n` writer - In System Programming Mode"]
pub type IspmodeNW<'a, REG> = crate::BitWriter1C<'a, REG>;
impl R {
    #[doc = "Bit 0 - In System Programming Mode"]
    #[inline(always)]
    pub fn ispmode_n(&self) -> IspmodeNR {
        IspmodeNR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - In System Programming Mode"]
    #[inline(always)]
    pub fn ispmode_n(&mut self) -> IspmodeNW<Mr0Spec> {
        IspmodeNW::new(self, 0)
    }
}
#[doc = "Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mr0Spec;
impl crate::RegisterSpec for Mr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mr0::R`](R) reader structure"]
impl crate::Readable for Mr0Spec {}
#[doc = "`write(|w| ..)` method takes [`mr0::W`](W) writer structure"]
impl crate::Writable for Mr0Spec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x01;
}
#[doc = "`reset()` method sets MR0 to value 0"]
impl crate::Resettable for Mr0Spec {}
