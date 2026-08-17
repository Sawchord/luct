use luct_store::StringStoreValue;
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Visitor},
};
use std::{marker::PhantomData, ops::Deref};
use web_time::{Duration, SystemTime};

/// Wrapper around a type to indicate, that the contained value has been validated
///
/// When wrapping a `T` into [`Validated`], it means that the value has been validated and will be
/// trusted from now on.
#[derive(Debug, Clone, Eq, PartialOrd, Ord, Serialize)]
pub struct Validated<T> {
    inner: T,
    validated_at: SystemTime,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Validated<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValidatedVisitor<T>(PhantomData<T>);
        const FIELDS: [&str; 2] = ["inner", "validated_at"];

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ValidatedVisitor<T> {
            type Value = Validated<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Validated")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut inner = None;
                let mut validated_at = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "inner" => {
                            if inner.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner = Some(map.next_value()?);
                        }
                        "validated_at" => {
                            if validated_at.is_some() {
                                return Err(serde::de::Error::duplicate_field("validated_at"));
                            }
                            validated_at = Some(map.next_value()?)
                        }
                        value => return Err(serde::de::Error::unknown_field(value, &FIELDS)),
                    }
                }

                let inner = inner.ok_or_else(|| serde::de::Error::missing_field("inner"))?;
                let validated_at =
                    validated_at.ok_or_else(|| serde::de::Error::missing_field("validated_at"))?;

                Ok(Validated {
                    inner,
                    validated_at,
                })
            }
        }

        deserializer.deserialize_struct("Validated", &FIELDS, ValidatedVisitor(PhantomData))
    }
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

impl<T: StringStoreValue + Serialize + DeserializeOwned> StringStoreValue for Validated<T> {
    fn serialize_value(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        serde_json::from_str(value)
            .ok()
            .or_else(|| Self::deserialize_value_legacy(value))
    }
}

impl<T: StringStoreValue> Validated<T> {
    // NOTE: Version 0.1 parses STHs this way. We can drop this, once we have implemented log sth removal
    fn deserialize_value_legacy(value: &str) -> Option<Self> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validated_json_roundtrip() {
        let test_data = Validated::new(5);
        let json = serde_json::to_string(&test_data).unwrap();
        dbg!(&json);
        let new_test_data = serde_json::from_str(&json).unwrap();
        assert_eq!(test_data, new_test_data)
    }

    #[test]
    fn legacy_validated_json() {
        let now = SystemTime::now();
        let now_str = serde_json::to_string(&now).unwrap();
        dbg!(&now_str);

        let legacy_validated = format!("[{}, 5]", now_str);
        let _new_test_data: Validated<usize> = serde_json::from_str(&legacy_validated).unwrap();
    }
}
