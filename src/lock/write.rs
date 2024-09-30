use std::{ptr, ops::{Deref, DerefMut}};

use super::{UnsafeRwLockWriteGuard, UnsafeRwLock};

impl <'a, T> UnsafeRwLockWriteGuard<'a, T> {
    pub fn new(lock: &'a UnsafeRwLock<T>) -> Self {
        unsafe {
            let writes = &mut *lock.state.writes.get();
            *writes += 1;
        }

        Self { lock }
    }
}

impl<'a, T> Deref for UnsafeRwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &***self.lock.state.write_lock.get()
        }
    }
}

impl<'a, T> DerefMut for UnsafeRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut ***self.lock.state.write_lock.get()
        }
    }
}

impl <'a, T> Drop for UnsafeRwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let writes = &mut *self.lock.state.writes.get();
            *writes -= 1;

            // If there are no more active write locks, drop the write lock
            if *writes == 0 {
                let _ = Box::from_raw(*self.lock.state.write_lock.get());
                *self.lock.state.write_lock.get() = ptr::null_mut();
            }
        }
    }
}
