#[doc = "Register `DMR0` reader"]
pub type R = crate::R<Dmr0Spec>;
#[doc = "Register `DMR0` writer"]
pub type W = crate::W<Dmr0Spec>;
#[doc = "Field `MATCH0` reader - Match 0 Value"]
pub type Match0R = crate::FieldReader<u32>;
#[doc = "Field `MATCH0` writer - Match 0 Value"]
pub type Match0W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Match 0 Value"]
    #[inline(always)]
    pub fn match0(&self) -> Match0R {
        Match0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Match 0 Value"]
    #[inline(always)]
    pub fn match0(&mut self) -> Match0W<Dmr0Spec> {
        Match0W::new(self, 0)
    }
}
#[doc = "Data Match 0\n\nYou can [`read`](crate::Reg::read) this register and get [`dmr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Dmr0Spec;
impl crate::RegisterSpec for Dmr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dmr0::R`](R) reader structure"]
impl crate::Readable for Dmr0Spec {}
#[doc = "`write(|w| ..)` method takes [`dmr0::W`](W) writer structure"]
impl crate::Writable for Dmr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DMR0 to value 0"]
impl crate::Resettable for Dmr0Spec {}
