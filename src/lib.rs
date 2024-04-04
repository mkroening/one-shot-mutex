//! A one-shot mutex that panics instead of (dead)locking on contention.
//!
//! This mutex allows no contention and panics on [`lock`] if it is already locked.
//! This is useful in situations where contention would be a bug,
//! such as in single-threaded programs that would deadlock on contention.
//!
//! [`lock`]: RawOneShotMutex::lock
//!
//! # Examples
//!
//! ```
//! use one_shot_mutex::OneShotMutex;
//!
//! static X: OneShotMutex<i32> = OneShotMutex::new(42);
//!
//! let x = X.lock();
//!
//! // This panics instead of deadlocking.
//! // let x2 = X.lock();
//!
//! // Once we unlock the mutex, we can lock it again.
//! drop(x);
//! let x = X.lock();
//! ```

#![no_std]

mod mutex;

pub use mutex::{OneShotMutex, OneShotMutexGuard, RawOneShotMutex};
