#[cfg(test)]
mod lock_tests {
    use super::super::*;

    #[test]
    fn test_read() {
        let lock = UnsafeRwLock::new_from(5);

        let read_guard = lock.read();
        assert_eq!(*read_guard, 5);
    }

    #[test]
    fn test_write() {
        let lock = UnsafeRwLock::new_from(5);

        let mut write_guard = lock.write();
        *write_guard = 10;

        let write_guard = lock.write();
        assert_eq!(*write_guard, 10);
    }

    #[test]
    fn test_write_read() {
        let lock = UnsafeRwLock::new_from(5);

        let mut write_guard = lock.write();
        *write_guard = 10;

        let read_guard = lock.read();
        assert_eq!(*read_guard, 10);
    }

    #[test]
    fn read_after_read() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.read();
        let _2 = lock.read();
    }

    #[test]
    fn write_after_write() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.write();
        let _2 = lock.write();
    }

    #[test]
    fn read_after_write() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.write();
        let _2 = lock.read();
    }

    #[test]
    #[should_panic(expected = "Cannot acquire write lock while read lock is active")]
    fn write_after_read() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.read();
        let _2 = lock.write();
    }

    #[test]
    fn drop_read() {
        let lock = UnsafeRwLock::new_from(0);

        {
            let _1 = lock.read();
        }

        let _2 = lock.read();
    }

    #[test]
    fn drop_write() {
        let lock = UnsafeRwLock::new_from(0);

        {
            let _1 = lock.write();
        }

        let _2 = lock.write();
    }

    #[test]
    fn drop_read_write() {
        let lock = UnsafeRwLock::new_from(0);

        {
            let _1 = lock.read();
        }

        let _2 = lock.write();
    }

    #[test]
    fn test_count_after_drop() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.read();
        let _2 = lock.read();

        drop(_1);
        drop(_2);

        let _3 = lock.write();
        let _4 = lock.write();

        unsafe {
            assert_eq!(*lock.state.reads.get(), 0);
            assert_eq!(*lock.state.writes.get(), 2);
        }

        drop(_3);
        drop(_4);

        let _5 = lock.read();
        let _6 = lock.read();

        unsafe {
            assert_eq!(*lock.state.reads.get(), 2);
            assert_eq!(*lock.state.writes.get(), 0);
        }

        drop(_5);
        drop(_6);

        let _7 = lock.write();
        let _8 = lock.read();

        unsafe {
            assert_eq!(*lock.state.reads.get(), 0);
            assert_eq!(*lock.state.writes.get(), 2);
        }

        drop(_7);
        drop(_8);

        unsafe {
            assert_eq!(*lock.state.reads.get(), 0);
            assert_eq!(*lock.state.writes.get(), 0);
        }
    }

    #[test]
    fn test_locks_after_drop() {
        let lock = UnsafeRwLock::new_from(0);

        let _1 = lock.read();
        let _2 = lock.read();

        unsafe {
            assert!(!(*lock.state.read_lock.get()).is_null());
            assert!((*lock.state.write_lock.get()).is_null());
        }

        drop(_1);
        drop(_2);

        let _1 = lock.write();
        let _2 = lock.write();

        unsafe {
            assert!((*lock.state.read_lock.get()).is_null());
            assert!(!(*lock.state.write_lock.get()).is_null());
        }

        drop(_1);
        drop(_2);

        let _1 = lock.write();
        let _2 = lock.read();

        unsafe {
            assert!((*lock.state.read_lock.get()).is_null());
            assert!(!(*lock.state.write_lock.get()).is_null());
        }

        drop(_1);
        drop(_2);

        unsafe {
            assert!((*lock.state.read_lock.get()).is_null());
            assert!((*lock.state.write_lock.get()).is_null());
        }
    }

    #[test]
    fn test_downgrade() {
        let lock = UnsafeRwLock::new_from(0);

        {
            let read_guard = lock.read();
            assert_eq!(read_guard.is_downgraded, false);
        }

        let _write_guard = lock.write();
        let read_guard = lock.read();

        assert_eq!(read_guard.is_downgraded, true);
    }

    #[test]
    fn test_inner_lock_after_drop() {
        let item = Arc::new(RwLock::new(0));

        {
            let lock = UnsafeRwLock::new(item.clone());
            let _write_guard = lock.write();
        }

       let _guard = item.try_write().unwrap();
    }

    #[test]
    #[should_panic(expected = "Would block")]
    fn test_inner_write_after_write() {
        let lock = UnsafeRwLock::new_from(0);

        let _guard = lock.write();
        let _guard_2 = match lock.inner.try_write() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => panic!("Would block"),
            Err(e) => Err(e).unwrap()
        };
    }

    #[test]
    fn test_clone() {
        let lock = UnsafeRwLock::new_from(0);
        let _1 = lock.write();
        let _2 = lock.write();

        let clone = lock.clone();
        let _3 = clone.read();
        let _4 = clone.read();

        unsafe {
            assert_eq!(*lock.state.reads.get(), 0);
            assert_eq!(*lock.state.writes.get(), 4);
        }

        {
            let clone = clone.clone();
            let _5 = clone.read();
            let _6 = clone.write();

            unsafe {
                assert_eq!(*lock.state.reads.get(), 0);
                assert_eq!(*lock.state.writes.get(), 6);
            }

            drop(_5);
            drop(_6);
            drop(clone);
            
            unsafe {
                assert_eq!(*lock.state.reads.get(), 0);
                assert_eq!(*lock.state.writes.get(), 4);
            }
        }
    }
}
