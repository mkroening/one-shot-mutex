use core::cell::Cell;

use lock_api::{GuardSend, RawMutex, RawMutexFair};

/// A one-shot mutex that panics instead of (dead)locking on contention.
///
/// This mutex allows no contention and panics instead of blocking on [`lock`] if it is already locked.
/// This is useful in situations where contention would be a bug,
/// such as in single-threaded programs that would deadlock on contention.
///
/// This mutex does not implement `Sync`, which permits a slightly more efficient implementation.
/// For a variant that does implement `Sync`, see [`sync::RawOneShotMutex`](crate::sync::RawOneShotMutex).
///
/// This mutex should be used through [`OneShotMutex`].
///
/// [`lock`]: Self::lock
///
/// # Examples
///
/// ```
/// use one_shot_mutex::unsync::OneShotMutex;
///
/// let m: OneShotMutex<i32> = OneShotMutex::new(42);
///
/// // This is equivalent to `X.try_lock().unwrap()`.
/// let x = m.lock();
///
/// // This panics instead of deadlocking.
/// // let x2 = m.lock();
///
/// // Once we unlock the mutex, we can lock it again.
/// drop(x);
/// let x = m.lock();
/// ```
pub struct RawOneShotMutex {
    lock: Cell<bool>,
}

impl RawOneShotMutex {
    pub const fn new() -> Self {
        Self::INIT
    }
}

impl Default for RawOneShotMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl RawMutex for RawOneShotMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        lock: Cell::new(false),
    };

    type GuardMarker = GuardSend;

    #[inline]
    fn lock(&self) {
        assert!(
            self.try_lock(),
            "called `lock` on a `RawOneShotMutex` that is already locked"
        );
    }

    #[inline]
    fn try_lock(&self) -> bool {
        let was_locked = self.lock.replace(true);
        !was_locked
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.lock.set(false);
    }

    #[inline]
    fn is_locked(&self) -> bool {
        self.lock.get()
    }
}

unsafe impl RawMutexFair for RawOneShotMutex {
    #[inline]
    unsafe fn unlock_fair(&self) {
        unsafe { self.unlock() }
    }

    #[inline]
    unsafe fn bump(&self) {}
}

/// A [`lock_api::Mutex`] based on [`RawOneShotMutex`].
pub type OneShotMutex<T> = lock_api::Mutex<RawOneShotMutex, T>;

/// A [`lock_api::MutexGuard`] based on [`RawOneShotMutex`].
pub type OneShotMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawOneShotMutex, T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock() {
        let mutex = OneShotMutex::new(42);
        let mut guard = mutex.lock();
        assert_eq!(*guard, 42);

        *guard += 1;
        drop(guard);
        let guard = mutex.lock();
        assert_eq!(*guard, 43);
    }

    #[test]
    #[should_panic]
    fn lock_panic() {
        let mutex = OneShotMutex::new(42);
        let _guard = mutex.lock();
        let _guard2 = mutex.lock();
    }

    #[test]
    fn try_lock() {
        let mutex = OneShotMutex::new(42);
        let mut guard = mutex.try_lock().unwrap();
        assert_eq!(*guard, 42);
        assert!(mutex.try_lock().is_none());

        *guard += 1;
        drop(guard);
        let guard = mutex.try_lock().unwrap();
        assert_eq!(*guard, 43);
    }
}
