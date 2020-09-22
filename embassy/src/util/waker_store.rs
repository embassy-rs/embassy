use core::task::Waker;

pub struct WakerStore {
    waker: Option<Waker>,
}

impl WakerStore {
    pub const fn new() -> Self {
        Self { waker: None }
    }

    pub fn store(&mut self, w: &Waker) {
        match self.waker {
            Some(ref w2) if (w2.will_wake(w)) => {}
            Some(_) => panic!("Waker overflow"),
            None => self.waker = Some(w.clone()),
        }
    }

    pub fn wake(&mut self) {
        self.waker.take().map(|w| w.wake());
    }
}
