use crate::mac::runner::Runner;

#[derive(Debug)]
pub struct Error {
    pub status: u32,
}

pub struct Control<'a> {
    runner: &'a Runner,
}

impl<'a> Control<'a> {
        pub(crate) fn new(runner: &'a Runner) -> Self {
            Self { runner: runner }
        }

    pub async fn init(&mut self) {
        // TODO
    }
}
