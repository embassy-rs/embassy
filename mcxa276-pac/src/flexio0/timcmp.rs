#[doc = "Register `TIMCMP[%s]` reader"]
pub type R = crate::R<TimcmpSpec>;
#[doc = "Register `TIMCMP[%s]` writer"]
pub type W = crate::W<TimcmpSpec>;
#[doc = "Field `CMP` reader - Timer Compare Value"]
pub type CmpR = crate::FieldReader<u16>;
#[doc = "Field `CMP` writer - Timer Compare Value"]
pub type CmpW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Timer Compare Value"]
    #[inline(always)]
    pub fn cmp(&self) -> CmpR {
        CmpR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Timer Compare Value"]
    #[inline(always)]
    pub fn cmp(&mut self) -> CmpW<TimcmpSpec> {
        CmpW::new(self, 0)
    }
}
#[doc = "Timer Compare\n\nYou can [`read`](crate::Reg::read) this register and get [`timcmp::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timcmp::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimcmpSpec;
impl crate::RegisterSpec for TimcmpSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timcmp::R`](R) reader structure"]
impl crate::Readable for TimcmpSpec {}
#[doc = "`write(|w| ..)` method takes [`timcmp::W`](W) writer structure"]
impl crate::Writable for TimcmpSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMCMP[%s] to value 0"]
impl crate::Resettable for TimcmpSpec {}
