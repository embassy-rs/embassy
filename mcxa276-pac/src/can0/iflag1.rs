#[doc = "Register `IFLAG1` reader"]
pub type R = crate::R<Iflag1Spec>;
#[doc = "Register `IFLAG1` writer"]
pub type W = crate::W<Iflag1Spec>;
#[doc = "Buffer MB0 Interrupt or Clear Legacy FIFO bit\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buf0i {
    #[doc = "0: MB0 has no occurrence of successfully completed transmission or reception."]
    BufferTxRxNotComplete = 0,
    #[doc = "1: MB0 has successfully completed transmission or reception."]
    BufferTxRxComplete = 1,
}
impl From<Buf0i> for bool {
    #[inline(always)]
    fn from(variant: Buf0i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF0I` reader - Buffer MB0 Interrupt or Clear Legacy FIFO bit"]
pub type Buf0iR = crate::BitReader<Buf0i>;
impl Buf0iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Buf0i {
        match self.bits {
            false => Buf0i::BufferTxRxNotComplete,
            true => Buf0i::BufferTxRxComplete,
        }
    }
    #[doc = "MB0 has no occurrence of successfully completed transmission or reception."]
    #[inline(always)]
    pub fn is_buffer_tx_rx_not_complete(&self) -> bool {
        *self == Buf0i::BufferTxRxNotComplete
    }
    #[doc = "MB0 has successfully completed transmission or reception."]
    #[inline(always)]
    pub fn is_buffer_tx_rx_complete(&self) -> bool {
        *self == Buf0i::BufferTxRxComplete
    }
}
#[doc = "Field `BUF0I` writer - Buffer MB0 Interrupt or Clear Legacy FIFO bit"]
pub type Buf0iW<'a, REG> = crate::BitWriter1C<'a, REG, Buf0i>;
impl<'a, REG> Buf0iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "MB0 has no occurrence of successfully completed transmission or reception."]
    #[inline(always)]
    pub fn buffer_tx_rx_not_complete(self) -> &'a mut crate::W<REG> {
        self.variant(Buf0i::BufferTxRxNotComplete)
    }
    #[doc = "MB0 has successfully completed transmission or reception."]
    #[inline(always)]
    pub fn buffer_tx_rx_complete(self) -> &'a mut crate::W<REG> {
        self.variant(Buf0i::BufferTxRxComplete)
    }
}
#[doc = "Field `BUF4TO1I` reader - Buffer MBi Interrupt or Reserved"]
pub type Buf4to1iR = crate::FieldReader;
#[doc = "Field `BUF4TO1I` writer - Buffer MBi Interrupt or Reserved"]
pub type Buf4to1iW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Buffer MB5 Interrupt or Frames available in Legacy RX FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buf5i {
    #[doc = "0: No occurrence of completed transmission or reception, or no frames available"]
    Id1 = 0,
    #[doc = "1: MB5 completed transmission or reception, or frames available"]
    Id2 = 1,
}
impl From<Buf5i> for bool {
    #[inline(always)]
    fn from(variant: Buf5i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF5I` reader - Buffer MB5 Interrupt or Frames available in Legacy RX FIFO"]
pub type Buf5iR = crate::BitReader<Buf5i>;
impl Buf5iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Buf5i {
        match self.bits {
            false => Buf5i::Id1,
            true => Buf5i::Id2,
        }
    }
    #[doc = "No occurrence of completed transmission or reception, or no frames available"]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Buf5i::Id1
    }
    #[doc = "MB5 completed transmission or reception, or frames available"]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Buf5i::Id2
    }
}
#[doc = "Field `BUF5I` writer - Buffer MB5 Interrupt or Frames available in Legacy RX FIFO"]
pub type Buf5iW<'a, REG> = crate::BitWriter1C<'a, REG, Buf5i>;
impl<'a, REG> Buf5iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No occurrence of completed transmission or reception, or no frames available"]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Buf5i::Id1)
    }
    #[doc = "MB5 completed transmission or reception, or frames available"]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Buf5i::Id2)
    }
}
#[doc = "Buffer MB6 Interrupt or Legacy RX FIFO Warning\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buf6i {
    #[doc = "0: No occurrence of MB6 completing transmission or reception, or FIFO not almost full."]
    Id1 = 0,
    #[doc = "1: MB6 completed transmission or reception, or FIFO almost full."]
    Id2 = 1,
}
impl From<Buf6i> for bool {
    #[inline(always)]
    fn from(variant: Buf6i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF6I` reader - Buffer MB6 Interrupt or Legacy RX FIFO Warning"]
pub type Buf6iR = crate::BitReader<Buf6i>;
impl Buf6iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Buf6i {
        match self.bits {
            false => Buf6i::Id1,
            true => Buf6i::Id2,
        }
    }
    #[doc = "No occurrence of MB6 completing transmission or reception, or FIFO not almost full."]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Buf6i::Id1
    }
    #[doc = "MB6 completed transmission or reception, or FIFO almost full."]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Buf6i::Id2
    }
}
#[doc = "Field `BUF6I` writer - Buffer MB6 Interrupt or Legacy RX FIFO Warning"]
pub type Buf6iW<'a, REG> = crate::BitWriter1C<'a, REG, Buf6i>;
impl<'a, REG> Buf6iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No occurrence of MB6 completing transmission or reception, or FIFO not almost full."]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Buf6i::Id1)
    }
    #[doc = "MB6 completed transmission or reception, or FIFO almost full."]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Buf6i::Id2)
    }
}
#[doc = "Buffer MB7 Interrupt or Legacy RX FIFO Overflow\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buf7i {
    #[doc = "0: No occurrence of MB7 completing transmission or reception, or no FIFO overflow."]
    Id1 = 0,
    #[doc = "1: MB7 completed transmission or reception, or FIFO overflow."]
    Id2 = 1,
}
impl From<Buf7i> for bool {
    #[inline(always)]
    fn from(variant: Buf7i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF7I` reader - Buffer MB7 Interrupt or Legacy RX FIFO Overflow"]
pub type Buf7iR = crate::BitReader<Buf7i>;
impl Buf7iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Buf7i {
        match self.bits {
            false => Buf7i::Id1,
            true => Buf7i::Id2,
        }
    }
    #[doc = "No occurrence of MB7 completing transmission or reception, or no FIFO overflow."]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Buf7i::Id1
    }
    #[doc = "MB7 completed transmission or reception, or FIFO overflow."]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Buf7i::Id2
    }
}
#[doc = "Field `BUF7I` writer - Buffer MB7 Interrupt or Legacy RX FIFO Overflow"]
pub type Buf7iW<'a, REG> = crate::BitWriter1C<'a, REG, Buf7i>;
impl<'a, REG> Buf7iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No occurrence of MB7 completing transmission or reception, or no FIFO overflow."]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Buf7i::Id1)
    }
    #[doc = "MB7 completed transmission or reception, or FIFO overflow."]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Buf7i::Id2)
    }
}
#[doc = "Field `BUF31TO8I` reader - Buffer MBi Interrupt"]
pub type Buf31to8iR = crate::FieldReader<u32>;
#[doc = "Field `BUF31TO8I` writer - Buffer MBi Interrupt"]
pub type Buf31to8iW<'a, REG> = crate::FieldWriter<'a, REG, 24, u32>;
impl R {
    #[doc = "Bit 0 - Buffer MB0 Interrupt or Clear Legacy FIFO bit"]
    #[inline(always)]
    pub fn buf0i(&self) -> Buf0iR {
        Buf0iR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:4 - Buffer MBi Interrupt or Reserved"]
    #[inline(always)]
    pub fn buf4to1i(&self) -> Buf4to1iR {
        Buf4to1iR::new(((self.bits >> 1) & 0x0f) as u8)
    }
    #[doc = "Bit 5 - Buffer MB5 Interrupt or Frames available in Legacy RX FIFO"]
    #[inline(always)]
    pub fn buf5i(&self) -> Buf5iR {
        Buf5iR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Buffer MB6 Interrupt or Legacy RX FIFO Warning"]
    #[inline(always)]
    pub fn buf6i(&self) -> Buf6iR {
        Buf6iR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Buffer MB7 Interrupt or Legacy RX FIFO Overflow"]
    #[inline(always)]
    pub fn buf7i(&self) -> Buf7iR {
        Buf7iR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:31 - Buffer MBi Interrupt"]
    #[inline(always)]
    pub fn buf31to8i(&self) -> Buf31to8iR {
        Buf31to8iR::new((self.bits >> 8) & 0x00ff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Buffer MB0 Interrupt or Clear Legacy FIFO bit"]
    #[inline(always)]
    pub fn buf0i(&mut self) -> Buf0iW<Iflag1Spec> {
        Buf0iW::new(self, 0)
    }
    #[doc = "Bits 1:4 - Buffer MBi Interrupt or Reserved"]
    #[inline(always)]
    pub fn buf4to1i(&mut self) -> Buf4to1iW<Iflag1Spec> {
        Buf4to1iW::new(self, 1)
    }
    #[doc = "Bit 5 - Buffer MB5 Interrupt or Frames available in Legacy RX FIFO"]
    #[inline(always)]
    pub fn buf5i(&mut self) -> Buf5iW<Iflag1Spec> {
        Buf5iW::new(self, 5)
    }
    #[doc = "Bit 6 - Buffer MB6 Interrupt or Legacy RX FIFO Warning"]
    #[inline(always)]
    pub fn buf6i(&mut self) -> Buf6iW<Iflag1Spec> {
        Buf6iW::new(self, 6)
    }
    #[doc = "Bit 7 - Buffer MB7 Interrupt or Legacy RX FIFO Overflow"]
    #[inline(always)]
    pub fn buf7i(&mut self) -> Buf7iW<Iflag1Spec> {
        Buf7iW::new(self, 7)
    }
    #[doc = "Bits 8:31 - Buffer MBi Interrupt"]
    #[inline(always)]
    pub fn buf31to8i(&mut self) -> Buf31to8iW<Iflag1Spec> {
        Buf31to8iW::new(self, 8)
    }
}
#[doc = "Interrupt Flags 1\n\nYou can [`read`](crate::Reg::read) this register and get [`iflag1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`iflag1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Iflag1Spec;
impl crate::RegisterSpec for Iflag1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`iflag1::R`](R) reader structure"]
impl crate::Readable for Iflag1Spec {}
#[doc = "`write(|w| ..)` method takes [`iflag1::W`](W) writer structure"]
impl crate::Writable for Iflag1Spec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xffff_ffff;
}
#[doc = "`reset()` method sets IFLAG1 to value 0"]
impl crate::Resettable for Iflag1Spec {}
