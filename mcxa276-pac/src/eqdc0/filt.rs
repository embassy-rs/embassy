#[doc = "Register `FILT` reader"]
pub type R = crate::R<FiltSpec>;
#[doc = "Register `FILT` writer"]
pub type W = crate::W<FiltSpec>;
#[doc = "Field `FILT_PER` reader - Input Filter Sample Period"]
pub type FiltPerR = crate::FieldReader;
#[doc = "Field `FILT_PER` writer - Input Filter Sample Period"]
pub type FiltPerW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `FILT_CNT` reader - Input Filter Sample Count"]
pub type FiltCntR = crate::FieldReader;
#[doc = "Field `FILT_CNT` writer - Input Filter Sample Count"]
pub type FiltCntW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Filter Clock Source selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FiltCs {
    #[doc = "0: Peripheral Clock"]
    FiltCs0 = 0,
    #[doc = "1: Prescaled peripheral clock by PRSC"]
    FiltCs1 = 1,
}
impl From<FiltCs> for bool {
    #[inline(always)]
    fn from(variant: FiltCs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILT_CS` reader - Filter Clock Source selection"]
pub type FiltCsR = crate::BitReader<FiltCs>;
impl FiltCsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FiltCs {
        match self.bits {
            false => FiltCs::FiltCs0,
            true => FiltCs::FiltCs1,
        }
    }
    #[doc = "Peripheral Clock"]
    #[inline(always)]
    pub fn is_filt_cs0(&self) -> bool {
        *self == FiltCs::FiltCs0
    }
    #[doc = "Prescaled peripheral clock by PRSC"]
    #[inline(always)]
    pub fn is_filt_cs1(&self) -> bool {
        *self == FiltCs::FiltCs1
    }
}
#[doc = "Field `FILT_CS` writer - Filter Clock Source selection"]
pub type FiltCsW<'a, REG> = crate::BitWriter<'a, REG, FiltCs>;
impl<'a, REG> FiltCsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral Clock"]
    #[inline(always)]
    pub fn filt_cs0(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCs::FiltCs0)
    }
    #[doc = "Prescaled peripheral clock by PRSC"]
    #[inline(always)]
    pub fn filt_cs1(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCs::FiltCs1)
    }
}
#[doc = "Field `PRSC` reader - Prescaler"]
pub type PrscR = crate::FieldReader;
#[doc = "Field `PRSC` writer - Prescaler"]
pub type PrscW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:7 - Input Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&self) -> FiltPerR {
        FiltPerR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:10 - Input Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&self) -> FiltCntR {
        FiltCntR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 11 - Filter Clock Source selection"]
    #[inline(always)]
    pub fn filt_cs(&self) -> FiltCsR {
        FiltCsR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:15 - Prescaler"]
    #[inline(always)]
    pub fn prsc(&self) -> PrscR {
        PrscR::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Input Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&mut self) -> FiltPerW<FiltSpec> {
        FiltPerW::new(self, 0)
    }
    #[doc = "Bits 8:10 - Input Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&mut self) -> FiltCntW<FiltSpec> {
        FiltCntW::new(self, 8)
    }
    #[doc = "Bit 11 - Filter Clock Source selection"]
    #[inline(always)]
    pub fn filt_cs(&mut self) -> FiltCsW<FiltSpec> {
        FiltCsW::new(self, 11)
    }
    #[doc = "Bits 12:15 - Prescaler"]
    #[inline(always)]
    pub fn prsc(&mut self) -> PrscW<FiltSpec> {
        PrscW::new(self, 12)
    }
}
#[doc = "Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`filt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`filt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FiltSpec;
impl crate::RegisterSpec for FiltSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`filt::R`](R) reader structure"]
impl crate::Readable for FiltSpec {}
#[doc = "`write(|w| ..)` method takes [`filt::W`](W) writer structure"]
impl crate::Writable for FiltSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FILT to value 0"]
impl crate::Resettable for FiltSpec {}
