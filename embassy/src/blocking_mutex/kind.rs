use super::{CriticalSectionMutex, Mutex, NoopMutex, ThreadModeMutex};

pub trait MutexKind {
    type Mutex<T>: Mutex<Data = T>;
}

pub enum CriticalSection {}
impl MutexKind for CriticalSection {
    type Mutex<T> = CriticalSectionMutex<T>;
}

pub enum ThreadMode {}
impl MutexKind for ThreadMode {
    type Mutex<T> = ThreadModeMutex<T>;
}
pub enum Noop {}
impl MutexKind for Noop {
    type Mutex<T> = NoopMutex<T>;
}
