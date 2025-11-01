#[doc = "Register `CH_CSR` reader"]
pub type R = crate::R<ChCsrSpec>;
#[doc = "Register `CH_CSR` writer"]
pub type W = crate::W<ChCsrSpec>;
#[doc = "Enable DMA Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erq {
    #[doc = "0: DMA hardware request signal for corresponding channel disabled"]
    Disable = 0,
    #[doc = "1: DMA hardware request signal for corresponding channel enabled"]
    Enable = 1,
}
impl From<Erq> for bool {
    #[inline(always)]
    fn from(variant: Erq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERQ` reader - Enable DMA Request"]
pub type ErqR = crate::BitReader<Erq>;
impl ErqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erq {
        match self.bits {
            false => Erq::Disable,
            true => Erq::Enable,
        }
    }
    #[doc = "DMA hardware request signal for corresponding channel disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erq::Disable
    }
    #[doc = "DMA hardware request signal for corresponding channel enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erq::Enable
    }
}
#[doc = "Field `ERQ` writer - Enable DMA Request"]
pub type ErqW<'a, REG> = crate::BitWriter<'a, REG, Erq>;
impl<'a, REG> ErqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DMA hardware request signal for corresponding channel disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erq::Disable)
    }
    #[doc = "DMA hardware request signal for corresponding channel enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erq::Enable)
    }
}
#[doc = "Enable Asynchronous DMA Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Earq {
    #[doc = "0: Disable asynchronous DMA request for the channel"]
    Disable = 0,
    #[doc = "1: Enable asynchronous DMA request for the channel"]
    Enable = 1,
}
impl From<Earq> for bool {
    #[inline(always)]
    fn from(variant: Earq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EARQ` reader - Enable Asynchronous DMA Request"]
pub type EarqR = crate::BitReader<Earq>;
impl EarqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Earq {
        match self.bits {
            false => Earq::Disable,
            true => Earq::Enable,
        }
    }
    #[doc = "Disable asynchronous DMA request for the channel"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Earq::Disable
    }
    #[doc = "Enable asynchronous DMA request for the channel"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Earq::Enable
    }
}
#[doc = "Field `EARQ` writer - Enable Asynchronous DMA Request"]
pub type EarqW<'a, REG> = crate::BitWriter<'a, REG, Earq>;
impl<'a, REG> EarqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable asynchronous DMA request for the channel"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Earq::Disable)
    }
    #[doc = "Enable asynchronous DMA request for the channel"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Earq::Enable)
    }
}
#[doc = "Enable Error Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eei {
    #[doc = "0: Error signal for corresponding channel does not generate error interrupt"]
    NoError = 0,
    #[doc = "1: Assertion of error signal for corresponding channel generates error interrupt request"]
    Error = 1,
}
impl From<Eei> for bool {
    #[inline(always)]
    fn from(variant: Eei) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EEI` reader - Enable Error Interrupt"]
pub type EeiR = crate::BitReader<Eei>;
impl EeiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Eei {
        match self.bits {
            false => Eei::NoError,
            true => Eei::Error,
        }
    }
    #[doc = "Error signal for corresponding channel does not generate error interrupt"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Eei::NoError
    }
    #[doc = "Assertion of error signal for corresponding channel generates error interrupt request"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Eei::Error
    }
}
#[doc = "Field `EEI` writer - Enable Error Interrupt"]
pub type EeiW<'a, REG> = crate::BitWriter<'a, REG, Eei>;
impl<'a, REG> EeiW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error signal for corresponding channel does not generate error interrupt"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Eei::NoError)
    }
    #[doc = "Assertion of error signal for corresponding channel generates error interrupt request"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Eei::Error)
    }
}
#[doc = "Enable Buffered Writes\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ebw {
    #[doc = "0: Buffered writes on system bus disabled"]
    Disable = 0,
    #[doc = "1: Buffered writes on system bus enabled"]
    Enable = 1,
}
impl From<Ebw> for bool {
    #[inline(always)]
    fn from(variant: Ebw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EBW` reader - Enable Buffered Writes"]
pub type EbwR = crate::BitReader<Ebw>;
impl EbwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ebw {
        match self.bits {
            false => Ebw::Disable,
            true => Ebw::Enable,
        }
    }
    #[doc = "Buffered writes on system bus disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ebw::Disable
    }
    #[doc = "Buffered writes on system bus enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ebw::Enable
    }
}
#[doc = "Field `EBW` writer - Enable Buffered Writes"]
pub type EbwW<'a, REG> = crate::BitWriter<'a, REG, Ebw>;
impl<'a, REG> EbwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Buffered writes on system bus disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ebw::Disable)
    }
    #[doc = "Buffered writes on system bus enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ebw::Enable)
    }
}
#[doc = "Field `DONE` reader - Channel Done"]
pub type DoneR = crate::BitReader;
#[doc = "Field `DONE` writer - Channel Done"]
pub type DoneW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `ACTIVE` reader - Channel Active"]
pub type ActiveR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Enable DMA Request"]
    #[inline(always)]
    pub fn erq(&self) -> ErqR {
        ErqR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enable Asynchronous DMA Request"]
    #[inline(always)]
    pub fn earq(&self) -> EarqR {
        EarqR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Error Interrupt"]
    #[inline(always)]
    pub fn eei(&self) -> EeiR {
        EeiR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enable Buffered Writes"]
    #[inline(always)]
    pub fn ebw(&self) -> EbwR {
        EbwR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 30 - Channel Done"]
    #[inline(always)]
    pub fn done(&self) -> DoneR {
        DoneR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Channel Active"]
    #[inline(always)]
    pub fn active(&self) -> ActiveR {
        ActiveR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable DMA Request"]
    #[inline(always)]
    pub fn erq(&mut self) -> ErqW<ChCsrSpec> {
        ErqW::new(self, 0)
    }
    #[doc = "Bit 1 - Enable Asynchronous DMA Request"]
    #[inline(always)]
    pub fn earq(&mut self) -> EarqW<ChCsrSpec> {
        EarqW::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Error Interrupt"]
    #[inline(always)]
    pub fn eei(&mut self) -> EeiW<ChCsrSpec> {
        EeiW::new(self, 2)
    }
    #[doc = "Bit 3 - Enable Buffered Writes"]
    #[inline(always)]
    pub fn ebw(&mut self) -> EbwW<ChCsrSpec> {
        EbwW::new(self, 3)
    }
    #[doc = "Bit 30 - Channel Done"]
    #[inline(always)]
    pub fn done(&mut self) -> DoneW<ChCsrSpec> {
        DoneW::new(self, 30)
    }
}
#[doc = "Channel Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_csr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_csr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChCsrSpec;
impl crate::RegisterSpec for ChCsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_csr::R`](R) reader structure"]
impl crate::Readable for ChCsrSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_csr::W`](W) writer structure"]
impl crate::Writable for ChCsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x4000_0000;
}
#[doc = "`reset()` method sets CH_CSR to value 0"]
impl crate::Resettable for ChCsrSpec {}
