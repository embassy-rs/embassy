use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Chip {
    pub name: String,
    pub family: String,
    pub line: String,
    pub cores: Vec<Core>,
    pub flash: Memory,
    pub ram: Memory,
    pub packages: Vec<Package>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Memory {
    pub bytes: u32,
    pub regions: HashMap<String, MemoryRegion>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MemoryRegion {
    pub base: u32,
    pub bytes: Option<u32>,
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
    pub block: Option<String>,
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
    pub registers: PeripheralRccRegisters,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct PeripheralRccRegisters {
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
    pub af: Option<String>,
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
    pub request: Option<u32>,
}

pub struct BlockInfo {
    /// usart_v1/USART -> usart
    pub module: String,
    /// usart_v1/USART -> v1
    pub version: String,
    /// usart_v1/USART -> USART
    pub block: String,
}

impl BlockInfo {
    pub fn parse(s: &str) -> Self {
        let mut s = s.split('/');
        let module = s.next().unwrap();
        let block = s.next().unwrap();
        assert!(s.next().is_none());
        let mut s = module.split('_');
        let module = s.next().unwrap();
        let version = s.next().unwrap();
        assert!(s.next().is_none());
        Self {
            module: module.to_string(),
            version: version.to_string(),
            block: block.to_string(),
        }
    }
}
