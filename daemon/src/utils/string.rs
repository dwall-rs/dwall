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

        buf[..copy_len].copy_from_slice(&wide_chars[..copy_len]);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_to_string() {
        // Test regular string
        let wide_array: [u16; 10] = [0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0, 0, 0, 0, 0]; // "Hello"
        assert_eq!(wide_array.to_string(), "Hello");

        // Test empty string
        let empty_array: [u16; 5] = [0, 0, 0, 0, 0];
        assert_eq!(empty_array.to_string(), "");

        // Test Chinese characters
        let special_array: [u16; 5] = [0x4F60, 0x597D, 0x4E16, 0x754C, 0]; // "Hello World" in Chinese
        assert_eq!(special_array.to_string(), "你好世界");
    }

    #[test]
    fn test_array_from_str() {
        // Test regular string
        let hello = "Hello";
        let wide_array = <[u16; 10]>::from_str(hello);
        assert_eq!(wide_array.to_string(), hello);

        // Test empty string
        let empty = "";
        let empty_array = <[u16; 5]>::from_str(empty);
        assert_eq!(empty_array.to_string(), empty);

        // Test special characters
        let special = "你好世界";
        let special_array = <[u16; 10]>::from_str(special);
        assert_eq!(special_array.to_string(), special);

        // Test truncation (array length insufficient)
        let long = "This string is too long for the array";
        let truncated_array = <[u16; 10]>::from_str(long);
        assert_eq!(truncated_array.to_string(), "This stri"); // Only 9 characters, as one position is reserved for the null terminator
    }

    #[test]
    fn test_slice_to_string() {
        // Test regular string
        let wide_slice: &[u16] = &[0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0]; // "Hello"
        assert_eq!(wide_slice.to_string(), "Hello");

        // Test slice without null terminator
        let no_null_slice: &[u16] = &[0x0048, 0x0065, 0x006C, 0x006C, 0x006F]; // "Hello" without null terminator
        assert_eq!(no_null_slice.to_string(), "Hello");

        // Test empty slice
        let empty_slice: &[u16] = &[0];
        assert_eq!(empty_slice.to_string(), "");
    }

    #[test]
    fn test_vec_to_string() {
        // Test regular string
        let wide_vec: Vec<u16> = vec![0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0]; // "Hello"
        assert_eq!(wide_vec.to_string(), "Hello");

        // Test empty vector
        let empty_vec: Vec<u16> = vec![0];
        assert_eq!(empty_vec.to_string(), "");

        // Test Chinese characters
        let special_vec: Vec<u16> = vec![0x4F60, 0x597D, 0x4E16, 0x754C, 0]; // "Hello World" in Chinese
        assert_eq!(special_vec.to_string(), "你好世界");
    }

    #[test]
    fn test_vec_from_str() {
        // Test regular string
        let hello = "Hello";
        let wide_vec = Vec::<u16>::from_str(hello);
        assert_eq!(wide_vec.to_string(), hello);

        // Test empty string
        let empty = "";
        let empty_vec = Vec::<u16>::from_str(empty);
        assert_eq!(empty_vec.to_string(), empty);

        // Test special characters
        let special = "你好世界";
        let special_vec = Vec::<u16>::from_str(special);
        assert_eq!(special_vec.to_string(), special);

        // Test long string (vectors automatically resize)
        let long = "This is a long string that a fixed-size array might not be able to handle";
        let long_vec = Vec::<u16>::from_str(long);
        assert_eq!(long_vec.to_string(), long);
    }
}
