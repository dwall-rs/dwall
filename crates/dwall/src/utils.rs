//! Shared utilities: wide-string conversions and a small typed value cache.

use std::any::TypeId;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use crate::color_scheme::{DaylightState, ThresholdConfig};
use crate::solar::{Position, SolarPosition};

// ── Wide string helpers ─────────────────────────────────────────────────────

/// Converts a null-terminated wide buffer into a `String`.
pub trait WideStringRead {
    fn to_string(&self) -> String;
}

/// Converts a Rust string into a wide buffer.
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
        let copy_len = wide_chars.len().min(N - 1); // reserve one slot for the null terminator

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

// ── Typed value cache ───────────────────────────────────────────────────────

macro_rules! define_cache {
    // Syntax: VariantName / field_name => Type
    //   $variant : UpperCamelCase, only for enum variants
    //   $field   : snake_case,     only for struct fields
    ( $( $variant:ident / $field:ident => $type:ty ),+ $(,)? ) => {

        #[derive(Debug, Clone)]
        pub(crate) enum CacheValue {
            $( $variant($type), )+
        }

        struct Inner {
            $( $field: Option<Entry>, )+
        }

        impl Inner {
            fn new() -> Self {
                Inner { $( $field: None, )+ }
            }

            fn slot_mut(&mut self, id: TypeId) -> &mut Option<Entry> {
                $(
                    if id == TypeId::of::<$type>() {
                        return &mut self.$field;
                    }
                )+
                unreachable!("Unregistered cache type");
            }

            fn purge_expired(&mut self) -> usize {
                let mut count = 0;
                $(
                    if self.$field.as_ref().is_some_and(|e| e.is_expired()) {
                        self.$field = None;
                        count += 1;
                    }
                )+
                count
            }
        }

        $(
            impl Cacheable for $type {
                fn into_value(self) -> CacheValue {
                    CacheValue::$variant(self)
                }
                fn from_value(v: &CacheValue) -> Option<&Self> {
                    match v {
                        CacheValue::$variant(inner) => Some(inner),
                        #[allow(unreachable_patterns)]
                        _ => None,
                    }
                }
            }
        )+
    };
}

define_cache! {
    //  enum variant     struct field      type
    //  UpperCamelCase  /  snake_case   => Type
    Position / position => Position,
    ThresholdConfig / threshold_config => ThresholdConfig,
    DaylightState / daylight_state => DaylightState,
    SolarPosition / solar_position => SolarPosition,
}

pub(crate) trait Cacheable: Clone + 'static {
    fn into_value(self) -> CacheValue;
    fn from_value(v: &CacheValue) -> Option<&Self>;
}

struct Entry {
    value: CacheValue,
    expires_at: Instant,
}

impl Entry {
    fn new(value: CacheValue, ttl: Duration) -> Self {
        Entry {
            value,
            expires_at: Instant::now() + ttl,
        }
    }
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

static CACHE: OnceLock<Cache> = OnceLock::new();

/// Returns the process-wide cache, initializing it on first use.
pub fn get_cache() -> &'static Cache {
    CACHE.get_or_init(|| Cache::new(Duration::from_hours(24 * 7)))
}

/// A small typed cache with per-entry TTL and background cleanup.
#[derive(Clone)]
pub struct Cache {
    inner: Arc<Mutex<Inner>>,
}

impl Cache {
    pub fn new(cleanup_interval: Duration) -> Self {
        let inner = Arc::new(Mutex::new(Inner::new()));
        let weak = Arc::clone(&inner);

        thread::spawn(move || {
            loop {
                thread::sleep(cleanup_interval);
                let removed = weak.lock().unwrap().purge_expired();
                if removed > 0 {
                    info!("[Cache] cleaned up {} expired entries", removed);
                }
            }
        });

        Cache { inner }
    }

    /// Store a value, specifying TTL
    pub(crate) fn set<T: Cacheable>(&self, value: T, ttl: Duration) {
        let cv = value.into_value();
        let entry = Entry::new(cv, ttl);
        *self.inner.lock().unwrap().slot_mut(TypeId::of::<T>()) = Some(entry);
    }

    /// Retrieve a value (returns None if missing or expired, with lazy deletion)
    pub(crate) fn get<T: Cacheable>(&self) -> Option<T> {
        let mut guard = self.inner.lock().unwrap();
        let slot = guard.slot_mut(TypeId::of::<T>());
        match slot {
            Some(e) if e.is_expired() => {
                *slot = None;
                None
            }
            Some(e) => T::from_value(&e.value).cloned(),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_to_string() {
        let wide_array: [u16; 10] = [0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0, 0, 0, 0, 0]; // "Hello"
        assert_eq!(wide_array.to_string(), "Hello");

        let empty_array: [u16; 5] = [0, 0, 0, 0, 0];
        assert_eq!(empty_array.to_string(), "");

        let special_array: [u16; 5] = [0x4F60, 0x597D, 0x4E16, 0x754C, 0]; // "你好世界"
        assert_eq!(special_array.to_string(), "你好世界");
    }

    #[test]
    fn test_array_from_str() {
        let hello = "Hello";
        let wide_array = <[u16; 10]>::from_str(hello);
        assert_eq!(wide_array.to_string(), hello);

        let empty = "";
        let empty_array = <[u16; 5]>::from_str(empty);
        assert_eq!(empty_array.to_string(), empty);

        let special = "你好世界";
        let special_array = <[u16; 10]>::from_str(special);
        assert_eq!(special_array.to_string(), special);

        let long = "This string is too long for the array";
        let truncated_array = <[u16; 10]>::from_str(long);
        assert_eq!(truncated_array.to_string(), "This stri");
    }

    #[test]
    fn test_slice_to_string() {
        let wide_slice: &[u16] = &[0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0]; // "Hello"
        assert_eq!(wide_slice.to_string(), "Hello");

        let no_null_slice: &[u16] = &[0x0048, 0x0065, 0x006C, 0x006C, 0x006F];
        assert_eq!(no_null_slice.to_string(), "Hello");

        let empty_slice: &[u16] = &[0];
        assert_eq!(empty_slice.to_string(), "");
    }

    #[test]
    fn test_vec_to_string() {
        let wide_vec: Vec<u16> = vec![0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0]; // "Hello"
        assert_eq!(wide_vec.to_string(), "Hello");

        let empty_vec: Vec<u16> = vec![0];
        assert_eq!(empty_vec.to_string(), "");

        let special_vec: Vec<u16> = vec![0x4F60, 0x597D, 0x4E16, 0x754C, 0]; // "你好世界"
        assert_eq!(special_vec.to_string(), "你好世界");
    }

    #[test]
    fn test_vec_from_str() {
        let hello = "Hello";
        let wide_vec = Vec::<u16>::from_str(hello);
        assert_eq!(wide_vec.to_string(), hello);

        let empty = "";
        let empty_vec = Vec::<u16>::from_str(empty);
        assert_eq!(empty_vec.to_string(), empty);

        let special = "你好世界";
        let special_vec = Vec::<u16>::from_str(special);
        assert_eq!(special_vec.to_string(), special);

        let long = "This is a long string that a fixed-size array might not be able to handle";
        let long_vec = Vec::<u16>::from_str(long);
        assert_eq!(long_vec.to_string(), long);
    }
}
