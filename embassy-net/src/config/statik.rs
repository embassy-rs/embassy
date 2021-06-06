use smoltcp::time::Instant;

use super::*;
use crate::{Interface, SocketSet};

pub struct StaticConfigurator {
    config: Config,
    returned: bool,
}

impl StaticConfigurator {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            returned: false,
        }
    }
}

impl Configurator for StaticConfigurator {
    fn poll(
        &mut self,
        _iface: &mut Interface,
        _sockets: &mut SocketSet,
        _timestamp: Instant,
    ) -> Event {
        if self.returned {
            Event::NoChange
        } else {
            self.returned = true;
            Event::Configured(self.config.clone())
        }
    }
}
