pub struct Config {}

impl Config {
    pub fn new() -> Self {
        Config {}
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

pub unsafe fn init(_config: Config) {}
