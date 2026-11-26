//! Process-wide serialization for operations that read-modify-write `config.toml`.
//!
//! The commands that persist the configuration run on Tauri's async runtime, so two
//! invocations (a save and a stop, for example) can be in flight at the same time.
//! `ConfigWriter` already makes each individual write atomic, but a stop's
//! read-modify-write is two steps: without a lock a write can land between its read
//! and its write and be lost. Holding this lock for the whole operation makes the
//! last operation's complete input the file's content.
//!
//! The frontend also queues its own writes (`src/store/config-write-queue.ts`); this
//! is the backstop for any other caller of these commands.

use std::sync::{Mutex, MutexGuard};

static CONFIG_FILE_LOCK: Mutex<()> = Mutex::new(());

/// Claims the configuration-file lock, ignoring a poisoned mutex.
///
/// A panic while the lock is held must not turn every later save into a second
/// failure mode: the file is still written atomically, so continuing is the safer
/// answer.
pub(crate) fn lock() -> MutexGuard<'static, ()> {
    CONFIG_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
