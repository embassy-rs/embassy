#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Metadata {
    pub name: &'static str,
    pub family: &'static str,
    pub line: &'static str,
    pub memory: &'static [MemoryRegion],
    pub peripherals: &'static [Peripheral],
    pub interrupts: &'static [Interrupt],
    pub dma_channels: &'static [DmaChannel],
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MemoryRegion {
    pub name: &'static str,
    pub kind: MemoryRegionKind,
    pub address: u32,
    pub size: u32,
    pub settings: Option<FlashSettings>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FlashSettings {
    pub erase_size: u32,
    pub write_size: u32,
    pub erase_value: u8,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MemoryRegionKind {
    Flash,
    Ram,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Interrupt {
    pub name: &'static str,
    pub number: u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Package {
    pub name: &'static str,
    pub package: &'static str,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Peripheral {
    pub name: &'static str,
    pub address: u64,
    pub registers: Option<PeripheralRegisters>,
    pub rcc: Option<PeripheralRcc>,
    pub pins: &'static [PeripheralPin],
    pub dma_channels: &'static [PeripheralDmaChannel],
    pub interrupts: &'static [PeripheralInterrupt],
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralRegisters {
    pub kind: &'static str,
    pub version: &'static str,
    pub block: &'static str,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralInterrupt {
    pub signal: &'static str,
    pub interrupt: &'static str,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralRcc {
    pub clock: &'static str,
    pub enable: Option<PeripheralRccRegister>,
    pub reset: Option<PeripheralRccRegister>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralRccRegister {
    pub register: &'static str,
    pub field: &'static str,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralPin {
    pub pin: &'static str,
    pub signal: &'static str,
    pub af: Option<u8>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DmaChannel {
    pub name: &'static str,
    pub dma: &'static str,
    pub channel: u32,
    pub dmamux: Option<&'static str>,
    pub dmamux_channel: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PeripheralDmaChannel {
    pub signal: &'static str,
    pub channel: Option<&'static str>,
    pub dmamux: Option<&'static str>,
    pub dma: Option<&'static str>,
    pub request: Option<u32>,
}
