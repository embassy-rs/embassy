pub trait MutexKind {
    type Mutex<T>: super::Mutex<Data = T>;
}

pub enum CriticalSection {}
impl MutexKind for CriticalSection {
    type Mutex<T> = super::CriticalSectionMutex<T>;
}

#[cfg(any(cortex_m, feature = "std"))]
pub enum ThreadMode {}
#[cfg(any(cortex_m, feature = "std"))]
impl MutexKind for ThreadMode {
    type Mutex<T> = super::ThreadModeMutex<T>;
}

pub enum Noop {}
impl MutexKind for Noop {
    type Mutex<T> = super::NoopMutex<T>;
}
