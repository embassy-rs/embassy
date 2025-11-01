#[doc = "Register `TCD_BITER_ELINKNO` reader"]
pub type R = crate::R<ElinknoTcdBiterElinknoSpec>;
#[doc = "Register `TCD_BITER_ELINKNO` writer"]
pub type W = crate::W<ElinknoTcdBiterElinknoSpec>;
#[doc = "Field `BITER` reader - Starting Major Iteration Count"]
pub type BiterR = crate::FieldReader<u16>;
#[doc = "Field `BITER` writer - Starting Major Iteration Count"]
pub type BiterW<'a, REG> = crate::FieldWriter<'a, REG, 15, u16>;
#[doc = "Enables Link\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elink {
    #[doc = "0: Channel-to-channel linking disabled"]
    Disable = 0,
    #[doc = "1: Channel-to-channel linking enabled"]
    Enable = 1,
}
impl From<Elink> for bool {
    #[inline(always)]
    fn from(variant: Elink) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ELINK` reader - Enables Link"]
pub type ElinkR = crate::BitReader<Elink>;
impl ElinkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Elink {
        match self.bits {
            false => Elink::Disable,
            true => Elink::Enable,
        }
    }
    #[doc = "Channel-to-channel linking disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Elink::Disable
    }
    #[doc = "Channel-to-channel linking enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Elink::Enable
    }
}
#[doc = "Field `ELINK` writer - Enables Link"]
pub type ElinkW<'a, REG> = crate::BitWriter<'a, REG, Elink>;
impl<'a, REG> ElinkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel-to-channel linking disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Elink::Disable)
    }
    #[doc = "Channel-to-channel linking enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Elink::Enable)
    }
}
impl R {
    #[doc = "Bits 0:14 - Starting Major Iteration Count"]
    #[inline(always)]
    pub fn biter(&self) -> BiterR {
        BiterR::new(self.bits & 0x7fff)
    }
    #[doc = "Bit 15 - Enables Link"]
    #[inline(always)]
    pub fn elink(&self) -> ElinkR {
        ElinkR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:14 - Starting Major Iteration Count"]
    #[inline(always)]
    pub fn biter(&mut self) -> BiterW<ElinknoTcdBiterElinknoSpec> {
        BiterW::new(self, 0)
    }
    #[doc = "Bit 15 - Enables Link"]
    #[inline(always)]
    pub fn elink(&mut self) -> ElinkW<ElinknoTcdBiterElinknoSpec> {
        ElinkW::new(self, 15)
    }
}
#[doc = "TCD Beginning Major Loop Count (Minor Loop Channel Linking Disabled)\n\nYou can [`read`](crate::Reg::read) this register and get [`elinkno_tcd_biter_elinkno::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`elinkno_tcd_biter_elinkno::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ElinknoTcdBiterElinknoSpec;
impl crate::RegisterSpec for ElinknoTcdBiterElinknoSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`elinkno_tcd_biter_elinkno::R`](R) reader structure"]
impl crate::Readable for ElinknoTcdBiterElinknoSpec {}
#[doc = "`write(|w| ..)` method takes [`elinkno_tcd_biter_elinkno::W`](W) writer structure"]
impl crate::Writable for ElinknoTcdBiterElinknoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_BITER_ELINKNO to value 0"]
impl crate::Resettable for ElinknoTcdBiterElinknoSpec {}
