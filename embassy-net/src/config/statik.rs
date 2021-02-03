use smoltcp::time::Instant;

use super::*;
use crate::fmt::*;
use crate::{Interface, SocketSet};

pub struct StaticConfigurator {
    config: UpConfig,
}

impl StaticConfigurator {
    pub fn new(config: UpConfig) -> Self {
        Self { config }
    }
}

impl Configurator for StaticConfigurator {
    fn poll(
        &mut self,
        _iface: &mut Interface,
        _sockets: &mut SocketSet,
        _timestamp: Instant,
    ) -> Option<Config> {
        Some(Config::Up(self.config.clone()))
    }
}
