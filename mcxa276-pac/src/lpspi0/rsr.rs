#[doc = "Register `RSR` reader"]
pub type R = crate::R<RsrSpec>;
#[doc = "Start of Frame\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sof {
    #[doc = "0: Subsequent data word or RX FIFO is empty (RXEMPTY=1)."]
    NextDataword = 0,
    #[doc = "1: First data word"]
    FirstDataword = 1,
}
impl From<Sof> for bool {
    #[inline(always)]
    fn from(variant: Sof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOF` reader - Start of Frame"]
pub type SofR = crate::BitReader<Sof>;
impl SofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sof {
        match self.bits {
            false => Sof::NextDataword,
            true => Sof::FirstDataword,
        }
    }
    #[doc = "Subsequent data word or RX FIFO is empty (RXEMPTY=1)."]
    #[inline(always)]
    pub fn is_next_dataword(&self) -> bool {
        *self == Sof::NextDataword
    }
    #[doc = "First data word"]
    #[inline(always)]
    pub fn is_first_dataword(&self) -> bool {
        *self == Sof::FirstDataword
    }
}
#[doc = "RX FIFO Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxempty {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Rxempty> for bool {
    #[inline(always)]
    fn from(variant: Rxempty) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEMPTY` reader - RX FIFO Empty"]
pub type RxemptyR = crate::BitReader<Rxempty>;
impl RxemptyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxempty {
        match self.bits {
            false => Rxempty::NotEmpty,
            true => Rxempty::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Rxempty::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Rxempty::Empty
    }
}
impl R {
    #[doc = "Bit 0 - Start of Frame"]
    #[inline(always)]
    pub fn sof(&self) -> SofR {
        SofR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RX FIFO Empty"]
    #[inline(always)]
    pub fn rxempty(&self) -> RxemptyR {
        RxemptyR::new(((self.bits >> 1) & 1) != 0)
    }
}
#[doc = "Receive Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rsr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RsrSpec;
impl crate::RegisterSpec for RsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rsr::R`](R) reader structure"]
impl crate::Readable for RsrSpec {}
#[doc = "`reset()` method sets RSR to value 0x02"]
impl crate::Resettable for RsrSpec {
    const RESET_VALUE: u32 = 0x02;
}
