use core::net::IpAddr;
use heapless::String;
use core::str::FromStr;
use core::fmt::Write;

/// Provides a higher level API for configuring and reading information for a given
/// context id.
pub struct Control<'a> {
    control: crate::Control<'a>,
    cid: u8,
}

pub struct Config<'a> {
    pub gateway: &'a str,
    pub auth_prot: AuthProt,
    pub auth: Option<(&'a str, &'a str)>,
}

#[repr(u8)]
pub enum AuthProt {
    None = 0,
    Pap = 1,
    Chap = 2,
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    BufferTooSmall,
    AtCommand,
    AddrParseError,
    Format,
}

impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        Self::Format
    }
}

#[derive(PartialEq, Debug)]
pub struct Status {
    pub attached: bool,
    pub ip: Option<IpAddr>,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Status {
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(f, "attached: {}", self.attached);
        if let Some(ip) = &self.ip {
            defmt::write!(f, ", ip: {}", defmt::Debug2Format(&ip));
        }
    }
}

impl<'a> Control<'a> {
    pub async fn new(control: crate::Control<'a>, cid: u8) -> Self {
        control.wait_init().await;
        Self { control, cid }
    }

    /// Bypass modem configurator
    pub async fn at_command(&self, req: &[u8], resp: &mut [u8]) -> usize {
        self.control.at_command(req, resp).await
    }

    /// Configures the modem with the provided config.
    pub async fn configure(&self, config: Config<'_>) -> Result<(), Error> {
        let mut cmd: String<128> = String::new();
        let mut buf: [u8; 256] = [0; 256];

        write!(cmd, "AT+CGDCONT={},\"IP\",\"{}\"", self.cid, config.gateway).map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(cmd.as_bytes(), &mut buf).await;
        let mut res = &buf[..n];
        let res = split_field(&mut res);
        if res != b"OK" {
            return Err(Error::AtCommand)
        }
        cmd.clear();

        write!(cmd, "AT+CGAUTH={},{}", self.cid, config.auth_prot as u8)?;
        if let Some((username, password)) = config.auth {
            write!(cmd, ",\"{}\",\"{}\"", username, password).map_err(|_| Error::BufferTooSmall)?;
        }
        let n = self.control.at_command(cmd.as_bytes(), &mut buf).await;
        let mut res = &buf[..n];
        let res = split_field(&mut res);
        if res != b"OK" {
            return Err(Error::AtCommand)
        }
        cmd.clear();

        let n = self.control.at_command(b"AT+CFUN=1", &mut buf).await;
        let mut res = &buf[..n];
        let res = split_field(&mut res);
        if res != b"OK" {
            return Err(Error::AtCommand);
        }

        Ok(())
    }

    pub async fn status(&self) -> Result<Status, Error> {
        let mut buf: [u8; 256] = [0; 256];
        let n = self.control.at_command(b"AT+CGATT?", &mut buf).await;
        let mut res = &buf[..n];
        pop_prefix(&mut res, b"+CGATT: ");
        let res = split_field(&mut res);
        let attached = res == b"1";

        if !attached {
            return Ok(Status { attached, ip: None })
        }

        let mut s: String<128> = String::new();
        write!(s, "AT+CGPADDR={}", self.cid)?;
        let n = self.control.at_command(s.as_bytes(), &mut buf).await;
        let mut res = &buf[..n];
        s.clear();

        write!(s, "+CGPADDR: {},", self.cid)?;

        if s.len() > res.len() {
            let res = split_field(&mut res);
            if res == b"OK" {
                Ok(Status { attached, ip: None })
            } else {
                Err(Error::AtCommand)
            }
        } else {
            pop_prefix(&mut res, s.as_bytes());

            let ip = split_field(&mut res);
            if !ip.is_empty() {
                let ip = IpAddr::from_str(unsafe { core::str::from_utf8_unchecked(ip) }).map_err(|_| Error::AddrParseError)?;
                self.control.open_raw_socket().await;
                Ok(Status { attached, ip: Some(ip) })
            } else {
                Ok(Status { attached, ip: None })
            }
        }
    }
}

pub(crate) fn is_whitespace(char: u8) -> bool {
    match char {
        b'\r' | b'\n' | b' ' => true,
        _ => false,
    }
}

pub(crate) fn is_separator(char: u8) -> bool {
    match char {
        b',' | b'\r' | b'\n' | b' ' => true,
        _ => false,
    }
}

pub(crate) fn split_field<'a>(data: &mut &'a [u8]) -> &'a [u8] {
    while !data.is_empty() && is_whitespace(data[0]) {
        *data = &data[1..];
    }

    if data.is_empty() {
        return &[];
    }

    if data[0] == b'"' {
        let data2 = &data[1..];
        let end = data2.iter().position(|&x| x == b'"').unwrap_or(data2.len());
        let field = &data2[..end];
        let mut rest = &data2[data2.len().min(end + 1)..];
        if rest.first() == Some(&b'\"') {
            rest = &rest[1..];
        }
        while !rest.is_empty() && is_separator(rest[0]) {
            rest = &rest[1..];
        }
        *data = rest;
        field
    } else {
        let end = data.iter().position(|&x| is_separator(x)).unwrap_or(data.len());
        let field = &data[0..end];
        let rest = &data[data.len().min(end + 1)..];
        *data = rest;
        field
    }
}

pub(crate) fn pop_prefix(data: &mut &[u8], prefix: &[u8]) {
    assert!(data.len() >= prefix.len());
    assert!(&data[..prefix.len()] == prefix);
    *data = &data[prefix.len()..];
}
