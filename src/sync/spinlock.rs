use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;

pub struct SpinLock<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    were_enabled: bool,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        let flags: u64;
        unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags); }
        let were_enabled = flags & (1 << 9) != 0;

        unsafe { core::arch::asm!("cli"); }

        loop {
            match self.locked.compare_exchange(
                false,
                true,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    break SpinLockGuard {
                        lock: self,
                        were_enabled,
                    }
                }
                Err(_) => {}
            }
        }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        if self.were_enabled {
            unsafe { core::arch::asm!("sti"); }
        }
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.lock.data.get()
        }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.lock.data.get()
        }
    }
}

unsafe impl<T: Send> Sync for SpinLock<T> {

}
unsafe impl<T:Send> Send for SpinLock<T> {

}