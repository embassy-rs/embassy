#[doc = "Register `SCR6PL` reader"]
pub type R = crate::R<Scr6plScr6plSpec>;
#[doc = "Register `SCR6PL` writer"]
pub type W = crate::W<Scr6plScr6plSpec>;
#[doc = "Field `RUN6P_MAX` reader - Run Length 6+ Maximum Limit"]
pub type Run6pMaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN6P_MAX` writer - Run Length 6+ Maximum Limit"]
pub type Run6pMaxW<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
#[doc = "Field `RUN6P_RNG` reader - Run Length 6+ Range"]
pub type Run6pRngR = crate::FieldReader<u16>;
#[doc = "Field `RUN6P_RNG` writer - Run Length 6+ Range"]
pub type Run6pRngW<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
impl R {
    #[doc = "Bits 0:10 - Run Length 6+ Maximum Limit"]
    #[inline(always)]
    pub fn run6p_max(&self) -> Run6pMaxR {
        Run6pMaxR::new((self.bits & 0x07ff) as u16)
    }
    #[doc = "Bits 16:26 - Run Length 6+ Range"]
    #[inline(always)]
    pub fn run6p_rng(&self) -> Run6pRngR {
        Run6pRngR::new(((self.bits >> 16) & 0x07ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:10 - Run Length 6+ Maximum Limit"]
    #[inline(always)]
    pub fn run6p_max(&mut self) -> Run6pMaxW<Scr6plScr6plSpec> {
        Run6pMaxW::new(self, 0)
    }
    #[doc = "Bits 16:26 - Run Length 6+ Range"]
    #[inline(always)]
    pub fn run6p_rng(&mut self) -> Run6pRngW<Scr6plScr6plSpec> {
        Run6pRngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 6+ Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr6pl_scr6pl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr6pl_scr6pl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr6plScr6plSpec;
impl crate::RegisterSpec for Scr6plScr6plSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr6pl_scr6pl::R`](R) reader structure"]
impl crate::Readable for Scr6plScr6plSpec {}
#[doc = "`write(|w| ..)` method takes [`scr6pl_scr6pl::W`](W) writer structure"]
impl crate::Writable for Scr6plScr6plSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR6PL to value 0x0012_0011"]
impl crate::Resettable for Scr6plScr6plSpec {
    const RESET_VALUE: u32 = 0x0012_0011;
}
