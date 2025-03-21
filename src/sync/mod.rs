//! One-shot lock variants that implement `Sync`.

mod mutex;
mod rwlock;

pub use mutex::{OneShotMutex, OneShotMutexGuard, RawOneShotMutex};
pub use rwlock::{
    OneShotRwLock, OneShotRwLockReadGuard, OneShotRwLockUpgradableReadGuard,
    OneShotRwLockWriteGuard, RawOneShotRwLock,
};
