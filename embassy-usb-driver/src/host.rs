use crate::{EndpointError, EndpointType};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl EndpointDescriptor {
    pub fn ep_type(&self) -> EndpointType {
        match self.attributes & 0x03 {
            0 => EndpointType::Control,
            1 => EndpointType::Isochronous,
            2 => EndpointType::Bulk,
            3 => EndpointType::Interrupt,
            _ => unreachable!(),
        }
    }
}

pub trait USBHostDriverTrait {
    type ChannelIn: ChannelIn;
    type ChannelOut: ChannelOut;

    async fn bus_reset(&mut self);

    async fn wait_for_device_connect(&mut self);

    async fn wait_for_device_disconnect(&mut self);

    // Control request
    async fn request_out(&mut self, addr: u8, bytes: &[u8]);

    async fn request_in(&mut self, addr: u8, bytes: &[u8], dest: &mut [u8]) -> Result<usize, ()>;

    fn reconfigure_channel0(&mut self, max_packet_size: u16, dev_addr: u8) -> Result<(), ()>;

    fn alloc_channel_in(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelIn, ()>;
    fn alloc_channel_out(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelOut, ()>;
}

pub trait ChannelIn {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError>;
}

pub trait ChannelOut {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError>;
}
