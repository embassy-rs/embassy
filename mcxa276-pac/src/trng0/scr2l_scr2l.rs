#[doc = "Register `SCR2L` reader"]
pub type R = crate::R<Scr2lScr2lSpec>;
#[doc = "Register `SCR2L` writer"]
pub type W = crate::W<Scr2lScr2lSpec>;
#[doc = "Field `RUN2_MAX` reader - Run Length 2 Maximum Limit"]
pub type Run2MaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN2_MAX` writer - Run Length 2 Maximum Limit"]
pub type Run2MaxW<'a, REG> = crate::FieldWriter<'a, REG, 14, u16>;
#[doc = "Field `RUN2_RNG` reader - Run Length 2 Range"]
pub type Run2RngR = crate::FieldReader<u16>;
#[doc = "Field `RUN2_RNG` writer - Run Length 2 Range"]
pub type Run2RngW<'a, REG> = crate::FieldWriter<'a, REG, 14, u16>;
impl R {
    #[doc = "Bits 0:13 - Run Length 2 Maximum Limit"]
    #[inline(always)]
    pub fn run2_max(&self) -> Run2MaxR {
        Run2MaxR::new((self.bits & 0x3fff) as u16)
    }
    #[doc = "Bits 16:29 - Run Length 2 Range"]
    #[inline(always)]
    pub fn run2_rng(&self) -> Run2RngR {
        Run2RngR::new(((self.bits >> 16) & 0x3fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:13 - Run Length 2 Maximum Limit"]
    #[inline(always)]
    pub fn run2_max(&mut self) -> Run2MaxW<Scr2lScr2lSpec> {
        Run2MaxW::new(self, 0)
    }
    #[doc = "Bits 16:29 - Run Length 2 Range"]
    #[inline(always)]
    pub fn run2_rng(&mut self) -> Run2RngW<Scr2lScr2lSpec> {
        Run2RngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 2 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr2l_scr2l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr2l_scr2l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr2lScr2lSpec;
impl crate::RegisterSpec for Scr2lScr2lSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr2l_scr2l::R`](R) reader structure"]
impl crate::Readable for Scr2lScr2lSpec {}
#[doc = "`write(|w| ..)` method takes [`scr2l_scr2l::W`](W) writer structure"]
impl crate::Writable for Scr2lScr2lSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR2L to value 0x0036_003e"]
impl crate::Resettable for Scr2lScr2lSpec {
    const RESET_VALUE: u32 = 0x0036_003e;
}
