#[doc = "Register `TRIGIEN` reader"]
pub type R = crate::R<TrigienSpec>;
#[doc = "Register `TRIGIEN` writer"]
pub type W = crate::W<TrigienSpec>;
#[doc = "Field `TRIE` reader - External Trigger Interrupt Enable"]
pub type TrieR = crate::FieldReader;
#[doc = "Field `TRIE` writer - External Trigger Interrupt Enable"]
pub type TrieW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - External Trigger Interrupt Enable"]
    #[inline(always)]
    pub fn trie(&self) -> TrieR {
        TrieR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - External Trigger Interrupt Enable"]
    #[inline(always)]
    pub fn trie(&mut self) -> TrieW<TrigienSpec> {
        TrieW::new(self, 0)
    }
}
#[doc = "External Trigger Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`trigien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrigienSpec;
impl crate::RegisterSpec for TrigienSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trigien::R`](R) reader structure"]
impl crate::Readable for TrigienSpec {}
#[doc = "`write(|w| ..)` method takes [`trigien::W`](W) writer structure"]
impl crate::Writable for TrigienSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TRIGIEN to value 0"]
impl crate::Resettable for TrigienSpec {}
