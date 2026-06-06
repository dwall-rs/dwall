use std::any::TypeId;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::Position;
use crate::domain::visual::{DaylightState, ThresholdConfig};

macro_rules! define_cache {
    // Syntax: VariantName / field_name => Type
    //   $variant : UpperCamelCase, only for enum variants
    //   $field   : snake_case,     only for struct fields
    ( $( $variant:ident / $field:ident => $type:ty ),+ $(,)? ) => {

        // 2a. Generate a unified enum, variant names use UpperCamelCase ($variant)
        #[derive(Debug, Clone)]
        pub enum CacheValue {
            $( $variant($type), )+
        }

        // 2b. Generate Inner, field names use snake_case ($field)
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
                    if self.$field.as_ref().map_or(false, |e| e.is_expired()) {
                        self.$field = None;
                        count += 1;
                    }
                )+
                count
            }
        }

        // 2c. Implement Cacheable for each business type
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
}

pub trait Cacheable: Clone + 'static {
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

pub fn get_cache() -> &'static Cache {
    CACHE.get_or_init(|| Cache::new(Duration::from_hours(24 * 7)))
}

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
    pub fn set<T: Cacheable>(&self, value: T, ttl: Duration) {
        let cv = value.into_value();
        let entry = Entry::new(cv, ttl);
        *self.inner.lock().unwrap().slot_mut(TypeId::of::<T>()) = Some(entry);
    }

    /// Retrieve a value (returns None if missing or expired, with lazy deletion)
    pub fn get<T: Cacheable>(&self) -> Option<T> {
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

    // /// Manually remove
    // pub fn remove<T: Cacheable>(&self) {
    //     *self.inner.lock().unwrap().slot_mut(TypeId::of::<T>()) = None;
    // }

    // /// Check if present and not expired
    // pub fn contains<T: Cacheable>(&self) -> bool {
    //     self.get::<T>().is_some()
    // }
}
