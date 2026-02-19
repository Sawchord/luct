use luct_store::StringStoreValue;
use std::ops::Deref;
use web_time::{Duration, SystemTime};

/// Wrapper around a type to indicate, that the contained value has been validated
///
/// When wrapping a `T` into [`Validated`], it means that the value has been validated and will be
/// trusted from now on.
#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct Validated<T> {
    inner: T,
    validated_at: SystemTime,
}

impl<T: PartialEq> PartialEq for Validated<T> {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: The validated_at should not influence equality
        self.inner == other.inner
    }
}

impl<T> Validated<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            validated_at: SystemTime::now(),
        }
    }

    pub(crate) fn validated_at(&self) -> SystemTime {
        self.validated_at
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: StringStoreValue> StringStoreValue for Validated<T> {
    fn serialize_value(&self) -> String {
        let inner = self.inner.serialize_value();
        let validated_at: u64 = self
            .validated_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        serde_json::to_string(&(validated_at, inner)).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        let (validated_at, inner): (u64, String) = serde_json::from_str(value).ok()?;

        let validated_at =
            SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(validated_at))?;
        let inner = T::deserialize_value(&inner)?;

        Some(Self {
            inner,
            validated_at,
        })
    }
}

// TODO: Round trip test
