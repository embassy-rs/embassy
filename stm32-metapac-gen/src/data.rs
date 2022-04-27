use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Chip {
    pub name: String,
    pub family: String,
    pub line: String,
    pub cores: Vec<Core>,
    pub memory: Vec<MemoryRegion>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MemoryRegion {
    pub name: String,
    pub kind: MemoryRegionKind,
    pub address: u32,
    pub size: u32,
    pub settings: Option<FlashSettings>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct FlashSettings {
    pub erase_size: u32,
    pub write_size: u32,
    pub erase_value: u8,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub enum MemoryRegionKind {
    #[serde(rename = "flash")]
    Flash,
    #[serde(rename = "ram")]
    Ram,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Core {
    pub name: String,
    pub peripherals: Vec<Peripheral>,
    pub interrupts: Vec<Interrupt>,
    pub dma_channels: Vec<DmaChannel>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Interrupt {
    pub name: String,
    pub number: u32,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub package: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Peripheral {
    pub name: String,
    pub address: u64,
    #[serde(default)]
    pub registers: Option<PeripheralRegisters>,
    #[serde(default)]
    pub rcc: Option<PeripheralRcc>,
    #[serde(default)]
    pub pins: Vec<PeripheralPin>,
    #[serde(default)]
    pub dma_channels: Vec<PeripheralDmaChannel>,
    #[serde(default)]
    pub interrupts: Vec<PeripheralInterrupt>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct PeripheralInterrupt {
    pub signal: String,
    pub interrupt: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct PeripheralRcc {
    pub clock: String,
    #[serde(default)]
    pub enable: Option<PeripheralRccRegister>,
    #[serde(default)]
    pub reset: Option<PeripheralRccRegister>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct PeripheralRccRegister {
    pub register: String,
    pub field: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct PeripheralPin {
    pub pin: String,
    pub signal: String,
    pub af: Option<u8>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct DmaChannel {
    pub name: String,
    pub dma: String,
    pub channel: u32,
    pub dmamux: Option<String>,
    pub dmamux_channel: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Hash)]
pub struct PeripheralDmaChannel {
    pub signal: String,
    pub channel: Option<String>,
    pub dmamux: Option<String>,
    pub dma: Option<String>,
    pub request: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Hash)]
pub struct PeripheralRegisters {
    pub kind: String,
    pub version: String,
    pub block: String,
}
