#[doc = "Register `ERREN` reader"]
pub type R = crate::R<ErrenSpec>;
#[doc = "Register `ERREN` writer"]
pub type W = crate::W<ErrenSpec>;
#[doc = "PIDERR Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piderren {
    #[doc = "0: Disable"]
    DisPiderrInt = 0,
    #[doc = "1: Enable"]
    EnPiderrInt = 1,
}
impl From<Piderren> for bool {
    #[inline(always)]
    fn from(variant: Piderren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIDERREN` reader - PIDERR Interrupt Enable"]
pub type PiderrenR = crate::BitReader<Piderren>;
impl PiderrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Piderren {
        match self.bits {
            false => Piderren::DisPiderrInt,
            true => Piderren::EnPiderrInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_piderr_int(&self) -> bool {
        *self == Piderren::DisPiderrInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_piderr_int(&self) -> bool {
        *self == Piderren::EnPiderrInt
    }
}
#[doc = "Field `PIDERREN` writer - PIDERR Interrupt Enable"]
pub type PiderrenW<'a, REG> = crate::BitWriter<'a, REG, Piderren>;
impl<'a, REG> PiderrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_piderr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Piderren::DisPiderrInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_piderr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Piderren::EnPiderrInt)
    }
}
#[doc = "CRC5/EOF Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crc5eofen {
    #[doc = "0: Disable"]
    DisCrc5Int = 0,
    #[doc = "1: Enable"]
    EnCrc5Int = 1,
}
impl From<Crc5eofen> for bool {
    #[inline(always)]
    fn from(variant: Crc5eofen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRC5EOFEN` reader - CRC5/EOF Interrupt Enable"]
pub type Crc5eofenR = crate::BitReader<Crc5eofen>;
impl Crc5eofenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crc5eofen {
        match self.bits {
            false => Crc5eofen::DisCrc5Int,
            true => Crc5eofen::EnCrc5Int,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_crc5_int(&self) -> bool {
        *self == Crc5eofen::DisCrc5Int
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_crc5_int(&self) -> bool {
        *self == Crc5eofen::EnCrc5Int
    }
}
#[doc = "Field `CRC5EOFEN` writer - CRC5/EOF Interrupt Enable"]
pub type Crc5eofenW<'a, REG> = crate::BitWriter<'a, REG, Crc5eofen>;
impl<'a, REG> Crc5eofenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_crc5_int(self) -> &'a mut crate::W<REG> {
        self.variant(Crc5eofen::DisCrc5Int)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_crc5_int(self) -> &'a mut crate::W<REG> {
        self.variant(Crc5eofen::EnCrc5Int)
    }
}
#[doc = "CRC16 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crc16en {
    #[doc = "0: Disable"]
    DisCrc16Int = 0,
    #[doc = "1: Enable"]
    EnCrc16Int = 1,
}
impl From<Crc16en> for bool {
    #[inline(always)]
    fn from(variant: Crc16en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRC16EN` reader - CRC16 Interrupt Enable"]
pub type Crc16enR = crate::BitReader<Crc16en>;
impl Crc16enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crc16en {
        match self.bits {
            false => Crc16en::DisCrc16Int,
            true => Crc16en::EnCrc16Int,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_crc16_int(&self) -> bool {
        *self == Crc16en::DisCrc16Int
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_crc16_int(&self) -> bool {
        *self == Crc16en::EnCrc16Int
    }
}
#[doc = "Field `CRC16EN` writer - CRC16 Interrupt Enable"]
pub type Crc16enW<'a, REG> = crate::BitWriter<'a, REG, Crc16en>;
impl<'a, REG> Crc16enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_crc16_int(self) -> &'a mut crate::W<REG> {
        self.variant(Crc16en::DisCrc16Int)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_crc16_int(self) -> &'a mut crate::W<REG> {
        self.variant(Crc16en::EnCrc16Int)
    }
}
#[doc = "DFN8 (Data Field Not Integer Number of Bytes) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dfn8en {
    #[doc = "0: Disable"]
    DisDfn8Int = 0,
    #[doc = "1: Enable"]
    EnDfn8Int = 1,
}
impl From<Dfn8en> for bool {
    #[inline(always)]
    fn from(variant: Dfn8en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DFN8EN` reader - DFN8 (Data Field Not Integer Number of Bytes) Interrupt Enable"]
pub type Dfn8enR = crate::BitReader<Dfn8en>;
impl Dfn8enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dfn8en {
        match self.bits {
            false => Dfn8en::DisDfn8Int,
            true => Dfn8en::EnDfn8Int,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_dfn8_int(&self) -> bool {
        *self == Dfn8en::DisDfn8Int
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_dfn8_int(&self) -> bool {
        *self == Dfn8en::EnDfn8Int
    }
}
#[doc = "Field `DFN8EN` writer - DFN8 (Data Field Not Integer Number of Bytes) Interrupt Enable"]
pub type Dfn8enW<'a, REG> = crate::BitWriter<'a, REG, Dfn8en>;
impl<'a, REG> Dfn8enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_dfn8_int(self) -> &'a mut crate::W<REG> {
        self.variant(Dfn8en::DisDfn8Int)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_dfn8_int(self) -> &'a mut crate::W<REG> {
        self.variant(Dfn8en::EnDfn8Int)
    }
}
#[doc = "BTOERR (Bus Timeout Error) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btoerren {
    #[doc = "0: Disable"]
    DisBtoerrInt = 0,
    #[doc = "1: Enable"]
    EnBtoerrInt = 1,
}
impl From<Btoerren> for bool {
    #[inline(always)]
    fn from(variant: Btoerren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTOERREN` reader - BTOERR (Bus Timeout Error) Interrupt Enable"]
pub type BtoerrenR = crate::BitReader<Btoerren>;
impl BtoerrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Btoerren {
        match self.bits {
            false => Btoerren::DisBtoerrInt,
            true => Btoerren::EnBtoerrInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_btoerr_int(&self) -> bool {
        *self == Btoerren::DisBtoerrInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_btoerr_int(&self) -> bool {
        *self == Btoerren::EnBtoerrInt
    }
}
#[doc = "Field `BTOERREN` writer - BTOERR (Bus Timeout Error) Interrupt Enable"]
pub type BtoerrenW<'a, REG> = crate::BitWriter<'a, REG, Btoerren>;
impl<'a, REG> BtoerrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_btoerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Btoerren::DisBtoerrInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_btoerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Btoerren::EnBtoerrInt)
    }
}
#[doc = "DMAERR Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmaerren {
    #[doc = "0: Disable"]
    DisDmaerrInt = 0,
    #[doc = "1: Enable"]
    EnDmaerrInt = 1,
}
impl From<Dmaerren> for bool {
    #[inline(always)]
    fn from(variant: Dmaerren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMAERREN` reader - DMAERR Interrupt Enable"]
pub type DmaerrenR = crate::BitReader<Dmaerren>;
impl DmaerrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmaerren {
        match self.bits {
            false => Dmaerren::DisDmaerrInt,
            true => Dmaerren::EnDmaerrInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_dmaerr_int(&self) -> bool {
        *self == Dmaerren::DisDmaerrInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_dmaerr_int(&self) -> bool {
        *self == Dmaerren::EnDmaerrInt
    }
}
#[doc = "Field `DMAERREN` writer - DMAERR Interrupt Enable"]
pub type DmaerrenW<'a, REG> = crate::BitWriter<'a, REG, Dmaerren>;
impl<'a, REG> DmaerrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_dmaerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaerren::DisDmaerrInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_dmaerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaerren::EnDmaerrInt)
    }
}
#[doc = "OWNERR Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownerren {
    #[doc = "0: Disable"]
    DisOwnerrInt = 0,
    #[doc = "1: Enable"]
    EnOwnerrInt = 1,
}
impl From<Ownerren> for bool {
    #[inline(always)]
    fn from(variant: Ownerren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OWNERREN` reader - OWNERR Interrupt Enable"]
pub type OwnerrenR = crate::BitReader<Ownerren>;
impl OwnerrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ownerren {
        match self.bits {
            false => Ownerren::DisOwnerrInt,
            true => Ownerren::EnOwnerrInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ownerr_int(&self) -> bool {
        *self == Ownerren::DisOwnerrInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ownerr_int(&self) -> bool {
        *self == Ownerren::EnOwnerrInt
    }
}
#[doc = "Field `OWNERREN` writer - OWNERR Interrupt Enable"]
pub type OwnerrenW<'a, REG> = crate::BitWriter<'a, REG, Ownerren>;
impl<'a, REG> OwnerrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ownerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Ownerren::DisOwnerrInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ownerr_int(self) -> &'a mut crate::W<REG> {
        self.variant(Ownerren::EnOwnerrInt)
    }
}
#[doc = "BTSERR (Bit Stuff Error) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btserren {
    #[doc = "0: Disable"]
    DisBtserrenInt = 0,
    #[doc = "1: Enable"]
    EnBtserrenInt = 1,
}
impl From<Btserren> for bool {
    #[inline(always)]
    fn from(variant: Btserren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTSERREN` reader - BTSERR (Bit Stuff Error) Interrupt Enable"]
pub type BtserrenR = crate::BitReader<Btserren>;
impl BtserrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Btserren {
        match self.bits {
            false => Btserren::DisBtserrenInt,
            true => Btserren::EnBtserrenInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_btserren_int(&self) -> bool {
        *self == Btserren::DisBtserrenInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_btserren_int(&self) -> bool {
        *self == Btserren::EnBtserrenInt
    }
}
#[doc = "Field `BTSERREN` writer - BTSERR (Bit Stuff Error) Interrupt Enable"]
pub type BtserrenW<'a, REG> = crate::BitWriter<'a, REG, Btserren>;
impl<'a, REG> BtserrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_btserren_int(self) -> &'a mut crate::W<REG> {
        self.variant(Btserren::DisBtserrenInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_btserren_int(self) -> &'a mut crate::W<REG> {
        self.variant(Btserren::EnBtserrenInt)
    }
}
impl R {
    #[doc = "Bit 0 - PIDERR Interrupt Enable"]
    #[inline(always)]
    pub fn piderren(&self) -> PiderrenR {
        PiderrenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - CRC5/EOF Interrupt Enable"]
    #[inline(always)]
    pub fn crc5eofen(&self) -> Crc5eofenR {
        Crc5eofenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - CRC16 Interrupt Enable"]
    #[inline(always)]
    pub fn crc16en(&self) -> Crc16enR {
        Crc16enR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - DFN8 (Data Field Not Integer Number of Bytes) Interrupt Enable"]
    #[inline(always)]
    pub fn dfn8en(&self) -> Dfn8enR {
        Dfn8enR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - BTOERR (Bus Timeout Error) Interrupt Enable"]
    #[inline(always)]
    pub fn btoerren(&self) -> BtoerrenR {
        BtoerrenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - DMAERR Interrupt Enable"]
    #[inline(always)]
    pub fn dmaerren(&self) -> DmaerrenR {
        DmaerrenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - OWNERR Interrupt Enable"]
    #[inline(always)]
    pub fn ownerren(&self) -> OwnerrenR {
        OwnerrenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - BTSERR (Bit Stuff Error) Interrupt Enable"]
    #[inline(always)]
    pub fn btserren(&self) -> BtserrenR {
        BtserrenR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - PIDERR Interrupt Enable"]
    #[inline(always)]
    pub fn piderren(&mut self) -> PiderrenW<ErrenSpec> {
        PiderrenW::new(self, 0)
    }
    #[doc = "Bit 1 - CRC5/EOF Interrupt Enable"]
    #[inline(always)]
    pub fn crc5eofen(&mut self) -> Crc5eofenW<ErrenSpec> {
        Crc5eofenW::new(self, 1)
    }
    #[doc = "Bit 2 - CRC16 Interrupt Enable"]
    #[inline(always)]
    pub fn crc16en(&mut self) -> Crc16enW<ErrenSpec> {
        Crc16enW::new(self, 2)
    }
    #[doc = "Bit 3 - DFN8 (Data Field Not Integer Number of Bytes) Interrupt Enable"]
    #[inline(always)]
    pub fn dfn8en(&mut self) -> Dfn8enW<ErrenSpec> {
        Dfn8enW::new(self, 3)
    }
    #[doc = "Bit 4 - BTOERR (Bus Timeout Error) Interrupt Enable"]
    #[inline(always)]
    pub fn btoerren(&mut self) -> BtoerrenW<ErrenSpec> {
        BtoerrenW::new(self, 4)
    }
    #[doc = "Bit 5 - DMAERR Interrupt Enable"]
    #[inline(always)]
    pub fn dmaerren(&mut self) -> DmaerrenW<ErrenSpec> {
        DmaerrenW::new(self, 5)
    }
    #[doc = "Bit 6 - OWNERR Interrupt Enable"]
    #[inline(always)]
    pub fn ownerren(&mut self) -> OwnerrenW<ErrenSpec> {
        OwnerrenW::new(self, 6)
    }
    #[doc = "Bit 7 - BTSERR (Bit Stuff Error) Interrupt Enable"]
    #[inline(always)]
    pub fn btserren(&mut self) -> BtserrenW<ErrenSpec> {
        BtserrenW::new(self, 7)
    }
}
#[doc = "Error Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`erren::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erren::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErrenSpec;
impl crate::RegisterSpec for ErrenSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`erren::R`](R) reader structure"]
impl crate::Readable for ErrenSpec {}
#[doc = "`write(|w| ..)` method takes [`erren::W`](W) writer structure"]
impl crate::Writable for ErrenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ERREN to value 0"]
impl crate::Resettable for ErrenSpec {}
