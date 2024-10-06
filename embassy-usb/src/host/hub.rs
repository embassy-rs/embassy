//! Usb hub driver implementation
//! 
//! Port indexes are zero-based

use embassy_time::Timer;
use embassy_usb_driver::host::{HostError, RequestType, SetupPacket, UsbChannel, UsbHostDriver};

use crate::{control::Request, host::DeviceInfo};

use super::{channel, descriptor::InterfaceDescriptor, Channel, ControlChannelExt, Device, USBDescriptor, UsbHost};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};
use bit_field::BitField;

/// USB 2.0 Spec heading: 11.23.2.1
#[derive(KnownLayout, FromBytes, Immutable, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
struct HubDescriptor {
    len: u8,
    desc_type: u8,
    port_num: u8,
    characteristics0: u8,
    characteristics1: u8,
    /// ms x 2
    power_on_delay: u8,
    /// Maximum controller current
    max_current: u8,
    /// 8 + 8 bits per port, at max 127 ports => 32 bytes
    port_buf: [u8; 32]
}

impl USBDescriptor for HubDescriptor {
    const SIZE: usize = size_of::<Self>();

    const DESC_TYPE: u8 = 0x29;

    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized {
        let (byref, _) = Self::ref_from_prefix(bytes).map_err(|_| ())?;
        if byref.desc_type != Self::DESC_TYPE {
            return Err(())
        }

        Ok(byref.clone())
    }
}

/// USB 2.0 Spec: 11.24.2 Table 11-17
#[derive(Clone, Copy)]
#[repr(u8)]
enum PortFeature {
    Connection = 0,
    Enable,
    Suspend,
    OverCurrent,
    Reset,
    Power = 8,
    LowSpeed,
    ChangeConnection = 16,
    ChangeEnable,
    ChangeSuspend,
    ChangeOverCurrent,
    ChangeReset,
    Test,
    Indicator,
}

bitflags! {
    /// USB 2.0 Spec: 11.24.2.7.1
    struct PortStatus: u16 {
        const CONNECTED   = 1 << 0;
        const ENABLED     = 1 << 1;
        const SUSPENDED   = 1 << 2;
        const OVERCURRENT = 1 << 3;
        const RESET       = 1 << 4;
        // Reserved: 5..8
        const POWERED     = 1 << 8;
        const LOW_SPEED   = 1 << 9;
        const HIGH_SPEED  = 1 << 10;
        const TEST_MODE   = 1 << 11;
        const INDICATOR_CUSTOM_COLOR = 1 << 12;
        // Reserved: 13..16
    }
}

bitflags! {
    /// USB 2.0 Spec: 11.24.2.7.2
    struct PortStatusChange: u16 {
        const CONNECT     = 1 << 0;
        const ENABLE      = 1 << 1;
        const SUSPEND     = 1 << 2;
        const OVERCURRENT = 1 << 3;
        const RESET       = 1 << 4;
        // Reserved: 5..16
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Speed {
    Low,
    Full,
    High
}

impl Speed {
    pub fn from_status(status: PortStatus) -> Self {
        let ls = status.contains(PortStatus::LOW_SPEED);
        let hs = status.contains(PortStatus::HIGH_SPEED);

        match (ls, hs) {
            (true, _) => Speed::Low,
            (false, false) => Speed::Full,
            (false, true) => Speed::High
        }
    }
}

pub struct UsbHub<'h, D: UsbHostDriver> {
    addr: u8,
    desc: HubDescriptor,
    control: Channel<'h, D, channel::Control, channel::InOut>,
    interrupt: Channel<'h, D, channel::Interrupt, channel::In>,
}

pub fn compatible(dev: &Device) -> bool {
    find_interface(dev).is_some()
}

pub fn find_interface(dev: &Device) -> Option<InterfaceDescriptor> {
    for iface in 0..dev.cfg_desc.num_interfaces {
        let iface = dev.cfg_desc.parse_interface(iface as usize).unwrap();

        match iface {
            InterfaceDescriptor { 
                interface_class: 0x09, 
                interface_subclass: 0x0, 
                interface_protocol: 0x0, 
                ..    
            } => return Some(iface),
            _ => continue
        }
    }
    
    None
}

impl<'h, D: UsbHostDriver> UsbHub<'h, D> {
    pub async fn configure(dev: &Device, host: &'h UsbHost<'_, D>) -> Result<Self, HostError> {
        let iface = find_interface(dev)
            .ok_or(HostError::Other("device is not compatible with hub driver"))?;
        let eps = iface.parse_endpoints::<1>();

        let desc = { 
            let mut cc = host.control_channel(dev.addr).await?;
            cc.request_descriptor::<HubDescriptor, 64>(true).await?
        };

        debug!("Hub descriptor: {}", desc);

        let mut hub = Self {
            addr: dev.addr,
            desc,
            control: host.alloc_control_channel(dev.addr).await?,
            interrupt: host.alloc_channel(dev.addr, &eps[0]).await?,
        };

        // Power-On ports
        for port in 0..hub.desc.port_num {
            hub.port_feature(true, PortFeature::Power, port, 0).await?;
        }
        Timer::after_millis(hub.desc.power_on_delay as u64 * 2).await;
        
        Ok(hub)
    }

    pub async fn poll(&mut self) -> Result<Device, HostError> {
        loop {
            // Wait for status change
            let mut buf = [0u8; 16];
            // 1 bit per port + 1 reserved
            let slice = &mut buf[..(self.desc.port_num as usize / 8) + 1];
            self.interrupt.request_in(slice).await?;
                       
            // Find first changed port
            let changed = slice.iter()
                .flat_map(|byte| (0..8).map(|bit| byte.get_bit(bit)))
                // Bit 0 is reserved, port status starts from bit 1
                .skip(1)
                .position(|bit| bit);
            
            if let Some(idx) = changed {
                let idx = idx as u8;
                trace!("HUB {}: port {} is changed, requesting status", self.addr, idx);
                // Target to self
                self.control.driver.retarget_channel(&mut self.control.channel, self.addr, 64, false)?;

                // Get status 
                let (status, change) = self.get_port_status(idx).await?;
                debug!("HUB {}: port {} status: {} change: {}", self.addr, idx, status, change);

                // TODO: Overcurrent protection
                // Clear reset change after reset
                if change.contains(PortStatusChange::RESET) {
                    self.port_feature(false, PortFeature::ChangeReset, idx, 0).await?;
                }

                if change.contains(PortStatusChange::CONNECT) {
                    match status.contains(PortStatus::CONNECTED) {
                        // Device connected, perform bus reset and configure
                        true => {
                            // Determine speed
                            let speed = Speed::from_status(status); 
                            
                            debug!(
                                "HUB {}: Device connected to port {} with {} speed", 
                                self.addr, idx, speed
                            );

                            // Determine if device needs low-speed preamble
                            let needs_pre = match speed {
                                Speed::Low => true,
                                Speed::Full => false,
                                Speed::High => {
                                    error!("High speed devices are not supported");
                                    continue
                                },
                            };
                            
                            // Clear connect status change
                            self.port_feature(false, PortFeature::ChangeConnection, idx, 0).await?;
                            
                            // Reset and wait
                            self.port_feature(true, PortFeature::Reset, idx, 0).await?;
                            Timer::after_millis(50).await;                           
                            self.port_feature(false, PortFeature::ChangeReset, idx, 0).await?;

                            return super::configure_device(
                                self.control.driver, 
                                &mut self.control.channel,
                                self.control.registry,
                                needs_pre,
                                Some((self.addr, idx))
                            ).await;
                        },
                        // Device disconnected, remove from registry
                        false => {
                            debug!("HUB {}: Device disconnected from port {}", self.addr, idx);
                            // Find device by hub and port
                            match self.control.registry.find_by_port(self.addr, idx).await {
                                Some(addr) => {
                                    let count = self.control.registry.remove_device(addr).await;
                                    debug!("Disconnected {} devices", count);
                                },
                                None => error!("There is no such device in registry"),
                            }

                            // Clear change status
                            self.port_feature(false, PortFeature::ChangeConnection, idx, 0).await?;
                        }
                    }
                }
            }                    
        }
    }

    /// Set/Clear PortFeature
    /// 
    /// USB 2.0 Spec: 11.24.2.13,1 
    async fn port_feature(
        &mut self, 
        set: bool, 
        feature: PortFeature, 
        port: u8, 
        selector: u8,
    ) -> Result<(), HostError> {
        let setup = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_CLASS | RequestType::RECIPIENT_OTHER,
            request: if set { Request::SET_FEATURE } else { Request::CLEAR_FEATURE },
            value: feature as u16,
            index: (selector as u16) << 8 | (port + 1) as u16,
            length: 0
        };
        
        self.control.control_out(&setup, &mut []).await?;
        Ok(())
    }

    /// GetPortStatus
    /// 
    /// USB 2.0 Spec: 11.24.2.7 
    async fn get_port_status(
        &mut self,
        port: u8,
    ) -> Result<(PortStatus, PortStatusChange), HostError> {
        let setup = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_CLASS | RequestType::RECIPIENT_OTHER,
            request: Request::GET_STATUS,
            value: 0,
            index: (port + 1) as u16,
            length: 4
        };

        let mut buf = [0u16; 2];
        
        self.control.control_in(&setup, buf.as_mut_bytes()).await?;

        let status = PortStatus::from_bits_truncate(buf[0]);
        let change = PortStatusChange::from_bits_truncate(buf[1]);
        
        Ok((status, change))
    }
}
