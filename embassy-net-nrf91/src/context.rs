use core::net::IpAddr;
use heapless::String;
use core::str::FromStr;
use core::fmt::Write;
use heapless::Vec;
use at_commands::{builder::CommandBuilder, parser::CommandParser};

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
    AtParseError,
    AddrParseError,
    Format,
}

impl From<at_commands::parser::ParseError> for Error {
    fn from(_: at_commands::parser::ParseError) -> Self {
        Self::AtParseError
    }
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
    pub gateway: Option<IpAddr>,
    pub dns: Vec<IpAddr, 2>,
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
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGDCONT")
            .with_int_parameter(self.cid)
            .with_string_parameter("IP")
            .with_string_parameter(config.gateway)
            .finish().map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        let mut op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGAUTH")
            .with_int_parameter(self.cid)
            .with_int_parameter(config.auth_prot as u8);
        if let Some((username, password)) = config.auth {
            op = op.with_string_parameter(username).with_string_parameter(password);
        }
        let op = op.finish().map_err(|_| Error::BufferTooSmall)?;

        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CFUN")
            .with_int_parameter(1)
            .finish().map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        Ok(())
    }

    pub async fn status(&self) -> Result<Status, Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_query(&mut cmd, true)
            .named("+CGATT")
            .finish().map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (res, ) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGATT: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\nOK").finish()?;
        let attached = res == 1;
        if !attached {
            return Ok(Status { attached, ip: None, gateway: None, dns: Vec::new() })
        }

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGPADDR")
            .with_int_parameter(self.cid)
            .finish().map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (_, ip1, ip2, ) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGPADDR: ")
            .expect_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_identifier(b"\r\nOK").finish()?;

        let ip = if let Some(ip) = ip1 {
            let ip = IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?;
            self.control.open_raw_socket().await;
            Some(ip)
        } else {
            None
        };

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGCONTRDP")
            .with_int_parameter(self.cid)
            .finish().map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (_cid, _bid, _apn, _mask, gateway, dns1, dns2, _, _, _, _, mtu) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGCONTRDP: ")
            .expect_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\nOK").finish()?;

        let gateway = if let Some(ip) = gateway {
            if ip.is_empty() {
                None
            } else {
                Some(IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?)
            }
        } else {
            None
        };

        let mut dns = Vec::new();
        if let Some(ip) = dns1 {
            dns.push(IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?).unwrap();
        }

        if let Some(ip) = dns2 {
            dns.push(IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?).unwrap();
        }

        Ok(Status {
            attached,
            ip,
            gateway,
            dns,
        })
    }
}
