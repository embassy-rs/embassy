#[doc = "Register `MB9_16B_CS` reader"]
pub type R = crate::R<Mb16bGroupMb9_16bCsSpec>;
#[doc = "Register `MB9_16B_CS` writer"]
pub type W = crate::W<Mb16bGroupMb9_16bCsSpec>;
#[doc = "Field `TIME_STAMP` reader - Free-Running Counter Time stamp. This 16-bit field is a copy of the Free-Running Timer, captured for Tx and Rx frames at the time when the beginning of the Identifier field appears on the CAN bus."]
pub type TimeStampR = crate::FieldReader<u16>;
#[doc = "Field `TIME_STAMP` writer - Free-Running Counter Time stamp. This 16-bit field is a copy of the Free-Running Timer, captured for Tx and Rx frames at the time when the beginning of the Identifier field appears on the CAN bus."]
pub type TimeStampW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `DLC` reader - Length of the data to be stored/transmitted."]
pub type DlcR = crate::FieldReader;
#[doc = "Field `DLC` writer - Length of the data to be stored/transmitted."]
pub type DlcW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `RTR` reader - Remote Transmission Request. One/zero for remote/data frame."]
pub type RtrR = crate::BitReader;
#[doc = "Field `RTR` writer - Remote Transmission Request. One/zero for remote/data frame."]
pub type RtrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `IDE` reader - ID Extended. One/zero for extended/standard format frame."]
pub type IdeR = crate::BitReader;
#[doc = "Field `IDE` writer - ID Extended. One/zero for extended/standard format frame."]
pub type IdeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SRR` reader - Substitute Remote Request. Contains a fixed recessive bit."]
pub type SrrR = crate::BitReader;
#[doc = "Field `SRR` writer - Substitute Remote Request. Contains a fixed recessive bit."]
pub type SrrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CODE` reader - Message Buffer Code. This 4-bit field can be accessed (read or write) by the CPU and by the FlexCAN module itself, as part of the message buffer matching and arbitration process."]
pub type CodeR = crate::FieldReader;
#[doc = "Field `CODE` writer - Message Buffer Code. This 4-bit field can be accessed (read or write) by the CPU and by the FlexCAN module itself, as part of the message buffer matching and arbitration process."]
pub type CodeW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `ESI` reader - Error State Indicator. This bit indicates if the transmitting node is error active or error passive."]
pub type EsiR = crate::BitReader;
#[doc = "Field `ESI` writer - Error State Indicator. This bit indicates if the transmitting node is error active or error passive."]
pub type EsiW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BRS` reader - Bit Rate Switch. This bit defines whether the bit rate is switched inside a CAN FD format frame."]
pub type BrsR = crate::BitReader;
#[doc = "Field `BRS` writer - Bit Rate Switch. This bit defines whether the bit rate is switched inside a CAN FD format frame."]
pub type BrsW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EDL` reader - Extended Data Length. This bit distinguishes between CAN format and CAN FD format frames. The EDL bit must not be set for Message Buffers configured to RANSWER with code field 0b1010."]
pub type EdlR = crate::BitReader;
#[doc = "Field `EDL` writer - Extended Data Length. This bit distinguishes between CAN format and CAN FD format frames. The EDL bit must not be set for Message Buffers configured to RANSWER with code field 0b1010."]
pub type EdlW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 0:15 - Free-Running Counter Time stamp. This 16-bit field is a copy of the Free-Running Timer, captured for Tx and Rx frames at the time when the beginning of the Identifier field appears on the CAN bus."]
    #[inline(always)]
    pub fn time_stamp(&self) -> TimeStampR {
        TimeStampR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:19 - Length of the data to be stored/transmitted."]
    #[inline(always)]
    pub fn dlc(&self) -> DlcR {
        DlcR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bit 20 - Remote Transmission Request. One/zero for remote/data frame."]
    #[inline(always)]
    pub fn rtr(&self) -> RtrR {
        RtrR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - ID Extended. One/zero for extended/standard format frame."]
    #[inline(always)]
    pub fn ide(&self) -> IdeR {
        IdeR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Substitute Remote Request. Contains a fixed recessive bit."]
    #[inline(always)]
    pub fn srr(&self) -> SrrR {
        SrrR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bits 24:27 - Message Buffer Code. This 4-bit field can be accessed (read or write) by the CPU and by the FlexCAN module itself, as part of the message buffer matching and arbitration process."]
    #[inline(always)]
    pub fn code(&self) -> CodeR {
        CodeR::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bit 29 - Error State Indicator. This bit indicates if the transmitting node is error active or error passive."]
    #[inline(always)]
    pub fn esi(&self) -> EsiR {
        EsiR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Bit Rate Switch. This bit defines whether the bit rate is switched inside a CAN FD format frame."]
    #[inline(always)]
    pub fn brs(&self) -> BrsR {
        BrsR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Extended Data Length. This bit distinguishes between CAN format and CAN FD format frames. The EDL bit must not be set for Message Buffers configured to RANSWER with code field 0b1010."]
    #[inline(always)]
    pub fn edl(&self) -> EdlR {
        EdlR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Free-Running Counter Time stamp. This 16-bit field is a copy of the Free-Running Timer, captured for Tx and Rx frames at the time when the beginning of the Identifier field appears on the CAN bus."]
    #[inline(always)]
    pub fn time_stamp(&mut self) -> TimeStampW<Mb16bGroupMb9_16bCsSpec> {
        TimeStampW::new(self, 0)
    }
    #[doc = "Bits 16:19 - Length of the data to be stored/transmitted."]
    #[inline(always)]
    pub fn dlc(&mut self) -> DlcW<Mb16bGroupMb9_16bCsSpec> {
        DlcW::new(self, 16)
    }
    #[doc = "Bit 20 - Remote Transmission Request. One/zero for remote/data frame."]
    #[inline(always)]
    pub fn rtr(&mut self) -> RtrW<Mb16bGroupMb9_16bCsSpec> {
        RtrW::new(self, 20)
    }
    #[doc = "Bit 21 - ID Extended. One/zero for extended/standard format frame."]
    #[inline(always)]
    pub fn ide(&mut self) -> IdeW<Mb16bGroupMb9_16bCsSpec> {
        IdeW::new(self, 21)
    }
    #[doc = "Bit 22 - Substitute Remote Request. Contains a fixed recessive bit."]
    #[inline(always)]
    pub fn srr(&mut self) -> SrrW<Mb16bGroupMb9_16bCsSpec> {
        SrrW::new(self, 22)
    }
    #[doc = "Bits 24:27 - Message Buffer Code. This 4-bit field can be accessed (read or write) by the CPU and by the FlexCAN module itself, as part of the message buffer matching and arbitration process."]
    #[inline(always)]
    pub fn code(&mut self) -> CodeW<Mb16bGroupMb9_16bCsSpec> {
        CodeW::new(self, 24)
    }
    #[doc = "Bit 29 - Error State Indicator. This bit indicates if the transmitting node is error active or error passive."]
    #[inline(always)]
    pub fn esi(&mut self) -> EsiW<Mb16bGroupMb9_16bCsSpec> {
        EsiW::new(self, 29)
    }
    #[doc = "Bit 30 - Bit Rate Switch. This bit defines whether the bit rate is switched inside a CAN FD format frame."]
    #[inline(always)]
    pub fn brs(&mut self) -> BrsW<Mb16bGroupMb9_16bCsSpec> {
        BrsW::new(self, 30)
    }
    #[doc = "Bit 31 - Extended Data Length. This bit distinguishes between CAN format and CAN FD format frames. The EDL bit must not be set for Message Buffers configured to RANSWER with code field 0b1010."]
    #[inline(always)]
    pub fn edl(&mut self) -> EdlW<Mb16bGroupMb9_16bCsSpec> {
        EdlW::new(self, 31)
    }
}
#[doc = "Message Buffer 9 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_cs::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_cs::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb16bGroupMb9_16bCsSpec;
impl crate::RegisterSpec for Mb16bGroupMb9_16bCsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_16b_group_mb9_16b_cs::R`](R) reader structure"]
impl crate::Readable for Mb16bGroupMb9_16bCsSpec {}
#[doc = "`write(|w| ..)` method takes [`mb_16b_group_mb9_16b_cs::W`](W) writer structure"]
impl crate::Writable for Mb16bGroupMb9_16bCsSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB9_16B_CS to value 0"]
impl crate::Resettable for Mb16bGroupMb9_16bCsSpec {}
