/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {}

impl Config {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    todo!()
}
