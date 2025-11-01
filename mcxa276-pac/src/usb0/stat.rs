#[doc = "Register `STAT` reader"]
pub type R = crate::R<StatSpec>;
#[doc = "Odd Bank\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Odd {
    #[doc = "0: Not in the odd bank"]
    NotInOddBank = 0,
    #[doc = "1: In the odd bank"]
    OddBank = 1,
}
impl From<Odd> for bool {
    #[inline(always)]
    fn from(variant: Odd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ODD` reader - Odd Bank"]
pub type OddR = crate::BitReader<Odd>;
impl OddR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Odd {
        match self.bits {
            false => Odd::NotInOddBank,
            true => Odd::OddBank,
        }
    }
    #[doc = "Not in the odd bank"]
    #[inline(always)]
    pub fn is_not_in_odd_bank(&self) -> bool {
        *self == Odd::NotInOddBank
    }
    #[doc = "In the odd bank"]
    #[inline(always)]
    pub fn is_odd_bank(&self) -> bool {
        *self == Odd::OddBank
    }
}
#[doc = "Transmit Indicator\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tx {
    #[doc = "0: Receive"]
    RxTransaction = 0,
    #[doc = "1: Transmit"]
    TxTransaction = 1,
}
impl From<Tx> for bool {
    #[inline(always)]
    fn from(variant: Tx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TX` reader - Transmit Indicator"]
pub type TxR = crate::BitReader<Tx>;
impl TxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tx {
        match self.bits {
            false => Tx::RxTransaction,
            true => Tx::TxTransaction,
        }
    }
    #[doc = "Receive"]
    #[inline(always)]
    pub fn is_rx_transaction(&self) -> bool {
        *self == Tx::RxTransaction
    }
    #[doc = "Transmit"]
    #[inline(always)]
    pub fn is_tx_transaction(&self) -> bool {
        *self == Tx::TxTransaction
    }
}
#[doc = "Field `ENDP` reader - Endpoint address"]
pub type EndpR = crate::FieldReader;
impl R {
    #[doc = "Bit 2 - Odd Bank"]
    #[inline(always)]
    pub fn odd(&self) -> OddR {
        OddR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Transmit Indicator"]
    #[inline(always)]
    pub fn tx(&self) -> TxR {
        TxR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:7 - Endpoint address"]
    #[inline(always)]
    pub fn endp(&self) -> EndpR {
        EndpR::new((self.bits >> 4) & 0x0f)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatSpec;
impl crate::RegisterSpec for StatSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`stat::R`](R) reader structure"]
impl crate::Readable for StatSpec {}
#[doc = "`reset()` method sets STAT to value 0"]
impl crate::Resettable for StatSpec {}
