//! Helper utility to configure a specific modem context.
use core::net::IpAddr;
use core::str::FromStr;

use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Timer};
use heapless::Vec;

/// Provides a higher level API for controlling a given context.
pub struct Control<'a> {
    control: crate::Control<'a>,
    cid: u8,
}

/// Set of LTE bands (1..=88) as a bitmap; bit 0 of byte 0 is band 1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bands([u8; 11]);

impl Bands {
    /// Highest band number supported by %XBANDLOCK.
    pub const MAX_BAND: u8 = 88;

    /// Empty set.
    pub const fn new() -> Self {
        Self([0; 11])
    }

    /// Set with the given band numbers enabled. Panics if a band is not 1..=88.
    pub const fn from_slice(bands: &[u8]) -> Self {
        let mut set = Self::new();
        let mut i = 0;
        while i < bands.len() {
            set.set(bands[i], true);
            i += 1;
        }
        set
    }

    /// Set from its raw bitmap.
    pub const fn from_bytes(bytes: [u8; 11]) -> Self {
        Self(bytes)
    }

    /// The raw bitmap.
    pub const fn to_bytes(self) -> [u8; 11] {
        self.0
    }

    /// Enable or disable a band. Panics if `band` is not 1..=88.
    pub const fn set(&mut self, band: u8, enabled: bool) {
        core::assert!(band >= 1 && band <= Self::MAX_BAND, "band must be 1..=88");
        let bit = (band - 1) as usize;
        if enabled {
            self.0[bit / 8] |= 1 << (bit % 8);
        } else {
            self.0[bit / 8] &= !(1 << (bit % 8));
        }
    }

    /// Whether a band is enabled. Panics if `band` is not 1..=88.
    pub const fn is_enabled(&self, band: u8) -> bool {
        core::assert!(band >= 1 && band <= Self::MAX_BAND, "band must be 1..=88");
        let bit = (band - 1) as usize;
        self.0[bit / 8] & (1 << (bit % 8)) != 0
    }

    /// Whether no band is enabled.
    pub const fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < self.0.len() {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl Default for Bands {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Bands {
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(f, "[");
        let mut first = true;
        for band in 1..=Self::MAX_BAND {
            if self.is_enabled(band) {
                defmt::write!(f, "{}B{}", if first { "" } else { "," }, band);
                first = false;
            }
        }
        defmt::write!(f, "]");
    }
}

/// Builds the %XBANDLOCK bit string: rightmost bit is band 1, leading zeroes omitted.
fn band_mask(bands: Bands, buf: &mut [u8; Bands::MAX_BAND as usize]) -> Result<&[u8], Error> {
    let mut max = 0;
    for band in 1..=Bands::MAX_BAND {
        if bands.is_enabled(band) {
            max = band;
        }
    }
    if max == 0 {
        return Err(Error::EmptyBands);
    }
    for band in 1..=max {
        buf[(max - band) as usize] = if bands.is_enabled(band) { b'1' } else { b'0' };
    }
    Ok(&buf[..max as usize])
}

/// Configuration for a given context
pub struct Config<'a> {
    /// Desired APN address.
    pub apn: &'a [u8],
    /// Desired authentication protocol.
    pub auth_prot: AuthProt,
    /// Credentials.
    pub auth: Option<(&'a [u8], &'a [u8])>,
    /// SIM pin
    pub pin: Option<&'a [u8]>,
    /// LTE band lock; None leaves any existing lock unchanged.
    pub band_lock: Option<BandLock>,
}

/// LTE band lock (AT%XBANDLOCK).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BandLock {
    /// Remove any existing band lock.
    Remove,
    /// Lock to the given bands until the next modem reset.
    Runtime(Bands),
    /// Lock to the given bands, persisted in modem flash across resets.
    Permanent(Bands),
}

/// Authentication protocol.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AuthProt {
    /// No authentication.
    None = 0,
    /// PAP authentication.
    Pap = 1,
    /// CHAP authentication.
    Chap = 2,
}

/// Error returned by control.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Not enough space for command.
    BufferTooSmall,
    /// Error parsing response from modem.
    AtParseError,
    /// Error parsing IP addresses.
    AddrParseError,
    /// Band lock requested with an empty band set.
    EmptyBands,
}

impl From<at_commands::parser::ParseError> for Error {
    fn from(_: at_commands::parser::ParseError) -> Self {
        Self::AtParseError
    }
}

/// Status of a given context.
#[derive(PartialEq, Debug)]
pub struct Status {
    /// Attached to APN or not.
    pub attached: bool,
    /// IP if assigned.
    pub ip: Option<IpAddr>,
    /// Gateway if assigned.
    pub gateway: Option<IpAddr>,
    /// DNS servers if assigned.
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
    /// Create a new instance of a control handle for a given context.
    ///
    /// Will wait for the modem to be initialized if not.
    pub async fn new(control: crate::Control<'a>, cid: u8) -> Self {
        control.wait_init().await;
        Self { control, cid }
    }

    /// Perform a raw AT command
    pub async fn at_command(&self, req: &[u8], resp: &mut [u8]) -> usize {
        self.control.at_command(req, resp).await
    }

    /// Configures the modem with the provided config.
    ///
    /// NOTE: This will disconnect the modem from any current APN and should not
    /// be called if the configuration has not been changed.
    ///
    /// After configuring, invoke [`enable()`] to activate the configuration.
    pub async fn configure(&self, config: &Config<'_>) -> Result<(), Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CFUN")
            .with_int_parameter(0)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        if let Some(band_lock) = config.band_lock {
            self.set_band_lock(band_lock).await?;
        }

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGDCONT")
            .with_int_parameter(self.cid)
            .with_string_parameter("IP")
            .with_string_parameter(config.apn)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        // info!("RES1: {}", unsafe { core::str::from_utf8_unchecked(&buf[..n]) });
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
        // info!("RES2: {}", unsafe { core::str::from_utf8_unchecked(&buf[..n]) });
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        if let Some(pin) = config.pin {
            let op = CommandBuilder::create_set(&mut cmd, true)
                .named("+CPIN")
                .with_string_parameter(pin)
                .finish()
                .map_err(|_| Error::BufferTooSmall)?;
            let _ = self.control.at_command(op, &mut buf).await;
            // Ignore ERROR which means no pin required
        }

        Ok(())
    }

    /// Set or remove the LTE band lock. Must be issued while the modem is
    /// deactivated; [`configure()`] applies [`Config::band_lock`] automatically.
    pub async fn set_band_lock(&self, band_lock: BandLock) -> Result<(), Error> {
        let mut cmd: [u8; 128] = [0; 128];
        let mut buf: [u8; 128] = [0; 128];

        let builder = CommandBuilder::create_set(&mut cmd, true).named("%XBANDLOCK");
        let op = match band_lock {
            BandLock::Remove => builder.with_int_parameter(0).finish(),
            BandLock::Permanent(bands) | BandLock::Runtime(bands) => {
                let mut mask = [0u8; Bands::MAX_BAND as usize];
                let mask = band_mask(bands, &mut mask)?;
                let operation = match band_lock {
                    BandLock::Permanent(_) => 1,
                    _ => 2,
                };
                builder.with_int_parameter(operation).with_string_parameter(mask).finish()
            }
        }
        .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;
        Ok(())
    }

    /// Attach to the PDN
    pub async fn attach(&self) -> Result<(), Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];
        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGATT")
            .with_int_parameter(1)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;
        Ok(())
    }

    /// Read current connectivity status for modem.
    pub async fn detach(&self) -> Result<(), Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];
        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGATT")
            .with_int_parameter(0)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;
        Ok(())
    }

    async fn attached(&self) -> Result<bool, Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_query(&mut cmd, true)
            .named("+CGATT")
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (res,) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGATT: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\nOK")
            .finish()?;
        Ok(res == 1)
    }

    /// Read current connectivity status for modem.
    pub async fn status(&self) -> Result<Status, Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_query(&mut cmd, true)
            .named("+CGATT")
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (res,) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGATT: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\nOK")
            .finish()?;
        let attached = res == 1;
        if !attached {
            return Ok(Status {
                attached,
                ip: None,
                gateway: None,
                dns: Vec::new(),
            });
        }

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGPADDR")
            .with_int_parameter(self.cid)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (_, ip1, _ip2) = CommandParser::parse(&buf[..n])
            .expect_identifier(b"+CGPADDR: ")
            .expect_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_identifier(b"\r\nOK")
            .finish()?;

        let ip = if let Some(ip) = ip1 {
            let ip = IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?;
            Some(ip)
        } else {
            None
        };

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CGCONTRDP")
            .with_int_parameter(self.cid)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        let (_cid, _bid, _apn, _mask, gateway, dns1, dns2, _, _, _, _, _mtu) = CommandParser::parse(&buf[..n])
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
            .expect_identifier(b"\r\nOK")
            .finish()?;

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
            if !ip.is_empty() {
                dns.push(IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?)
                    .unwrap();
            }
        }

        if let Some(ip) = dns2 {
            if !ip.is_empty() {
                dns.push(IpAddr::from_str(ip).map_err(|_| Error::AddrParseError)?)
                    .unwrap();
            }
        }

        Ok(Status {
            attached,
            ip,
            gateway,
            dns,
        })
    }

    async fn wait_attached(&self) -> Result<Status, Error> {
        while !self.attached().await? {
            Timer::after(Duration::from_secs(1)).await;
        }
        let status = self.status().await?;
        Ok(status)
    }

    /// Disable modem
    pub async fn disable(&self) -> Result<(), Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CFUN")
            .with_int_parameter(0)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        Ok(())
    }

    /// Enable modem
    pub async fn enable(&self) -> Result<(), Error> {
        let mut cmd: [u8; 256] = [0; 256];
        let mut buf: [u8; 256] = [0; 256];

        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("+CFUN")
            .with_int_parameter(1)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;

        // Make modem survive PDN detaches
        let op = CommandBuilder::create_set(&mut cmd, true)
            .named("%XPDNCFG")
            .with_int_parameter(1)
            .finish()
            .map_err(|_| Error::BufferTooSmall)?;
        let n = self.control.at_command(op, &mut buf).await;
        CommandParser::parse(&buf[..n]).expect_identifier(b"OK").finish()?;
        Ok(())
    }

    /// Run a control loop for this context, ensuring that reaattach is handled.
    pub async fn run<F: Fn(&Status)>(&self, reattach: F) -> Result<(), Error> {
        self.enable().await?;
        let status = self.wait_attached().await?;
        let mut fd = self.control.open_raw_socket().await;
        self.control.set_link_state(LinkState::Up);
        reattach(&status);

        loop {
            if !self.attached().await? {
                self.control.set_link_state(LinkState::Down);
                trace!("detached");

                self.control.close_raw_socket(fd).await;
                let status = self.wait_attached().await?;
                trace!("attached");
                fd = self.control.open_raw_socket().await;
                self.control.set_link_state(LinkState::Up);
                reattach(&status);
            }
            Timer::after(Duration::from_secs(10)).await;
        }
    }
}
