pub fn to_u16(buf: &[u8]) -> u16 {
    ((buf[1] as u16) << 8) | buf[0] as u16
}

pub fn to_u32(buf: &[u8]) -> u32 {
    ((buf[0] as u32) << 0) + ((buf[1] as u32) << 8) + ((buf[2] as u32) << 16) + ((buf[3] as u32) << 24)
}
