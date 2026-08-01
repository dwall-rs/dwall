//! Safe wrapper for Windows HANDLE

use std::fmt::Debug;
use windows::Win32::Foundation::HANDLE;
use windows::core::Free;

#[derive(Debug)]
pub struct HandleWrapper(HANDLE);

impl HandleWrapper {
    pub fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    pub fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { self.0.free() };
        }
    }
}
