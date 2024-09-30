use std::{ptr, ops::Deref};

use super::{UnsafeRwLockReadGuard, UnsafeRwLock};

impl <'a, T> UnsafeRwLockReadGuard<'a, T> {
    pub fn new(lock: &'a UnsafeRwLock<T>, is_downgraded: bool) -> Self {
        unsafe {
            if is_downgraded {
                let writes = &mut *lock.state.writes.get();
                *writes += 1;
            } else {
                let reads = &mut *lock.state.reads.get();
                *reads += 1;
            }
        }

        Self { lock, is_downgraded }
    }
}

impl<'a, T> Deref for UnsafeRwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if self.is_downgraded {
                &***self.lock.state.write_lock.get()
            } else {
                &***self.lock.state.read_lock.get()
            }
        }
    }
}

impl <'a, T> Drop for UnsafeRwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if self.is_downgraded {
                let writes = &mut *self.lock.state.writes.get();
                *writes -= 1;

                // If there are no more active write locks, drop the write lock
                if *writes == 0 {
                    let _ = Box::from_raw(*self.lock.state.write_lock.get());
                    *self.lock.state.write_lock.get() = ptr::null_mut();
                }
            } else {
                let reads = &mut *self.lock.state.reads.get();
                *reads -= 1;

                // If there are no more active read locks, drop the read lock
                if *reads == 0 {
                    let _ = Box::from_raw(*self.lock.state.read_lock.get());
                    *self.lock.state.read_lock.get() = ptr::null_mut();
                }
            }
        }
    }
}
