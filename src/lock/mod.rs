use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::cell::UnsafeCell;
use std::ptr;

mod read;
mod write;
mod tests;

/// Wrapper around a RwLock (Arc<RwLock<T>>) that allows for multiple read locks or multiple write locks (on a single thread), but not
/// both at the same time. It is achieved by reusing the same inner lock, so it is multi-thread safe.
///
/// Writes and reads can coexist in the case that a read is acquired after a write (it gets downgraded)
///
/// # Usage
/// User should make sure there is no Arc<RwLock<T>> clone while having this lock on the same
/// thread, otherwise it may cause deadlocks.
///
/// You should use this lock with caution, as it enables multiple mutable references to the same
/// data.
///
/// It should be constructed after receiving/moving some thread-safe data, and should stay in the same
/// thread. 
///
/// # Errors
/// Will return an error when trying to acquire a write lock while a read lock is active.
///
/// # Panics
/// Will panic when inner lock panics. (e.g. poisoned lock)
pub struct UnsafeRwLock<T> where T: 'static {
    inner: Arc<RwLock<T>>, // The actual RwLock we're managing
    state: Rc<LockState<T>>, // The state of the lock   
}

struct LockState<T> where T: 'static {
    read_lock: UnsafeCell<*mut RwLockReadGuard<'static, T>>, // Pointer to the read lock
    write_lock: UnsafeCell<*mut RwLockWriteGuard<'static, T>>, // Pointer to the write lock
    reads: UnsafeCell<usize>, // Number of active read locks
    writes: UnsafeCell<usize>, // Number of active write locks
}

impl<T> LockState<T> {
    pub fn new() -> Self {
        Self {
            read_lock: UnsafeCell::new(ptr::null_mut()),
            write_lock: UnsafeCell::new(ptr::null_mut()),
            reads: UnsafeCell::new(0),
            writes: UnsafeCell::new(0)
        }
    }
}

pub struct UnsafeRwLockReadGuard<'a, T> where T:'static {
    lock: &'a UnsafeRwLock<T>,
    is_downgraded: bool, // Whether this guard was downgraded from a write lock
}

pub struct UnsafeRwLockWriteGuard<'a, T> where T: 'static {
    lock: &'a UnsafeRwLock<T>,
}

impl<T> UnsafeRwLock<T> {
    /// Create a new UnsafeRwLock from an Arc<RwLock<T>>
    pub fn new(lock: Arc<RwLock<T>>) -> Self {
        Self {
            inner: lock,
            state: Rc::new(LockState::new())
        }
    }

    /// Wrap value T in an Arc<RwLock<T>>, T itself should not be any kind of lock
    pub fn new_from(value: T) -> Self {
        Self::new(Arc::new(RwLock::new(value)))
    }

    /// Acquires a read lock and stores it
    pub fn read(&self) -> UnsafeRwLockReadGuard<T> {
        unsafe {
            // If we already have a write lock, return a reference to it
            if !(*self.state.write_lock.get()).is_null() {
                return UnsafeRwLockReadGuard::new(self, true)
            }

            // If we don't have a read lock yet, acquire it and store it
            if (*self.state.read_lock.get()).is_null() {
                let lock = ManuallyDrop::new(self.inner.read().unwrap());
                let guard = Box::new(lock);
                let guard = Box::into_raw(guard);
                 *self.state.read_lock.get() = std::mem::transmute::<_, *mut RwLockReadGuard<'static, T>>(guard);
            }

            // Return the read lock
            UnsafeRwLockReadGuard::new(self, false)
        }
    }

    /// Acquires a write lock and stores it
    ///
    /// # Errors
    /// Will return an error when a read lock exists
    pub fn write(&self) -> Result<UnsafeRwLockWriteGuard<T>, String> {
        unsafe {
            // If we already have a read lock, panic
            if !(*self.state.read_lock.get()).is_null() {
                return Err("Cannot acquire write lock while read lock is active".to_string())
            }

            // If we don't have a write lock yet, acquire it and store it
            if (*self.state.write_lock.get()).is_null() {
                let lock = ManuallyDrop::new(self.inner.write().unwrap());
                let guard = Box::new(lock);
                let guard = Box::into_raw(guard);
                *self.state.write_lock.get() = std::mem::transmute::<_, *mut RwLockWriteGuard<'static, T>>(guard);    
            }

            // Return the write lock
            Ok(UnsafeRwLockWriteGuard::new(self))
        }
    }
}

impl<T> Drop for UnsafeRwLock<T> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.state) > 1 {
            return
        }

        unsafe {
            // Clean up any raw pointers we have stored
            if !(*self.state.read_lock.get()).is_null() {
                let _ = Box::from_raw(*self.state.read_lock.get());
            }
            if !(*self.state.write_lock.get()).is_null() {
                let _ = Box::from_raw(*self.state.write_lock.get());
            }
        }
    }
}

impl<T> Clone for UnsafeRwLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            state: self.state.clone()
        }
    }
}
