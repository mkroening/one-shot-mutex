//! One-shot locks that panic instead of (dead)locking on contention.
//!
//! These locks allow no contention and panic instead of blocking on `lock` if they are already locked.
//! This is useful in situations where contention would be a bug,
//! such as in single-threaded programs that would deadlock on contention.
//!
//! See the [`sync::RawOneShotMutex`] and [`sync::RawOneShotRwLock`] types for more information.

#![no_std]

pub mod sync;
pub mod unsync;
