use crate::descriptor::descriptor_type;
use crate::types::EndpointAddress;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadError;

pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn eof(&self) -> bool {
        self.data.is_empty()
    }

    pub fn read<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
        let n = self.data.get(0..N).ok_or(ReadError)?;
        self.data = &self.data[N..];
        Ok(n.try_into().unwrap())
    }

    pub fn read_u8(&mut self) -> Result<u8, ReadError> {
        Ok(u8::from_le_bytes(self.read()?))
    }
    pub fn read_u16(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.read()?))
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], ReadError> {
        let res = self.data.get(0..len).ok_or(ReadError)?;
        self.data = &self.data[len..];
        Ok(res)
    }

    pub fn read_descriptors(&mut self) -> DescriptorIter<'_, 'a> {
        DescriptorIter { r: self }
    }
}

pub struct DescriptorIter<'a, 'b> {
    r: &'a mut Reader<'b>,
}

impl<'a, 'b> Iterator for DescriptorIter<'a, 'b> {
    type Item = Result<(u8, Reader<'a>), ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.r.eof() {
            return None;
        }

        let len = match self.r.read_u8() {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };
        let type_ = match self.r.read_u8() {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };
        let data = match self.r.read_slice(len as usize - 2) {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok((type_, Reader::new(data))))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointInfo {
    pub configuration: u8,
    pub interface: u8,
    pub interface_alt: u8,
    pub ep_address: EndpointAddress,
}

pub fn foreach_endpoint(data: &[u8], mut f: impl FnMut(EndpointInfo)) -> Result<(), ReadError> {
    let mut ep = EndpointInfo {
        configuration: 0,
        interface: 0,
        interface_alt: 0,
        ep_address: EndpointAddress::from(0),
    };
    for res in Reader::new(data).read_descriptors() {
        let (kind, mut r) = res?;
        match kind {
            descriptor_type::CONFIGURATION => {
                let _total_length = r.read_u16()?;
                let _total_length = r.read_u8()?;
                ep.configuration = r.read_u8()?;
            }
            descriptor_type::INTERFACE => {
                ep.interface = r.read_u8()?;
                ep.interface_alt = r.read_u8()?;
            }
            descriptor_type::ENDPOINT => {
                ep.ep_address = EndpointAddress::from(r.read_u8()?);
                f(ep)
            }
            _ => {}
        }
    }
    Ok(())
}
