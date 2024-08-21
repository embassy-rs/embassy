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

pub trait USBHostDriverTrait {
    type ChannelIn;
    type ChannelOut;

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
