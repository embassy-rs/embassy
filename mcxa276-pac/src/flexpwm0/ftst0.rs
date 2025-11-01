#[doc = "Register `FTST0` reader"]
pub type R = crate::R<Ftst0Spec>;
#[doc = "Register `FTST0` writer"]
pub type W = crate::W<Ftst0Spec>;
#[doc = "Fault Test\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ftest {
    #[doc = "0: No fault"]
    NoFault = 0,
    #[doc = "1: Cause a simulated fault"]
    Fault = 1,
}
impl From<Ftest> for bool {
    #[inline(always)]
    fn from(variant: Ftest) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FTEST` reader - Fault Test"]
pub type FtestR = crate::BitReader<Ftest>;
impl FtestR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ftest {
        match self.bits {
            false => Ftest::NoFault,
            true => Ftest::Fault,
        }
    }
    #[doc = "No fault"]
    #[inline(always)]
    pub fn is_no_fault(&self) -> bool {
        *self == Ftest::NoFault
    }
    #[doc = "Cause a simulated fault"]
    #[inline(always)]
    pub fn is_fault(&self) -> bool {
        *self == Ftest::Fault
    }
}
#[doc = "Field `FTEST` writer - Fault Test"]
pub type FtestW<'a, REG> = crate::BitWriter<'a, REG, Ftest>;
impl<'a, REG> FtestW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No fault"]
    #[inline(always)]
    pub fn no_fault(self) -> &'a mut crate::W<REG> {
        self.variant(Ftest::NoFault)
    }
    #[doc = "Cause a simulated fault"]
    #[inline(always)]
    pub fn fault(self) -> &'a mut crate::W<REG> {
        self.variant(Ftest::Fault)
    }
}
impl R {
    #[doc = "Bit 0 - Fault Test"]
    #[inline(always)]
    pub fn ftest(&self) -> FtestR {
        FtestR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Fault Test"]
    #[inline(always)]
    pub fn ftest(&mut self) -> FtestW<Ftst0Spec> {
        FtestW::new(self, 0)
    }
}
#[doc = "Fault Test Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ftst0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ftst0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ftst0Spec;
impl crate::RegisterSpec for Ftst0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`ftst0::R`](R) reader structure"]
impl crate::Readable for Ftst0Spec {}
#[doc = "`write(|w| ..)` method takes [`ftst0::W`](W) writer structure"]
impl crate::Writable for Ftst0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FTST0 to value 0"]
impl crate::Resettable for Ftst0Spec {}
