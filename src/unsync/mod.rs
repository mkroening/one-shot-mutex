//! One-shot lock variants that do not implement `Sync`.
//!
//! These one-shot locks not implement `Sync`, which permits slightly more efficient
//! implementations.

mod mutex;
mod rwlock;

pub use mutex::{OneShotMutex, OneShotMutexGuard, RawOneShotMutex};
pub use rwlock::{
    OneShotRwLock, OneShotRwLockReadGuard, OneShotRwLockUpgradableReadGuard,
    OneShotRwLockWriteGuard, RawOneShotRwLock,
};
