#[doc = "Register `STAR` reader"]
pub type R = crate::R<StarSpec>;
#[doc = "Register `STAR` writer"]
pub type W = crate::W<StarSpec>;
#[doc = "Transmit NACK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnack {
    #[doc = "0: Transmit ACK"]
    TransmitAck = 0,
    #[doc = "1: Transmit NACK"]
    TransmitNack = 1,
}
impl From<Txnack> for bool {
    #[inline(always)]
    fn from(variant: Txnack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNACK` reader - Transmit NACK"]
pub type TxnackR = crate::BitReader<Txnack>;
impl TxnackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnack {
        match self.bits {
            false => Txnack::TransmitAck,
            true => Txnack::TransmitNack,
        }
    }
    #[doc = "Transmit ACK"]
    #[inline(always)]
    pub fn is_transmit_ack(&self) -> bool {
        *self == Txnack::TransmitAck
    }
    #[doc = "Transmit NACK"]
    #[inline(always)]
    pub fn is_transmit_nack(&self) -> bool {
        *self == Txnack::TransmitNack
    }
}
#[doc = "Field `TXNACK` writer - Transmit NACK"]
pub type TxnackW<'a, REG> = crate::BitWriter<'a, REG, Txnack>;
impl<'a, REG> TxnackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Transmit ACK"]
    #[inline(always)]
    pub fn transmit_ack(self) -> &'a mut crate::W<REG> {
        self.variant(Txnack::TransmitAck)
    }
    #[doc = "Transmit NACK"]
    #[inline(always)]
    pub fn transmit_nack(self) -> &'a mut crate::W<REG> {
        self.variant(Txnack::TransmitNack)
    }
}
impl R {
    #[doc = "Bit 0 - Transmit NACK"]
    #[inline(always)]
    pub fn txnack(&self) -> TxnackR {
        TxnackR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit NACK"]
    #[inline(always)]
    pub fn txnack(&mut self) -> TxnackW<StarSpec> {
        TxnackW::new(self, 0)
    }
}
#[doc = "Target Transmit ACK\n\nYou can [`read`](crate::Reg::read) this register and get [`star::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`star::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StarSpec;
impl crate::RegisterSpec for StarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`star::R`](R) reader structure"]
impl crate::Readable for StarSpec {}
#[doc = "`write(|w| ..)` method takes [`star::W`](W) writer structure"]
impl crate::Writable for StarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STAR to value 0"]
impl crate::Resettable for StarSpec {}
