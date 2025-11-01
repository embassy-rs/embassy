#[doc = "Register `DMR1` reader"]
pub type R = crate::R<Dmr1Spec>;
#[doc = "Register `DMR1` writer"]
pub type W = crate::W<Dmr1Spec>;
#[doc = "Field `MATCH1` reader - Match 1 Value"]
pub type Match1R = crate::FieldReader<u32>;
#[doc = "Field `MATCH1` writer - Match 1 Value"]
pub type Match1W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Match 1 Value"]
    #[inline(always)]
    pub fn match1(&self) -> Match1R {
        Match1R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Match 1 Value"]
    #[inline(always)]
    pub fn match1(&mut self) -> Match1W<Dmr1Spec> {
        Match1W::new(self, 0)
    }
}
#[doc = "Data Match 1\n\nYou can [`read`](crate::Reg::read) this register and get [`dmr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Dmr1Spec;
impl crate::RegisterSpec for Dmr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dmr1::R`](R) reader structure"]
impl crate::Readable for Dmr1Spec {}
#[doc = "`write(|w| ..)` method takes [`dmr1::W`](W) writer structure"]
impl crate::Writable for Dmr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DMR1 to value 0"]
impl crate::Resettable for Dmr1Spec {}
