#[doc = "Register `PINCFG` reader"]
pub type R = crate::R<PincfgSpec>;
#[doc = "Register `PINCFG` writer"]
pub type W = crate::W<PincfgSpec>;
#[doc = "Trigger Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trgsel {
    #[doc = "0: Input trigger disabled"]
    Disabled = 0,
    #[doc = "1: Input trigger used instead of the RXD pin input"]
    TrgRxd = 1,
    #[doc = "2: Input trigger used instead of the CTS_B pin input"]
    TrgCts = 2,
    #[doc = "3: Input trigger used to modulate the TXD pin output, which (after TXINV configuration) is internally ANDed with the input trigger"]
    TrgTxd = 3,
}
impl From<Trgsel> for u8 {
    #[inline(always)]
    fn from(variant: Trgsel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Trgsel {
    type Ux = u8;
}
impl crate::IsEnum for Trgsel {}
#[doc = "Field `TRGSEL` reader - Trigger Select"]
pub type TrgselR = crate::FieldReader<Trgsel>;
impl TrgselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgsel {
        match self.bits {
            0 => Trgsel::Disabled,
            1 => Trgsel::TrgRxd,
            2 => Trgsel::TrgCts,
            3 => Trgsel::TrgTxd,
            _ => unreachable!(),
        }
    }
    #[doc = "Input trigger disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Trgsel::Disabled
    }
    #[doc = "Input trigger used instead of the RXD pin input"]
    #[inline(always)]
    pub fn is_trg_rxd(&self) -> bool {
        *self == Trgsel::TrgRxd
    }
    #[doc = "Input trigger used instead of the CTS_B pin input"]
    #[inline(always)]
    pub fn is_trg_cts(&self) -> bool {
        *self == Trgsel::TrgCts
    }
    #[doc = "Input trigger used to modulate the TXD pin output, which (after TXINV configuration) is internally ANDed with the input trigger"]
    #[inline(always)]
    pub fn is_trg_txd(&self) -> bool {
        *self == Trgsel::TrgTxd
    }
}
#[doc = "Field `TRGSEL` writer - Trigger Select"]
pub type TrgselW<'a, REG> = crate::FieldWriter<'a, REG, 2, Trgsel, crate::Safe>;
impl<'a, REG> TrgselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Input trigger disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::Disabled)
    }
    #[doc = "Input trigger used instead of the RXD pin input"]
    #[inline(always)]
    pub fn trg_rxd(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::TrgRxd)
    }
    #[doc = "Input trigger used instead of the CTS_B pin input"]
    #[inline(always)]
    pub fn trg_cts(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::TrgCts)
    }
    #[doc = "Input trigger used to modulate the TXD pin output, which (after TXINV configuration) is internally ANDed with the input trigger"]
    #[inline(always)]
    pub fn trg_txd(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::TrgTxd)
    }
}
impl R {
    #[doc = "Bits 0:1 - Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&self) -> TrgselR {
        TrgselR::new((self.bits & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&mut self) -> TrgselW<PincfgSpec> {
        TrgselW::new(self, 0)
    }
}
#[doc = "Pin Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`pincfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pincfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PincfgSpec;
impl crate::RegisterSpec for PincfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pincfg::R`](R) reader structure"]
impl crate::Readable for PincfgSpec {}
#[doc = "`write(|w| ..)` method takes [`pincfg::W`](W) writer structure"]
impl crate::Writable for PincfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINCFG to value 0"]
impl crate::Resettable for PincfgSpec {}
