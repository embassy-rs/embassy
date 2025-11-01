#[doc = "Register `PDR[%s]` reader"]
pub type R = crate::R<PdrSpec>;
#[doc = "Register `PDR[%s]` writer"]
pub type W = crate::W<PdrSpec>;
#[doc = "Pin Data (I/O)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pd {
    #[doc = "0: Logic zero"]
    Pd0 = 0,
    #[doc = "1: Logic one"]
    Pd1 = 1,
}
impl From<Pd> for bool {
    #[inline(always)]
    fn from(variant: Pd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PD` reader - Pin Data (I/O)"]
pub type PdR = crate::BitReader<Pd>;
impl PdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pd {
        match self.bits {
            false => Pd::Pd0,
            true => Pd::Pd1,
        }
    }
    #[doc = "Logic zero"]
    #[inline(always)]
    pub fn is_pd0(&self) -> bool {
        *self == Pd::Pd0
    }
    #[doc = "Logic one"]
    #[inline(always)]
    pub fn is_pd1(&self) -> bool {
        *self == Pd::Pd1
    }
}
#[doc = "Field `PD` writer - Pin Data (I/O)"]
pub type PdW<'a, REG> = crate::BitWriter<'a, REG, Pd>;
impl<'a, REG> PdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic zero"]
    #[inline(always)]
    pub fn pd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pd::Pd0)
    }
    #[doc = "Logic one"]
    #[inline(always)]
    pub fn pd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pd::Pd1)
    }
}
impl R {
    #[doc = "Bit 0 - Pin Data (I/O)"]
    #[inline(always)]
    pub fn pd(&self) -> PdR {
        PdR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Pin Data (I/O)"]
    #[inline(always)]
    pub fn pd(&mut self) -> PdW<PdrSpec> {
        PdW::new(self, 0)
    }
}
#[doc = "Pin Data\n\nYou can [`read`](crate::Reg::read) this register and get [`pdr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PdrSpec;
impl crate::RegisterSpec for PdrSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`pdr::R`](R) reader structure"]
impl crate::Readable for PdrSpec {}
#[doc = "`write(|w| ..)` method takes [`pdr::W`](W) writer structure"]
impl crate::Writable for PdrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PDR[%s] to value 0"]
impl crate::Resettable for PdrSpec {}
