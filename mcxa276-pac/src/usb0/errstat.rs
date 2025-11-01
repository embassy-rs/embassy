#[doc = "Register `ERRSTAT` reader"]
pub type R = crate::R<ErrstatSpec>;
#[doc = "Register `ERRSTAT` writer"]
pub type W = crate::W<ErrstatSpec>;
#[doc = "PID Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piderr {
    #[doc = "0: Did not fail"]
    IntNo = 0,
    #[doc = "1: Failed"]
    IntYes = 1,
}
impl From<Piderr> for bool {
    #[inline(always)]
    fn from(variant: Piderr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIDERR` reader - PID Error Flag"]
pub type PiderrR = crate::BitReader<Piderr>;
impl PiderrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Piderr {
        match self.bits {
            false => Piderr::IntNo,
            true => Piderr::IntYes,
        }
    }
    #[doc = "Did not fail"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Piderr::IntNo
    }
    #[doc = "Failed"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Piderr::IntYes
    }
}
#[doc = "Field `PIDERR` writer - PID Error Flag"]
pub type PiderrW<'a, REG> = crate::BitWriter1C<'a, REG, Piderr>;
impl<'a, REG> PiderrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Did not fail"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Piderr::IntNo)
    }
    #[doc = "Failed"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Piderr::IntYes)
    }
}
#[doc = "CRC5 Error or End of Frame Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crc5eof {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Crc5eof> for bool {
    #[inline(always)]
    fn from(variant: Crc5eof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRC5EOF` reader - CRC5 Error or End of Frame Error Flag"]
pub type Crc5eofR = crate::BitReader<Crc5eof>;
impl Crc5eofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crc5eof {
        match self.bits {
            false => Crc5eof::IntNo,
            true => Crc5eof::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Crc5eof::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Crc5eof::IntYes
    }
}
#[doc = "Field `CRC5EOF` writer - CRC5 Error or End of Frame Error Flag"]
pub type Crc5eofW<'a, REG> = crate::BitWriter1C<'a, REG, Crc5eof>;
impl<'a, REG> Crc5eofW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Crc5eof::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Crc5eof::IntYes)
    }
}
#[doc = "CRC16 Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crc16 {
    #[doc = "0: Not rejected"]
    IntNo = 0,
    #[doc = "1: Rejected"]
    IntYes = 1,
}
impl From<Crc16> for bool {
    #[inline(always)]
    fn from(variant: Crc16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRC16` reader - CRC16 Error Flag"]
pub type Crc16R = crate::BitReader<Crc16>;
impl Crc16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crc16 {
        match self.bits {
            false => Crc16::IntNo,
            true => Crc16::IntYes,
        }
    }
    #[doc = "Not rejected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Crc16::IntNo
    }
    #[doc = "Rejected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Crc16::IntYes
    }
}
#[doc = "Field `CRC16` writer - CRC16 Error Flag"]
pub type Crc16W<'a, REG> = crate::BitWriter1C<'a, REG, Crc16>;
impl<'a, REG> Crc16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not rejected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Crc16::IntNo)
    }
    #[doc = "Rejected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Crc16::IntYes)
    }
}
#[doc = "Data Field Not 8 Bits Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dfn8 {
    #[doc = "0: Integer number of bytes"]
    IntNo = 0,
    #[doc = "1: Not an integer number of bytes"]
    IntYes = 1,
}
impl From<Dfn8> for bool {
    #[inline(always)]
    fn from(variant: Dfn8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DFN8` reader - Data Field Not 8 Bits Flag"]
pub type Dfn8R = crate::BitReader<Dfn8>;
impl Dfn8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dfn8 {
        match self.bits {
            false => Dfn8::IntNo,
            true => Dfn8::IntYes,
        }
    }
    #[doc = "Integer number of bytes"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Dfn8::IntNo
    }
    #[doc = "Not an integer number of bytes"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Dfn8::IntYes
    }
}
#[doc = "Field `DFN8` writer - Data Field Not 8 Bits Flag"]
pub type Dfn8W<'a, REG> = crate::BitWriter1C<'a, REG, Dfn8>;
impl<'a, REG> Dfn8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Integer number of bytes"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Dfn8::IntNo)
    }
    #[doc = "Not an integer number of bytes"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Dfn8::IntYes)
    }
}
#[doc = "Bus Turnaround Timeout Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btoerr {
    #[doc = "0: Not timed out"]
    IntNo = 0,
    #[doc = "1: Timed out"]
    IntYes = 1,
}
impl From<Btoerr> for bool {
    #[inline(always)]
    fn from(variant: Btoerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTOERR` reader - Bus Turnaround Timeout Error Flag"]
pub type BtoerrR = crate::BitReader<Btoerr>;
impl BtoerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Btoerr {
        match self.bits {
            false => Btoerr::IntNo,
            true => Btoerr::IntYes,
        }
    }
    #[doc = "Not timed out"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Btoerr::IntNo
    }
    #[doc = "Timed out"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Btoerr::IntYes
    }
}
#[doc = "Field `BTOERR` writer - Bus Turnaround Timeout Error Flag"]
pub type BtoerrW<'a, REG> = crate::BitWriter1C<'a, REG, Btoerr>;
impl<'a, REG> BtoerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not timed out"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Btoerr::IntNo)
    }
    #[doc = "Timed out"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Btoerr::IntYes)
    }
}
#[doc = "DMA Access Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmaerr {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Dmaerr> for bool {
    #[inline(always)]
    fn from(variant: Dmaerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMAERR` reader - DMA Access Error Flag"]
pub type DmaerrR = crate::BitReader<Dmaerr>;
impl DmaerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmaerr {
        match self.bits {
            false => Dmaerr::IntNo,
            true => Dmaerr::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Dmaerr::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Dmaerr::IntYes
    }
}
#[doc = "Field `DMAERR` writer - DMA Access Error Flag"]
pub type DmaerrW<'a, REG> = crate::BitWriter1C<'a, REG, Dmaerr>;
impl<'a, REG> DmaerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaerr::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaerr::IntYes)
    }
}
#[doc = "BD Unavailable Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownerr {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Ownerr> for bool {
    #[inline(always)]
    fn from(variant: Ownerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OWNERR` reader - BD Unavailable Error Flag"]
pub type OwnerrR = crate::BitReader<Ownerr>;
impl OwnerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ownerr {
        match self.bits {
            false => Ownerr::IntNo,
            true => Ownerr::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Ownerr::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Ownerr::IntYes
    }
}
#[doc = "Field `OWNERR` writer - BD Unavailable Error Flag"]
pub type OwnerrW<'a, REG> = crate::BitWriter1C<'a, REG, Ownerr>;
impl<'a, REG> OwnerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Ownerr::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Ownerr::IntYes)
    }
}
#[doc = "Bit Stuff Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btserr {
    #[doc = "0: Packet not rejected due to the error"]
    IntNo = 0,
    #[doc = "1: Packet rejected due to the error"]
    IntYes = 1,
}
impl From<Btserr> for bool {
    #[inline(always)]
    fn from(variant: Btserr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTSERR` reader - Bit Stuff Error Flag"]
pub type BtserrR = crate::BitReader<Btserr>;
impl BtserrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Btserr {
        match self.bits {
            false => Btserr::IntNo,
            true => Btserr::IntYes,
        }
    }
    #[doc = "Packet not rejected due to the error"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Btserr::IntNo
    }
    #[doc = "Packet rejected due to the error"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Btserr::IntYes
    }
}
#[doc = "Field `BTSERR` writer - Bit Stuff Error Flag"]
pub type BtserrW<'a, REG> = crate::BitWriter1C<'a, REG, Btserr>;
impl<'a, REG> BtserrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Packet not rejected due to the error"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Btserr::IntNo)
    }
    #[doc = "Packet rejected due to the error"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Btserr::IntYes)
    }
}
impl R {
    #[doc = "Bit 0 - PID Error Flag"]
    #[inline(always)]
    pub fn piderr(&self) -> PiderrR {
        PiderrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - CRC5 Error or End of Frame Error Flag"]
    #[inline(always)]
    pub fn crc5eof(&self) -> Crc5eofR {
        Crc5eofR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - CRC16 Error Flag"]
    #[inline(always)]
    pub fn crc16(&self) -> Crc16R {
        Crc16R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Data Field Not 8 Bits Flag"]
    #[inline(always)]
    pub fn dfn8(&self) -> Dfn8R {
        Dfn8R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Bus Turnaround Timeout Error Flag"]
    #[inline(always)]
    pub fn btoerr(&self) -> BtoerrR {
        BtoerrR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - DMA Access Error Flag"]
    #[inline(always)]
    pub fn dmaerr(&self) -> DmaerrR {
        DmaerrR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - BD Unavailable Error Flag"]
    #[inline(always)]
    pub fn ownerr(&self) -> OwnerrR {
        OwnerrR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Bit Stuff Error Flag"]
    #[inline(always)]
    pub fn btserr(&self) -> BtserrR {
        BtserrR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - PID Error Flag"]
    #[inline(always)]
    pub fn piderr(&mut self) -> PiderrW<ErrstatSpec> {
        PiderrW::new(self, 0)
    }
    #[doc = "Bit 1 - CRC5 Error or End of Frame Error Flag"]
    #[inline(always)]
    pub fn crc5eof(&mut self) -> Crc5eofW<ErrstatSpec> {
        Crc5eofW::new(self, 1)
    }
    #[doc = "Bit 2 - CRC16 Error Flag"]
    #[inline(always)]
    pub fn crc16(&mut self) -> Crc16W<ErrstatSpec> {
        Crc16W::new(self, 2)
    }
    #[doc = "Bit 3 - Data Field Not 8 Bits Flag"]
    #[inline(always)]
    pub fn dfn8(&mut self) -> Dfn8W<ErrstatSpec> {
        Dfn8W::new(self, 3)
    }
    #[doc = "Bit 4 - Bus Turnaround Timeout Error Flag"]
    #[inline(always)]
    pub fn btoerr(&mut self) -> BtoerrW<ErrstatSpec> {
        BtoerrW::new(self, 4)
    }
    #[doc = "Bit 5 - DMA Access Error Flag"]
    #[inline(always)]
    pub fn dmaerr(&mut self) -> DmaerrW<ErrstatSpec> {
        DmaerrW::new(self, 5)
    }
    #[doc = "Bit 6 - BD Unavailable Error Flag"]
    #[inline(always)]
    pub fn ownerr(&mut self) -> OwnerrW<ErrstatSpec> {
        OwnerrW::new(self, 6)
    }
    #[doc = "Bit 7 - Bit Stuff Error Flag"]
    #[inline(always)]
    pub fn btserr(&mut self) -> BtserrW<ErrstatSpec> {
        BtserrW::new(self, 7)
    }
}
#[doc = "Error Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`errstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`errstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErrstatSpec;
impl crate::RegisterSpec for ErrstatSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`errstat::R`](R) reader structure"]
impl crate::Readable for ErrstatSpec {}
#[doc = "`write(|w| ..)` method takes [`errstat::W`](W) writer structure"]
impl crate::Writable for ErrstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u8 = 0xff;
}
#[doc = "`reset()` method sets ERRSTAT to value 0"]
impl crate::Resettable for ErrstatSpec {}
