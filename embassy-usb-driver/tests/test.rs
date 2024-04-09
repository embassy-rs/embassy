use embassy_usb_driver::*;

struct TestEndpointIn {
    info: EndpointInfo,
    writes: Vec<Vec<u8>>,
}

impl Endpoint for TestEndpointIn {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {}
}

impl EndpointInSinglePacket for TestEndpointIn {
    async fn write_one_packet(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.writes.push(buf.to_vec());
        Ok(())
    }
}

#[test]
fn endpoint_in_single_packet() -> Result<(), EndpointError> {
    let mut endpoint = TestEndpointIn {
        info: EndpointInfo {
            addr: EndpointAddress::from(0x81),
            ep_type: EndpointType::Interrupt,
            max_packet_size: 64,
            interval_ms: 40,
        },
        writes: Vec::new(),
    };

    // Write 250 bytes.
    let buf: [u8; 250] = std::array::from_fn(|n| n as u8);
    embassy_futures::block_on(endpoint.write(&buf))?;

    // The data should have been written as 3 64-byte packets, followed by a 58-byte packet.
    let expected: [Vec<u8>; 4] = [
        (0..64).collect(),
        (64..128).collect(),
        (128..192).collect(),
        (192..250).collect(),
    ];
    assert_eq!(endpoint.writes, expected);

    // A 0-length write should result in a 0-length call to write_one_packet()
    endpoint.writes = Vec::new();
    let buf: [u8; 0] = [];
    embassy_futures::block_on(endpoint.write(&buf))?;
    let expected: [Vec<u8>; 1] = [Vec::new()];
    assert_eq!(endpoint.writes, expected);

    Ok(())
}
