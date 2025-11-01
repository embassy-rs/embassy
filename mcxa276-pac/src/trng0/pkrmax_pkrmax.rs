#[doc = "Register `PKRMAX` reader"]
pub type R = crate::R<PkrmaxPkrmaxSpec>;
#[doc = "Register `PKRMAX` writer"]
pub type W = crate::W<PkrmaxPkrmaxSpec>;
#[doc = "Field `PKR_MAX` reader - Poker Maximum Limit."]
pub type PkrMaxR = crate::FieldReader<u32>;
#[doc = "Field `PKR_MAX` writer - Poker Maximum Limit."]
pub type PkrMaxW<'a, REG> = crate::FieldWriter<'a, REG, 24, u32>;
impl R {
    #[doc = "Bits 0:23 - Poker Maximum Limit."]
    #[inline(always)]
    pub fn pkr_max(&self) -> PkrMaxR {
        PkrMaxR::new(self.bits & 0x00ff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:23 - Poker Maximum Limit."]
    #[inline(always)]
    pub fn pkr_max(&mut self) -> PkrMaxW<PkrmaxPkrmaxSpec> {
        PkrMaxW::new(self, 0)
    }
}
#[doc = "Poker Maximum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrmax_pkrmax::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkrmax_pkrmax::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkrmaxPkrmaxSpec;
impl crate::RegisterSpec for PkrmaxPkrmaxSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrmax_pkrmax::R`](R) reader structure"]
impl crate::Readable for PkrmaxPkrmaxSpec {}
#[doc = "`write(|w| ..)` method takes [`pkrmax_pkrmax::W`](W) writer structure"]
impl crate::Writable for PkrmaxPkrmaxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKRMAX to value 0x0640"]
impl crate::Resettable for PkrmaxPkrmaxSpec {
    const RESET_VALUE: u32 = 0x0640;
}
