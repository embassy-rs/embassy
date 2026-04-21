//! Host driver for USB hubs.
#![allow(missing_docs)]
//!
//! Handles the deferred bus reset and port state/speed detection required for hub enumeration.
//! Requires the USB driver to support Interrupt IN pipes.

use core::num::NonZeroU8;

use bitflags::bitflags;
use embassy_time::Timer;
use embassy_usb::control::Request;
use embassy_usb_driver::host::{HostError, SplitInfo, SplitSpeed, UsbHostDriver, UsbPipe, pipe};
use embassy_usb_driver::{Direction, EndpointInfo, EndpointType, Speed};
use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::control::{ControlPipeExt, ControlType, Recipient, RequestType, SetupPacket};
use crate::descriptor::{DEFAULT_MAX_DESCRIPTOR_SIZE, InterfaceDescriptor, USBDescriptor};
use crate::handler::{BusRoute, EnumerationInfo, HandlerEvent, RegisterError};
use crate::{EnumerationError, UsbHost};

pub struct HubHandler<'d, H: UsbHostDriver<'d>, const MAX_PORTS: usize> {
    interrupt_channel: H::Pipe<pipe::Interrupt, pipe::In>,
    control_channel: H::Pipe<pipe::Control, pipe::InOut>,
    desc: HubDescriptor,
    device_address: u8,
    device_lut: [Option<NonZeroU8>; MAX_PORTS],
    route: BusRoute,
    _phantom: core::marker::PhantomData<&'d ()>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HubEvent {
    DeviceDetected { port: u8, speed: Speed },
    DeviceRemoved { address: Option<NonZeroU8>, port: u8 },
}

impl<'d, H: UsbHostDriver<'d>, const MAX_PORTS: usize> HubHandler<'d, H, MAX_PORTS> {
    /// Attempt to register a hub handler for the given device.
    pub async fn try_register(bus: &H, enum_info: &EnumerationInfo) -> Result<Self, RegisterError> {
        let ls_over_fs = matches!(enum_info.split(), Some(s) if s.device_speed() == SplitSpeed::Low);
        let mut control_channel = bus.alloc_pipe::<pipe::Control, pipe::InOut>(
            enum_info.device_address,
            &EndpointInfo {
                addr: 0.into(),
                ep_type: EndpointType::Control,
                max_packet_size: enum_info
                    .device_desc
                    .max_packet_size0
                    .min(if ls_over_fs { 8 } else { 64 }) as u16,
                interval_ms: 0,
            },
            enum_info.split(),
        )?;

        let mut cfg_desc_buf = [0u8; DEFAULT_MAX_DESCRIPTOR_SIZE];
        let configuration = enum_info
            .active_config_or_set_default(&mut control_channel, &mut cfg_desc_buf)
            .await?;

        let iface = configuration
            .iter_interface()
            .find(|v| {
                matches!(
                    v,
                    InterfaceDescriptor {
                        interface_class: 0x09,
                        interface_subclass: 0x0,
                        interface_protocol: 0x0,
                        ..
                    }
                )
            })
            .ok_or(RegisterError::NoSupportedInterface)?;

        let interrupt_ep = iface
            .iter_endpoints()
            .find(|v| v.ep_type() == EndpointType::Interrupt && v.ep_dir() == Direction::In)
            .ok_or(RegisterError::NoSupportedInterface)?;

        let interrupt_channel = bus.alloc_pipe::<pipe::Interrupt, pipe::In>(
            enum_info.device_address,
            &interrupt_ep.into(),
            enum_info.split(),
        )?;

        let desc = control_channel.request_descriptor::<HubDescriptor, 64>(0, true).await?;

        let mut hub = HubHandler {
            interrupt_channel,
            control_channel,
            desc,
            device_address: enum_info.device_address,
            device_lut: [None; MAX_PORTS],
            route: enum_info.route,
            _phantom: core::marker::PhantomData,
        };

        for port in 0..hub.desc.port_num {
            hub.port_feature(true, PortFeature::Power, port, 0).await?;
        }
        Timer::after_millis(hub.desc.power_on_delay as u64 * 2).await;

        Ok(hub)
    }

    /// Wait for a hub port status change event.
    pub async fn wait_for_event(&mut self) -> Result<HandlerEvent<HubEvent>, HostError> {
        loop {
            // 1 hub + maximum of 255 ports (USB 2.0 Spec 11.12.3 and 11.23.2.1)
            let mut buf = [0u8; (1 + 255) / u8::BITS as usize];
            let slice = &mut buf[..(self.desc.port_num as usize / 8) + 1];
            self.interrupt_channel.request_in(slice).await?;

            let mut hub_changes = HubInterrupt(slice);
            if hub_changes.take_hub_change() {
                trace!("HUB {}: hub changed, requesting status", self.device_address);

                let (status, change) = self.get_hub_status().await?;
                debug!(
                    "HUB {}: hub status: {:?} change: {:?}",
                    self.device_address, status, change
                );

                if !change.is_empty() {
                    return Err(HostError::Other("Unhandled hub status change"));
                }
            }
            while let Some(port) = hub_changes.take_port_change() {
                trace!("HUB {}: port {} changed, requesting status", self.device_address, port);

                let (status, mut change) = self.get_port_status(port).await?;
                debug!(
                    "HUB {}: port {} status: {:?} change: {:?}",
                    self.device_address, port, status, change
                );

                if change.contains(PortStatusChange::RESET) {
                    change.toggle(PortStatusChange::RESET);
                    self.port_feature(false, PortFeature::ChangeReset, port, 0).await?;
                }

                if change.contains(PortStatusChange::CONNECT) {
                    change.toggle(PortStatusChange::CONNECT);
                    self.port_feature(false, PortFeature::ChangeConnection, port, 0).await?;
                    match status.contains(PortStatus::CONNECTED) {
                        true => {
                            let speed: Speed = status.into();
                            debug!(
                                "HUB {}: Device connected to port {} at {:?}",
                                self.device_address, port, speed
                            );
                            return Ok(HandlerEvent::HandlerEvent(HubEvent::DeviceDetected { port, speed }));
                        }
                        false => {
                            debug!("HUB {}: Device disconnected from port {}", self.device_address, port);
                            let device_ref = self.device_lut.get_mut(port as usize);
                            return Ok(HandlerEvent::HandlerEvent(HubEvent::DeviceRemoved {
                                address: device_ref.and_then(|v| v.take()),
                                port,
                            }));
                        }
                    }
                }

                if !change.is_empty() {
                    return Err(HostError::Other("Unhandled port status change"));
                }
            }
        }
    }

    #[allow(dead_code)]
    async fn hub_feature(&mut self, set: bool, feature: HubFeature) -> Result<(), HostError> {
        let setup = SetupPacket {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Class,
                recipient: Recipient::Device,
            },
            request: if set {
                Request::SET_FEATURE
            } else {
                Request::CLEAR_FEATURE
            },
            value: feature as u16,
            index: 0,
            length: 0,
        };
        self.control_channel.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    async fn get_hub_status(&mut self) -> Result<(HubStatus, HubStatusChange), HostError> {
        let setup = SetupPacket {
            request_type: RequestType {
                direction: Direction::In,
                control_type: ControlType::Class,
                recipient: Recipient::Device,
            },
            request: Request::GET_STATUS,
            value: 0,
            index: 0,
            length: 4,
        };
        let mut buf = [0u8; 4];
        self.control_channel.control_in(&setup.to_bytes(), &mut buf).await?;
        Ok((
            HubStatus::from_bits_truncate(u16::from_le_bytes(buf[..2].try_into().unwrap())),
            HubStatusChange::from_bits_truncate(u16::from_le_bytes(buf[2..].try_into().unwrap())),
        ))
    }

    /// Reset a port and enumerate the device attached to it.
    ///
    /// `port` and `speed` are the 0-based port index and device speed as
    /// reported by [`HubEvent::DeviceDetected`]. `bus` is the USB host to
    /// use for enumeration, and it must be the same bus that this hub is
    /// registered on.
    ///
    /// Returns the [`EnumerationInfo`] for the device and bytes written
    /// to `config_buffer`.
    ///
    /// The route included in the [`EnumerationInfo`] is computed per
    /// USB 2.0 §11.14:
    ///
    /// - If this hub is itself reached through a split transaction (e.g. a
    ///   full-speed hub behind a high-speed hub's Transaction Translator),
    ///   the child inherits the parent's TT address and port, with the
    ///   `device_speed` updated to match the attached device. This is
    ///   correct because the topmost high-speed hub in the chain owns the TT
    ///   that services the entire subtree below it.
    /// - Otherwise, if this hub introduces a speed mismatch with the child
    ///   (HS hub with an LS/FS child, or FS hub with an LS child, where the
    ///   latter uses the legacy `PRE` prefix on full-speed buses), a new
    ///   [`SplitInfo`] is constructed pointing at this hub.
    /// - Otherwise the child is reached directly at its native speed.
    pub async fn enumerate_port(
        &mut self,
        bus: &mut UsbHost<'d, H>,
        config_buffer: &mut [u8],
        port: u8,
        speed: Speed,
    ) -> Result<(EnumerationInfo, usize), EnumerationError> {
        self.port_feature(true, PortFeature::Reset, port, 0).await?;
        // USB 2.0 §7.1.7.5: TDRSTR ≥ 10 ms. Match the 50 ms margin used in similar drivers.
        Timer::after_millis(50).await;
        self.port_feature(false, PortFeature::ChangeReset, port, 0).await?;

        let route = match self.route.split() {
            Some(parent_split) => match speed {
                Speed::Low => BusRoute::Translated(SplitInfo::new(
                    parent_split.hub_addr(),
                    parent_split.port(),
                    SplitSpeed::Low,
                )),
                Speed::Full => BusRoute::Translated(SplitInfo::new(
                    parent_split.hub_addr(),
                    parent_split.port(),
                    SplitSpeed::Full,
                )),
                Speed::High => BusRoute::Direct(speed),
            },
            None => {
                let split_speed = match (speed, self.route.device_speed()) {
                    (Speed::Low, Speed::Full | Speed::High) => Some(SplitSpeed::Low),
                    (Speed::Full, Speed::High) => Some(SplitSpeed::Full),
                    _ => None,
                };
                match split_speed {
                    Some(ss) => BusRoute::Translated(SplitInfo::new(self.device_address, port + 1, ss)),
                    None => BusRoute::Direct(speed),
                }
            }
        };

        let (info, config_len) = bus.enumerate(route, config_buffer).await?;

        // Store the device address in the LUT for later retrieval on disconnect.
        self.device_lut[port as usize] = NonZeroU8::new(info.device_address);

        Ok((info, config_len))
    }

    async fn port_feature(&mut self, set: bool, feature: PortFeature, port: u8, selector: u8) -> Result<(), HostError> {
        let setup = SetupPacket {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Class,
                recipient: Recipient::Other,
            },
            request: if set {
                Request::SET_FEATURE
            } else {
                Request::CLEAR_FEATURE
            },
            value: feature as u16,
            index: ((selector as u16) << 8) | (port + 1) as u16,
            length: 0,
        };
        self.control_channel.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    async fn get_port_status(&mut self, port: u8) -> Result<(PortStatus, PortStatusChange), HostError> {
        let setup = SetupPacket {
            request_type: RequestType {
                direction: Direction::In,
                control_type: ControlType::Class,
                recipient: Recipient::Other,
            },
            request: Request::GET_STATUS,
            value: 0,
            index: (port + 1) as u16,
            length: 4,
        };
        let mut buf = [0u8; 4];
        self.control_channel.control_in(&setup.to_bytes(), &mut buf).await?;
        Ok((
            PortStatus::from_bits_truncate(u16::from_le_bytes(buf[..2].try_into().unwrap())),
            PortStatusChange::from_bits_truncate(u16::from_le_bytes(buf[2..].try_into().unwrap())),
        ))
    }
}

/// Helper to interpret the data of the interrupt channel.
struct HubInterrupt<'a>(&'a mut [u8]);

impl HubInterrupt<'_> {
    /// Returns `true` if the hub has a status change, consuming it.
    /// Returns `false` if the hub does not have a status change.
    fn take_hub_change(&mut self) -> bool {
        let mut hub_change = false;
        // The hub is in idx 0 bit 0.
        if let Some(b) = self.0.get_mut(0) {
            if *b & 1 != 0 {
                *b &= !1;
                hub_change = true;
            }
        }
        hub_change
    }
    /// Returns the 0-based port number of the first port that has a status change, consuming it.
    /// Returns `None` if no port has a status change.
    ///
    /// ### Panic
    /// Panics if the hub status change has not been taken.
    fn take_port_change(&mut self) -> Option<u8> {
        self.0
            .iter_mut()
            .enumerate()
            .find(|(_, v)| v.trailing_zeros() < u8::BITS)
            .map(|(idx, v)| {
                let bit = v.trailing_zeros() as usize;
                if idx == 0 && bit == 0 {
                    panic!("the hub change must be taken before a port change is taken");
                }
                *v &= !(1 << bit);
                // The first port is in idx 0 bit 1.
                // This code starts port numbers at 0, so it needs a `- 1`.
                // On the other hand, the usb spec starts port numbers at 1,
                // so this number will be increased by 1 for `SetupPacket`.
                (bit + idx * 8 - 1) as u8
            })
    }
}

/// USB 2.0 Spec 11.23.2.1
#[derive(KnownLayout, FromBytes, Immutable, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
struct HubDescriptor {
    len: u8,
    desc_type: u8,
    port_num: u8,
    characteristics0: u8,
    characteristics1: u8,
    /// Power-on delay in units of 2ms.
    power_on_delay: u8,
    max_current: u8,
    port_buf: [u8; 32],
}

impl USBDescriptor for HubDescriptor {
    const SIZE: usize = core::mem::size_of::<Self>();
    const DESC_TYPE: u8 = 0x29;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (byref, _) = Self::ref_from_prefix(bytes).map_err(|_| ())?;
        if byref.desc_type != Self::DESC_TYPE {
            return Err(());
        }
        Ok(byref.clone())
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
enum HubFeature {
    ChangeHubLocalPower = 0,
    ChangeHubOverCurrent,
}

bitflags! {
    /// USB 2.0 Spec 11.24.2.6
    #[derive(Debug)]
    struct HubStatus: u16 {
        const LOCAL_POWER = 1 << 0;
        const OVERCURRENT = 1 << 1;
    }
}

bitflags! {
    /// USB 2.0 Spec 11.24.2.6
    #[derive(Debug)]
    struct HubStatusChange: u16 {
        const LOCAL_POWER = 1 << 0;
        const OVERCURRENT = 1 << 1;
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HubStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HubStatus({=u16:b})", self.bits());
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HubStatusChange {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HubStatusChange({=u16:b})", self.bits());
    }
}

#[allow(dead_code)]
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
    /// USB 2.0 Spec 11.24.2.7.1
    #[derive(Debug)]
    struct PortStatus: u16 {
        const CONNECTED   = 1 << 0;
        const ENABLED     = 1 << 1;
        const SUSPENDED   = 1 << 2;
        const OVERCURRENT = 1 << 3;
        const RESET       = 1 << 4;
        const POWERED     = 1 << 8;
        const LOW_SPEED   = 1 << 9;
        const HIGH_SPEED  = 1 << 10;
        const TEST_MODE   = 1 << 11;
        const INDICATOR_CUSTOM_COLOR = 1 << 12;
    }
}

bitflags! {
    /// USB 2.0 Spec 11.24.2.7.2
    #[derive(Debug)]
    struct PortStatusChange: u16 {
        const CONNECT     = 1 << 0;
        const ENABLE      = 1 << 1;
        const SUSPEND     = 1 << 2;
        const OVERCURRENT = 1 << 3;
        const RESET       = 1 << 4;
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PortStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "PortStatus({=u16:b})", self.bits());
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PortStatusChange {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "PortStatusChange({=u16:b})", self.bits());
    }
}

impl From<PortStatus> for Speed {
    fn from(value: PortStatus) -> Self {
        match (
            value.contains(PortStatus::LOW_SPEED),
            value.contains(PortStatus::HIGH_SPEED),
        ) {
            (true, _) => Speed::Low,
            (false, false) => Speed::Full,
            (false, true) => Speed::High,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::HubInterrupt;

    #[test]
    fn test_hub_interrupt_0() {
        let mut buf: [u8; _] = [];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), false);
        assert_eq!(changes.take_port_change(), None);
    }

    #[test]
    fn test_hub_interrupt_1_empty() {
        let mut buf: [u8; _] = [0b0000_0000];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), false);
        assert_eq!(changes.take_port_change(), None);
    }

    #[test]
    fn test_hub_interrupt_1_hub() {
        let mut buf: [u8; _] = [0b0000_0001];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), true);
        assert_eq!(changes.take_port_change(), None);
    }

    #[test]
    fn test_hub_interrupt_1_port() {
        let mut buf: [u8; _] = [0b0000_0010];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), false);
        assert_eq!(changes.take_port_change(), Some(0));
        assert_eq!(changes.take_port_change(), None);
    }

    #[test]
    fn test_hub_interrupt_1_full() {
        let mut buf: [u8; _] = [0b1111_1111];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), true);
        assert_eq!(changes.take_port_change(), Some(0));
        assert_eq!(changes.take_port_change(), Some(1));
        assert_eq!(changes.take_port_change(), Some(2));
        assert_eq!(changes.take_port_change(), Some(3));
        assert_eq!(changes.take_port_change(), Some(4));
        assert_eq!(changes.take_port_change(), Some(5));
        assert_eq!(changes.take_port_change(), Some(6));
        assert_eq!(changes.take_port_change(), None);
    }

    #[test]
    fn test_hub_interrupt_3_hub_empty_port() {
        let mut buf: [u8; _] = [0b0000_0001, 0b0000_0000, 0b1000_0000];
        let mut changes = HubInterrupt(&mut buf);
        assert_eq!(changes.take_hub_change(), true);
        // empty byte
        assert_eq!(changes.take_port_change(), Some(22));
        assert_eq!(changes.take_port_change(), None);
    }
}
