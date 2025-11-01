#[doc = "Register `SINTCLR` reader"]
pub type R = crate::R<SintclrSpec>;
#[doc = "Register `SINTCLR` writer"]
pub type W = crate::W<SintclrSpec>;
#[doc = "Field `START` reader - START Interrupt Enable Clear Flag"]
pub type StartR = crate::BitReader;
#[doc = "Field `START` writer - START Interrupt Enable Clear Flag"]
pub type StartW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `MATCHED` reader - Matched Interrupt Enable Clear Flag"]
pub type MatchedR = crate::BitReader;
#[doc = "Field `MATCHED` writer - Matched Interrupt Enable Clear Flag"]
pub type MatchedW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `STOP` reader - STOP Interrupt Enable Clear Flag"]
pub type StopR = crate::BitReader;
#[doc = "Field `STOP` writer - STOP Interrupt Enable Clear Flag"]
pub type StopW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `RXPEND` reader - RXPEND Interrupt Enable Clear Flag"]
pub type RxpendR = crate::BitReader;
#[doc = "Field `RXPEND` writer - RXPEND Interrupt Enable Clear Flag"]
pub type RxpendW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `TXSEND` reader - TXSEND Interrupt Enable Clear Flag"]
pub type TxsendR = crate::BitReader;
#[doc = "Field `TXSEND` writer - TXSEND Interrupt Enable Clear Flag"]
pub type TxsendW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `DACHG` reader - DACHG Interrupt Enable Clear Flag"]
pub type DachgR = crate::BitReader;
#[doc = "Field `DACHG` writer - DACHG Interrupt Enable Clear Flag"]
pub type DachgW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `CCC` reader - CCC Interrupt Enable Clear Flag"]
pub type CccR = crate::BitReader;
#[doc = "Field `CCC` writer - CCC Interrupt Enable Clear Flag"]
pub type CccW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `ERRWARN` reader - ERRWARN Interrupt Enable Clear Flag"]
pub type ErrwarnR = crate::BitReader;
#[doc = "Field `ERRWARN` writer - ERRWARN Interrupt Enable Clear Flag"]
pub type ErrwarnW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `DDRMATCHED` reader - DDRMATCHED Interrupt Enable Clear Flag"]
pub type DdrmatchedR = crate::BitReader;
#[doc = "Field `DDRMATCHED` writer - DDRMATCHED Interrupt Enable Clear Flag"]
pub type DdrmatchedW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `CHANDLED` reader - CHANDLED Interrupt Enable Clear Flag"]
pub type ChandledR = crate::BitReader;
#[doc = "Field `CHANDLED` writer - CHANDLED Interrupt Enable Clear Flag"]
pub type ChandledW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `EVENT` reader - EVENT Interrupt Enable Clear Flag"]
pub type EventR = crate::BitReader;
#[doc = "Field `EVENT` writer - EVENT Interrupt Enable Clear Flag"]
pub type EventW<'a, REG> = crate::BitWriter1C<'a, REG>;
impl R {
    #[doc = "Bit 8 - START Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Matched Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn matched(&self) -> MatchedR {
        MatchedR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - STOP Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn stop(&self) -> StopR {
        StopR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TXSEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn txsend(&self) -> TxsendR {
        TxsendR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - DACHG Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn dachg(&self) -> DachgR {
        DachgR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - CCC Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ccc(&self) -> CccR {
        CccR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - DDRMATCHED Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ddrmatched(&self) -> DdrmatchedR {
        DdrmatchedR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - CHANDLED Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn chandled(&self) -> ChandledR {
        ChandledR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - EVENT Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn event(&self) -> EventR {
        EventR::new(((self.bits >> 18) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - START Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<SintclrSpec> {
        StartW::new(self, 8)
    }
    #[doc = "Bit 9 - Matched Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn matched(&mut self) -> MatchedW<SintclrSpec> {
        MatchedW::new(self, 9)
    }
    #[doc = "Bit 10 - STOP Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn stop(&mut self) -> StopW<SintclrSpec> {
        StopW::new(self, 10)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn rxpend(&mut self) -> RxpendW<SintclrSpec> {
        RxpendW::new(self, 11)
    }
    #[doc = "Bit 12 - TXSEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn txsend(&mut self) -> TxsendW<SintclrSpec> {
        TxsendW::new(self, 12)
    }
    #[doc = "Bit 13 - DACHG Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn dachg(&mut self) -> DachgW<SintclrSpec> {
        DachgW::new(self, 13)
    }
    #[doc = "Bit 14 - CCC Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ccc(&mut self) -> CccW<SintclrSpec> {
        CccW::new(self, 14)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn errwarn(&mut self) -> ErrwarnW<SintclrSpec> {
        ErrwarnW::new(self, 15)
    }
    #[doc = "Bit 16 - DDRMATCHED Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ddrmatched(&mut self) -> DdrmatchedW<SintclrSpec> {
        DdrmatchedW::new(self, 16)
    }
    #[doc = "Bit 17 - CHANDLED Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn chandled(&mut self) -> ChandledW<SintclrSpec> {
        ChandledW::new(self, 17)
    }
    #[doc = "Bit 18 - EVENT Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn event(&mut self) -> EventW<SintclrSpec> {
        EventW::new(self, 18)
    }
}
#[doc = "Target Interrupt Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`sintclr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sintclr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SintclrSpec;
impl crate::RegisterSpec for SintclrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sintclr::R`](R) reader structure"]
impl crate::Readable for SintclrSpec {}
#[doc = "`write(|w| ..)` method takes [`sintclr::W`](W) writer structure"]
impl crate::Writable for SintclrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0007_ff00;
}
#[doc = "`reset()` method sets SINTCLR to value 0"]
impl crate::Resettable for SintclrSpec {}
