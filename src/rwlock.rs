use core::sync::atomic::{AtomicUsize, Ordering};

use lock_api::{
    GuardSend, RawRwLock, RawRwLockDowngrade, RawRwLockRecursive, RawRwLockUpgrade,
    RawRwLockUpgradeDowngrade,
};

/// A one-shot readers-writer lock that panics instead of (dead)locking on contention.
///
/// This lock allows no contention and panics on [`lock_shared`], [`lock_exclusive`], [`lock_upgradable`], and [`upgrade`] if it is already locked conflictingly.
/// This is useful in situations where contention would be a bug,
/// such as in single-threaded programs that would deadlock on contention.
///
/// [`lock_shared`]: RawOneShotRwLock::lock_shared
/// [`lock_exclusive`]: RawOneShotRwLock::lock_exclusive
/// [`lock_upgradable`]: RawOneShotRwLock::lock_upgradable
/// [`upgrade`]: RawOneShotRwLock::upgrade
///
/// # Examples
///
/// ```
/// use one_shot_mutex::OneShotRwLock;
///
/// static X: OneShotRwLock<i32> = OneShotRwLock::new(42);
///
/// // This is equivalent to `X.try_write().unwrap()`.
/// let x = X.write();
///
/// // This panics instead of deadlocking.
/// // let x2 = X.write();
///
/// // Once we unlock the mutex, we can lock it again.
/// drop(x);
/// let x = X.write();
/// ```
pub struct RawOneShotRwLock {
    lock: AtomicUsize,
}

/// Normal shared lock counter
const SHARED: usize = 1 << 2;
/// Special upgradable shared lock flag
const UPGRADABLE: usize = 1 << 1;
/// Exclusive lock flag
const EXCLUSIVE: usize = 1;

impl RawOneShotRwLock {
    #[inline]
    fn is_locked_shared(&self) -> bool {
        self.lock.load(Ordering::Relaxed) & !(EXCLUSIVE | UPGRADABLE) != 0
    }

    #[inline]
    fn is_locked_upgradable(&self) -> bool {
        self.lock.load(Ordering::Relaxed) & UPGRADABLE == UPGRADABLE
    }

    /// Acquire a shared lock, returning the new lock value.
    #[inline]
    fn acquire_shared(&self) -> usize {
        let value = self.lock.fetch_add(SHARED, Ordering::Acquire);

        // An arbitrary cap that allows us to catch overflows long before they happen
        if value > usize::MAX / 2 {
            self.lock.fetch_sub(SHARED, Ordering::Relaxed);
            panic!("Too many shared locks, cannot safely proceed");
        }

        value
    }
}

unsafe impl RawRwLock for RawOneShotRwLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        lock: AtomicUsize::new(0),
    };

    type GuardMarker = GuardSend;

    #[inline]
    fn lock_shared(&self) {
        assert!(
            self.try_lock_shared(),
            "called `lock_shared` on a `RawOneShotRwLock` that is already locked exclusively"
        );
    }

    #[inline]
    fn try_lock_shared(&self) -> bool {
        let value = self.acquire_shared();

        let acquired = value & EXCLUSIVE != EXCLUSIVE;

        if !acquired {
            unsafe {
                self.unlock_shared();
            }
        }

        acquired
    }

    #[inline]
    unsafe fn unlock_shared(&self) {
        debug_assert!(self.is_locked_shared());

        self.lock.fetch_sub(SHARED, Ordering::Release);
    }

    #[inline]
    fn lock_exclusive(&self) {
        assert!(
            self.try_lock_exclusive(),
            "called `lock_exclusive` on a `RawOneShotRwLock` that is already locked"
        );
    }

    #[inline]
    fn try_lock_exclusive(&self) -> bool {
        self.lock
            .compare_exchange(0, EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    unsafe fn unlock_exclusive(&self) {
        debug_assert!(self.is_locked_exclusive());

        self.lock.fetch_and(!EXCLUSIVE, Ordering::Release);
    }

    #[inline]
    fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed) != 0
    }

    #[inline]
    fn is_locked_exclusive(&self) -> bool {
        self.lock.load(Ordering::Relaxed) & EXCLUSIVE == EXCLUSIVE
    }
}

unsafe impl RawRwLockRecursive for RawOneShotRwLock {
    #[inline]
    fn lock_shared_recursive(&self) {
        self.lock_shared();
    }

    #[inline]
    fn try_lock_shared_recursive(&self) -> bool {
        self.try_lock_shared()
    }
}

unsafe impl RawRwLockDowngrade for RawOneShotRwLock {
    #[inline]
    unsafe fn downgrade(&self) {
        // Reserve the shared guard for ourselves
        self.acquire_shared();

        unsafe {
            self.unlock_exclusive();
        }
    }
}

unsafe impl RawRwLockUpgrade for RawOneShotRwLock {
    #[inline]
    fn lock_upgradable(&self) {
        assert!(
            self.try_lock_upgradable(),
            "called `lock_upgradable` on a `RawOneShotRwLock` that is already locked upgradably or exclusively"
        );
    }

    #[inline]
    fn try_lock_upgradable(&self) -> bool {
        let value = self.lock.fetch_or(UPGRADABLE, Ordering::Acquire);

        let acquired = value & (UPGRADABLE | EXCLUSIVE) == 0;

        if !acquired && value & UPGRADABLE == 0 {
            unsafe {
                self.unlock_upgradable();
            }
        }

        acquired
    }

    #[inline]
    unsafe fn unlock_upgradable(&self) {
        debug_assert!(self.is_locked_upgradable());

        self.lock.fetch_and(!UPGRADABLE, Ordering::Release);
    }

    #[inline]
    unsafe fn upgrade(&self) {
        assert!(
            self.try_upgrade(),
            "called `upgrade` on a `RawOneShotRwLock` that is also locked shared by others"
        );
    }

    #[inline]
    unsafe fn try_upgrade(&self) -> bool {
        self.lock
            .compare_exchange(UPGRADABLE, EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
}

unsafe impl RawRwLockUpgradeDowngrade for RawOneShotRwLock {
    #[inline]
    unsafe fn downgrade_upgradable(&self) {
        self.acquire_shared();

        unsafe {
            self.unlock_upgradable();
        }
    }

    #[inline]
    unsafe fn downgrade_to_upgradable(&self) {
        debug_assert!(self.is_locked_exclusive());

        self.lock
            .fetch_xor(UPGRADABLE | EXCLUSIVE, Ordering::Release);
    }
}

/// A [`lock_api::RwLock`] based on [`RawOneShotRwLock`].
pub type OneShotRwLock<T> = lock_api::RwLock<RawOneShotRwLock, T>;

/// A [`lock_api::RwLockReadGuard`] based on [`RawOneShotRwLock`].
pub type OneShotRwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawOneShotRwLock, T>;

/// A [`lock_api::RwLockUpgradableReadGuard`] based on [`RawOneShotRwLock`].
pub type OneShotRwLockUpgradableReadGuard<'a, T> =
    lock_api::RwLockUpgradableReadGuard<'a, RawOneShotRwLock, T>;

/// A [`lock_api::RwLockWriteGuard`] based on [`RawOneShotRwLock`].
pub type OneShotRwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawOneShotRwLock, T>;

#[cfg(test)]
mod tests {
    use lock_api::RwLockUpgradableReadGuard;

    use super::*;

    #[test]
    fn lock_exclusive() {
        let lock = OneShotRwLock::new(42);
        let mut guard = lock.write();
        assert_eq!(*guard, 42);

        *guard += 1;
        drop(guard);
        let guard = lock.write();
        assert_eq!(*guard, 43);
    }

    #[test]
    #[should_panic]
    fn lock_exclusive_panic() {
        let lock = OneShotRwLock::new(42);
        let _guard = lock.write();
        let _guard2 = lock.write();
    }

    #[test]
    #[should_panic]
    fn lock_exclusive_shared_panic() {
        let lock = OneShotRwLock::new(42);
        let _guard = lock.write();
        let _guard2 = lock.read();
    }

    #[test]
    fn try_lock_exclusive() {
        let lock = OneShotRwLock::new(42);
        let mut guard = lock.try_write().unwrap();
        assert_eq!(*guard, 42);
        assert!(lock.try_write().is_none());

        *guard += 1;
        drop(guard);
        let guard = lock.try_write().unwrap();
        assert_eq!(*guard, 43);
    }

    #[test]
    fn lock_shared() {
        let lock = OneShotRwLock::new(42);
        let guard = lock.read();
        assert_eq!(*guard, 42);
        let guard2 = lock.read();
        assert_eq!(*guard2, 42);
    }

    #[test]
    #[should_panic]
    fn lock_shared_panic() {
        let lock = OneShotRwLock::new(42);
        let _guard = lock.write();
        let _guard2 = lock.read();
    }

    #[test]
    fn try_lock_shared() {
        let lock = OneShotRwLock::new(42);
        let guard = lock.try_read().unwrap();
        assert_eq!(*guard, 42);
        assert!(lock.try_write().is_none());

        let guard2 = lock.try_read().unwrap();
        assert_eq!(*guard2, 42);
    }

    #[test]
    fn lock_upgradable() {
        let lock = OneShotRwLock::new(42);
        let guard = lock.upgradable_read();
        assert_eq!(*guard, 42);
        assert!(lock.try_write().is_none());

        let mut upgraded = RwLockUpgradableReadGuard::upgrade(guard);
        *upgraded += 1;
        drop(upgraded);
        let guard2 = lock.upgradable_read();
        assert_eq!(*guard2, 43);
    }

    #[test]
    #[should_panic]
    fn lock_upgradable_panic() {
        let lock = OneShotRwLock::new(42);
        let _guard = lock.upgradable_read();
        let _guard2 = lock.upgradable_read();
    }

    #[test]
    #[should_panic]
    fn lock_upgradable_write_panic() {
        let lock = OneShotRwLock::new(42);
        let _guard = lock.write();
        let _guard2 = lock.upgradable_read();
    }

    #[test]
    fn try_lock_upgradable() {
        let lock = OneShotRwLock::new(42);
        let guard = lock.try_upgradable_read().unwrap();
        assert_eq!(*guard, 42);
        assert!(lock.try_write().is_none());

        let mut upgraded = RwLockUpgradableReadGuard::try_upgrade(guard).unwrap();
        *upgraded += 1;
        drop(upgraded);
        let guard2 = lock.try_upgradable_read().unwrap();
        assert_eq!(*guard2, 43);
    }

    #[test]
    #[should_panic]
    fn upgrade_panic() {
        let lock = OneShotRwLock::new(42);
        let guard = lock.upgradable_read();
        let _guard2 = lock.read();
        let _guard3 = RwLockUpgradableReadGuard::upgrade(guard);
    }
}
