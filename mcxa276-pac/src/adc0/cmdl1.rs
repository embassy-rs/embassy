#[doc = "Register `CMDL1` reader"]
pub type R = crate::R<Cmdl1Spec>;
#[doc = "Register `CMDL1` writer"]
pub type W = crate::W<Cmdl1Spec>;
#[doc = "Input Channel Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Adch {
    #[doc = "0: Select CH0A."]
    SelectCh0 = 0,
    #[doc = "1: Select CH1A."]
    SelectCh1 = 1,
    #[doc = "2: Select CH2A."]
    SelectCh2 = 2,
    #[doc = "3: Select CH3A."]
    SelectCh3 = 3,
    #[doc = "4: Select corresponding channel CHnA."]
    SelectCorrespondingChannel4 = 4,
    #[doc = "5: Select corresponding channel CHnA."]
    SelectCorrespondingChannel5 = 5,
    #[doc = "6: Select corresponding channel CHnA."]
    SelectCorrespondingChannel6 = 6,
    #[doc = "7: Select corresponding channel CHnA."]
    SelectCorrespondingChannel7 = 7,
    #[doc = "8: Select corresponding channel CHnA."]
    SelectCorrespondingChannel8 = 8,
    #[doc = "9: Select corresponding channel CHnA."]
    SelectCorrespondingChannel9 = 9,
    #[doc = "30: Select CH30A."]
    SelectCh30 = 30,
    #[doc = "31: Select CH31A."]
    SelectCh31 = 31,
}
impl From<Adch> for u8 {
    #[inline(always)]
    fn from(variant: Adch) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Adch {
    type Ux = u8;
}
impl crate::IsEnum for Adch {}
#[doc = "Field `ADCH` reader - Input Channel Select"]
pub type AdchR = crate::FieldReader<Adch>;
impl AdchR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Adch> {
        match self.bits {
            0 => Some(Adch::SelectCh0),
            1 => Some(Adch::SelectCh1),
            2 => Some(Adch::SelectCh2),
            3 => Some(Adch::SelectCh3),
            4 => Some(Adch::SelectCorrespondingChannel4),
            5 => Some(Adch::SelectCorrespondingChannel5),
            6 => Some(Adch::SelectCorrespondingChannel6),
            7 => Some(Adch::SelectCorrespondingChannel7),
            8 => Some(Adch::SelectCorrespondingChannel8),
            9 => Some(Adch::SelectCorrespondingChannel9),
            30 => Some(Adch::SelectCh30),
            31 => Some(Adch::SelectCh31),
            _ => None,
        }
    }
    #[doc = "Select CH0A."]
    #[inline(always)]
    pub fn is_select_ch0(&self) -> bool {
        *self == Adch::SelectCh0
    }
    #[doc = "Select CH1A."]
    #[inline(always)]
    pub fn is_select_ch1(&self) -> bool {
        *self == Adch::SelectCh1
    }
    #[doc = "Select CH2A."]
    #[inline(always)]
    pub fn is_select_ch2(&self) -> bool {
        *self == Adch::SelectCh2
    }
    #[doc = "Select CH3A."]
    #[inline(always)]
    pub fn is_select_ch3(&self) -> bool {
        *self == Adch::SelectCh3
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_4(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel4
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_5(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel5
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_6(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel6
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_7(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel7
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_8(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel8
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn is_select_corresponding_channel_9(&self) -> bool {
        *self == Adch::SelectCorrespondingChannel9
    }
    #[doc = "Select CH30A."]
    #[inline(always)]
    pub fn is_select_ch30(&self) -> bool {
        *self == Adch::SelectCh30
    }
    #[doc = "Select CH31A."]
    #[inline(always)]
    pub fn is_select_ch31(&self) -> bool {
        *self == Adch::SelectCh31
    }
}
#[doc = "Field `ADCH` writer - Input Channel Select"]
pub type AdchW<'a, REG> = crate::FieldWriter<'a, REG, 5, Adch>;
impl<'a, REG> AdchW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Select CH0A."]
    #[inline(always)]
    pub fn select_ch0(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh0)
    }
    #[doc = "Select CH1A."]
    #[inline(always)]
    pub fn select_ch1(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh1)
    }
    #[doc = "Select CH2A."]
    #[inline(always)]
    pub fn select_ch2(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh2)
    }
    #[doc = "Select CH3A."]
    #[inline(always)]
    pub fn select_ch3(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh3)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_4(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel4)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_5(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel5)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_6(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel6)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_7(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel7)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_8(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel8)
    }
    #[doc = "Select corresponding channel CHnA."]
    #[inline(always)]
    pub fn select_corresponding_channel_9(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCorrespondingChannel9)
    }
    #[doc = "Select CH30A."]
    #[inline(always)]
    pub fn select_ch30(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh30)
    }
    #[doc = "Select CH31A."]
    #[inline(always)]
    pub fn select_ch31(self) -> &'a mut crate::W<REG> {
        self.variant(Adch::SelectCh31)
    }
}
#[doc = "Conversion Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ctype {
    #[doc = "0: Single-Ended Mode. Only A side channel is converted."]
    SingleEndedASideChannel = 0,
}
impl From<Ctype> for u8 {
    #[inline(always)]
    fn from(variant: Ctype) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ctype {
    type Ux = u8;
}
impl crate::IsEnum for Ctype {}
#[doc = "Field `CTYPE` reader - Conversion Type"]
pub type CtypeR = crate::FieldReader<Ctype>;
impl CtypeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ctype> {
        match self.bits {
            0 => Some(Ctype::SingleEndedASideChannel),
            _ => None,
        }
    }
    #[doc = "Single-Ended Mode. Only A side channel is converted."]
    #[inline(always)]
    pub fn is_single_ended_a_side_channel(&self) -> bool {
        *self == Ctype::SingleEndedASideChannel
    }
}

pub type CtypeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Ctype>;

#[doc = "Select Resolution of Conversions\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    #[doc = "0: Standard resolution. Single-ended 12-bit conversion."]
    Data12Bits = 0,
    #[doc = "1: High resolution. Single-ended 16-bit conversion."]
    Data16Bits = 1,
}
impl From<Mode> for bool {
    #[inline(always)]
    fn from(variant: Mode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MODE` reader - Select Resolution of Conversions"]
pub type ModeR = crate::BitReader<Mode>;
impl ModeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mode {
        match self.bits {
            false => Mode::Data12Bits,
            true => Mode::Data16Bits,
        }
    }
    #[doc = "Standard resolution. Single-ended 12-bit conversion."]
    #[inline(always)]
    pub fn is_data_12_bits(&self) -> bool {
        *self == Mode::Data12Bits
    }
    #[doc = "High resolution. Single-ended 16-bit conversion."]
    #[inline(always)]
    pub fn is_data_16_bits(&self) -> bool {
        *self == Mode::Data16Bits
    }
}
#[doc = "Field `MODE` writer - Select Resolution of Conversions"]
pub type ModeW<'a, REG> = crate::BitWriter<'a, REG, Mode>;
impl<'a, REG> ModeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Standard resolution. Single-ended 12-bit conversion."]
    #[inline(always)]
    pub fn data_12_bits(self) -> &'a mut crate::W<REG> {
        self.variant(Mode::Data12Bits)
    }
    #[doc = "High resolution. Single-ended 16-bit conversion."]
    #[inline(always)]
    pub fn data_16_bits(self) -> &'a mut crate::W<REG> {
        self.variant(Mode::Data16Bits)
    }
}
impl R {
    #[doc = "Bits 0:4 - Input Channel Select"]
    #[inline(always)]
    pub fn adch(&self) -> AdchR {
        AdchR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:6 - Conversion Type"]
    #[inline(always)]
    pub fn ctype(&self) -> CtypeR {
        CtypeR::new(((self.bits >> 5) & 3) as u8)
    }
    #[doc = "Bit 7 - Select Resolution of Conversions"]
    #[inline(always)]
    pub fn mode(&self) -> ModeR {
        ModeR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Input Channel Select"]
    #[inline(always)]
    pub fn adch(&mut self) -> AdchW<Cmdl1Spec> {
        AdchW::new(self, 0)
    }
    #[doc = "Bits 5:6 - Conversion Type"]
    #[inline(always)]
    pub fn ctype(&mut self) -> CtypeW<Cmdl1Spec> {
        CtypeW::new(self, 5)
    }
    #[doc = "Bit 7 - Select Resolution of Conversions"]
    #[inline(always)]
    pub fn mode(&mut self) -> ModeW<Cmdl1Spec> {
        ModeW::new(self, 7)
    }
}
#[doc = "Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cmdl1Spec;
impl crate::RegisterSpec for Cmdl1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cmdl1::R`](R) reader structure"]
impl crate::Readable for Cmdl1Spec {}
#[doc = "`write(|w| ..)` method takes [`cmdl1::W`](W) writer structure"]
impl crate::Writable for Cmdl1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CMDL1 to value 0"]
impl crate::Resettable for Cmdl1Spec {}
