use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::rwlock::RwLock;
use futures_executor::block_on;

#[futures_test::test]
async fn test_rwlock_read() {
    let lock = RwLock::<NoopRawMutex, _>::new(5);

    {
        let read_guard = lock.read().await;
        assert_eq!(*read_guard, 5);
    }

    {
        let read_guard = lock.read().await;
        assert_eq!(*read_guard, 5);
    }
}

#[futures_test::test]
async fn test_rwlock_write() {
    let lock = RwLock::<NoopRawMutex, _>::new(5);

    {
        let mut write_guard = lock.write().await;
        *write_guard = 10;
    }

    {
        let read_guard = lock.read().await;
        assert_eq!(*read_guard, 10);
    }
}

#[futures_test::test]
async fn test_rwlock_try_read() {
    let lock = RwLock::<NoopRawMutex, _>::new(5);

    {
        let read_guard = lock.try_read().unwrap();
        assert_eq!(*read_guard, 5);
    }

    {
        let read_guard = lock.try_read().unwrap();
        assert_eq!(*read_guard, 5);
    }
}

#[futures_test::test]
async fn test_rwlock_try_write() {
    let lock = RwLock::<NoopRawMutex, _>::new(5);

    {
        let mut write_guard = lock.try_write().unwrap();
        *write_guard = 10;
    }

    {
        let read_guard = lock.try_read().unwrap();
        assert_eq!(*read_guard, 10);
    }
}

#[futures_test::test]
async fn test_rwlock_fairness() {
    let lock = RwLock::<NoopRawMutex, _>::new(5);

    let read1 = lock.read().await;
    let read2 = lock.read().await;

    let write_fut = lock.write();
    futures_util::pin_mut!(write_fut);

    assert!(futures_util::poll!(write_fut.as_mut()).is_pending());

    drop(read1);
    drop(read2);

    assert!(futures_util::poll!(write_fut.as_mut()).is_ready());
}
