#[doc = "Register `MTDR` writer"]
pub type W = crate::W<MtdrSpec>;
#[doc = "Field `DATA` writer - Transmit Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Command Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmd {
    #[doc = "0: Transmit the value in DATA\\[7:0\\]"]
    TransmitData7Through0 = 0,
    #[doc = "1: Receive (DATA\\[7:0\\] + 1) bytes"]
    ReceiveData7Through0PlusOne = 1,
    #[doc = "2: Generate Stop condition on I2C bus"]
    GenerateStopCondition = 2,
    #[doc = "3: Receive and discard (DATA\\[7:0\\] + 1) bytes"]
    ReceiveAndDiscardData7Through0PlusOne = 3,
    #[doc = "4: Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\]"]
    GenerateStartAndTransmitAddressInData7Through0 = 4,
    #[doc = "5: Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] (this transfer expects a NACK to be returned)"]
    GenerateStartAndTransmitAddressInData7Through0ExpectNack = 5,
    #[doc = "6: Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] using HS mode"]
    GenerateStartAndTransmitAddressInData7Through0UsingHighSpeedMode = 6,
    #[doc = "7: Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] using HS mode (this transfer expects a NACK to be returned)"]
    GenerateStartAndTransmitAddressInData7Through0UsingHighSpeedModeExpectNack = 7,
}
impl From<Cmd> for u8 {
    #[inline(always)]
    fn from(variant: Cmd) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmd {
    type Ux = u8;
}
impl crate::IsEnum for Cmd {}
#[doc = "Field `CMD` writer - Command Data"]
pub type CmdW<'a, REG> = crate::FieldWriter<'a, REG, 3, Cmd, crate::Safe>;
impl<'a, REG> CmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Transmit the value in DATA\\[7:0\\]"]
    #[inline(always)]
    pub fn transmit_data_7_through_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::TransmitData7Through0)
    }
    #[doc = "Receive (DATA\\[7:0\\] + 1) bytes"]
    #[inline(always)]
    pub fn receive_data_7_through_0_plus_one(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::ReceiveData7Through0PlusOne)
    }
    #[doc = "Generate Stop condition on I2C bus"]
    #[inline(always)]
    pub fn generate_stop_condition(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::GenerateStopCondition)
    }
    #[doc = "Receive and discard (DATA\\[7:0\\] + 1) bytes"]
    #[inline(always)]
    pub fn receive_and_discard_data_7_through_0_plus_one(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::ReceiveAndDiscardData7Through0PlusOne)
    }
    #[doc = "Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\]"]
    #[inline(always)]
    pub fn generate_start_and_transmit_address_in_data_7_through_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::GenerateStartAndTransmitAddressInData7Through0)
    }
    #[doc = "Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] (this transfer expects a NACK to be returned)"]
    #[inline(always)]
    pub fn generate_start_and_transmit_address_in_data_7_through_0_expect_nack(
        self,
    ) -> &'a mut crate::W<REG> {
        self.variant(Cmd::GenerateStartAndTransmitAddressInData7Through0ExpectNack)
    }
    #[doc = "Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] using HS mode"]
    #[inline(always)]
    pub fn generate_start_and_transmit_address_in_data_7_through_0_using_high_speed_mode(
        self,
    ) -> &'a mut crate::W<REG> {
        self.variant(Cmd::GenerateStartAndTransmitAddressInData7Through0UsingHighSpeedMode)
    }
    #[doc = "Generate (repeated) Start on the I2C bus and transmit the address in DATA\\[7:0\\] using HS mode (this transfer expects a NACK to be returned)"]
    #[inline(always)]
    pub fn generate_start_and_transmit_address_in_data_7_through_0_using_high_speed_mode_expect_nack(
        self,
    ) -> &'a mut crate::W<REG> {
        self.variant(
            Cmd::GenerateStartAndTransmitAddressInData7Through0UsingHighSpeedModeExpectNack,
        )
    }
}
impl W {
    #[doc = "Bits 0:7 - Transmit Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<MtdrSpec> {
        DataW::new(self, 0)
    }
    #[doc = "Bits 8:10 - Command Data"]
    #[inline(always)]
    pub fn cmd(&mut self) -> CmdW<MtdrSpec> {
        CmdW::new(self, 8)
    }
}
#[doc = "Controller Transmit Data\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mtdr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MtdrSpec;
impl crate::RegisterSpec for MtdrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mtdr::W`](W) writer structure"]
impl crate::Writable for MtdrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MTDR to value 0"]
impl crate::Resettable for MtdrSpec {}
