pub trait WideStringExt {
    fn to_string(&self) -> String;
}

impl<const N: usize> WideStringExt for [u16; N] {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}

impl WideStringExt for [u16] {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}

impl WideStringExt for Vec<u16> {
    fn to_string(&self) -> String {
        let pos = self.iter().position(|&c| c == 0).unwrap_or(self.len());
        String::from_utf16_lossy(&self[0..pos])
    }
}
