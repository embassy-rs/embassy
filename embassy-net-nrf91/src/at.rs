use crate::{Error, Control};

// Drives the control loop of the modem based on declarative configuration.
pub struct AtDriver<'a> {
    control: Control<'a>,
    config: Config,
}

pub struct Config {
    pub network: NetworkConfig,
}

pub struct NetworkConfig {
    pub apn: &'static str,
    pub prot: AuthProtection,
    pub userid: &'static str,
    pub password: &'static str,
}

#[repr(u8)]
pub enum AuthProtection {
    None = 0,
    Pap = 1,
    Chap = 2,
}

impl<'a> AtDriver<'a> {
    pub async fn new(control: Control<'a>, config: Config) -> Result<Self, Error> {
        control.wait_init().await;
        Ok(Self {
            control,
            config,
        })
    }

    async fn setup(&self) -> Result<(), Error> {

    }

    pub fn run(&self, stack: Stack<crate::NetDriver<'static>>) -> ! {
        loop {

        }
    }
}
