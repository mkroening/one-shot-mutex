//! One-shot locks that panic instead of (dead)locking on contention.
//!
//! These locks allow no contention and panic on `lock` if they are already locked.
//! This is useful in situations where contention would be a bug,
//! such as in single-threaded programs that would deadlock on contention.

#![no_std]

mod mutex;

pub use mutex::{OneShotMutex, OneShotMutexGuard, RawOneShotMutex};
