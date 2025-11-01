#[doc = "Register `PARAM` reader"]
pub type R = crate::R<ParamSpec>;
#[doc = "Field `TXFIFO` reader - Transmit FIFO Size"]
pub type TxfifoR = crate::FieldReader;
#[doc = "Field `RXFIFO` reader - Receive FIFO Size"]
pub type RxfifoR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Transmit FIFO Size"]
    #[inline(always)]
    pub fn txfifo(&self) -> TxfifoR {
        TxfifoR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Receive FIFO Size"]
    #[inline(always)]
    pub fn rxfifo(&self) -> RxfifoR {
        RxfifoR::new(((self.bits >> 8) & 0xff) as u8)
    }
}
#[doc = "Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ParamSpec;
impl crate::RegisterSpec for ParamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`param::R`](R) reader structure"]
impl crate::Readable for ParamSpec {}
#[doc = "`reset()` method sets PARAM to value 0x0202"]
impl crate::Resettable for ParamSpec {
    const RESET_VALUE: u32 = 0x0202;
}
