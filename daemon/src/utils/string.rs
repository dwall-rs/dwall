use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub trait WideStringRead {
    fn to_string(&self) -> String;
}

pub trait WideStringExt: WideStringRead {
    fn from_str(s: &str) -> Self;
}

impl<const N: usize> WideStringRead for [u16; N] {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}

impl<const N: usize> WideStringExt for [u16; N] {
    fn from_str(s: &str) -> Self {
        let mut buf = [0u16; N];
        let wide_chars: Vec<u16> = OsStr::new(s).encode_wide().collect();
        let copy_len = wide_chars.len().min(N - 1); // Ensure one position is reserved for the null terminator

        for i in 0..copy_len {
            buf[i] = wide_chars[i];
        }

        buf
    }
}

impl WideStringRead for [u16] {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}

impl WideStringRead for Vec<u16> {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}

impl WideStringExt for Vec<u16> {
    fn from_str(s: &str) -> Self {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}
