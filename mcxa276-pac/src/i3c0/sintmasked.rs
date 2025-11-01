#[doc = "Register `SINTMASKED` reader"]
pub type R = crate::R<SintmaskedSpec>;
#[doc = "Field `START` reader - START Interrupt Mask"]
pub type StartR = crate::BitReader;
#[doc = "Field `MATCHED` reader - MATCHED Interrupt Mask"]
pub type MatchedR = crate::BitReader;
#[doc = "Field `STOP` reader - STOP Interrupt Mask"]
pub type StopR = crate::BitReader;
#[doc = "Field `RXPEND` reader - RXPEND Interrupt Mask"]
pub type RxpendR = crate::BitReader;
#[doc = "Field `TXSEND` reader - TXSEND Interrupt Mask"]
pub type TxsendR = crate::BitReader;
#[doc = "Field `DACHG` reader - DACHG Interrupt Mask"]
pub type DachgR = crate::BitReader;
#[doc = "Field `CCC` reader - CCC Interrupt Mask"]
pub type CccR = crate::BitReader;
#[doc = "Field `ERRWARN` reader - ERRWARN Interrupt Mask"]
pub type ErrwarnR = crate::BitReader;
#[doc = "Field `DDRMATCHED` reader - DDRMATCHED Interrupt Mask"]
pub type DdrmatchedR = crate::BitReader;
#[doc = "Field `CHANDLED` reader - CHANDLED Interrupt Mask"]
pub type ChandledR = crate::BitReader;
#[doc = "Field `EVENT` reader - EVENT Interrupt Mask"]
pub type EventR = crate::BitReader;
impl R {
    #[doc = "Bit 8 - START Interrupt Mask"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - MATCHED Interrupt Mask"]
    #[inline(always)]
    pub fn matched(&self) -> MatchedR {
        MatchedR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - STOP Interrupt Mask"]
    #[inline(always)]
    pub fn stop(&self) -> StopR {
        StopR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Mask"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TXSEND Interrupt Mask"]
    #[inline(always)]
    pub fn txsend(&self) -> TxsendR {
        TxsendR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - DACHG Interrupt Mask"]
    #[inline(always)]
    pub fn dachg(&self) -> DachgR {
        DachgR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - CCC Interrupt Mask"]
    #[inline(always)]
    pub fn ccc(&self) -> CccR {
        CccR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Mask"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - DDRMATCHED Interrupt Mask"]
    #[inline(always)]
    pub fn ddrmatched(&self) -> DdrmatchedR {
        DdrmatchedR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - CHANDLED Interrupt Mask"]
    #[inline(always)]
    pub fn chandled(&self) -> ChandledR {
        ChandledR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - EVENT Interrupt Mask"]
    #[inline(always)]
    pub fn event(&self) -> EventR {
        EventR::new(((self.bits >> 18) & 1) != 0)
    }
}
#[doc = "Target Interrupt Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`sintmasked::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SintmaskedSpec;
impl crate::RegisterSpec for SintmaskedSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sintmasked::R`](R) reader structure"]
impl crate::Readable for SintmaskedSpec {}
#[doc = "`reset()` method sets SINTMASKED to value 0"]
impl crate::Resettable for SintmaskedSpec {}
