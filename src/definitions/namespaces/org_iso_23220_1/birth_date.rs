use crate::definitions::{
    namespaces::fulldate::FullDate,
    traits::{FromJson, FromJsonError},
};
use anyhow::anyhow;
use serde_json::Value as Json;
use std::{fmt, str::FromStr};

/// `birth_date` in the `org.iso.23220.1` namespace.
///
/// Unlike `org.iso.18013.5.1`, where it is a plain `full-date`, ISO/IEC TS 23220-2 6.3.1.3
/// defines `birth_date` as a map:
///
/// ```text
/// "birth_date" = {
///     "birth_date" : full-date,     ; YYYY-MM-DD
///     ? "approximate_mask" : tstr   ; 8-digit flag (YYYYMMDD), 1 = masked
/// }
/// ```
///
/// The issuing authority picks one exact date for `birth_date` even when parts of it are
/// unknown, and `age_over_NN` / `age_in_years` are calculated from that date. The mask only
/// says which digits are not real; it is not for calculation.
///
/// For convenience, [`FromJson`] also accepts a bare `"YYYY-MM-DD"` string as a date with no
/// mask. The CBOR encoding is always the map.
#[derive(Debug, Clone)]
pub struct BirthDate {
    pub birth_date: FullDate,
    pub approximate_mask: Option<ApproximateMask>,
}

/// The `approximate_mask` of a [`BirthDate`]: eight `0`/`1` digits lining up with
/// `YYYYMMDD`, where `1` marks a digit of `birth_date` that is unknown.
///
/// `"00000011"` says the day is unknown; `"00001111"` says the month and day are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproximateMask(String);

impl ApproximateMask {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("approximate_mask must be 8 digits, each 0 or 1, received {0:?}")]
    InvalidMask(String),
}

impl FromStr for ApproximateMask {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        if s.len() == 8 && s.bytes().all(|b| b == b'0' || b == b'1') {
            Ok(Self(s.to_string()))
        } else {
            Err(Error::InvalidMask(s.to_string()))
        }
    }
}

impl fmt::Display for ApproximateMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromJson for ApproximateMask {
    fn from_json(v: &Json) -> Result<Self, FromJsonError> {
        String::from_json(v)?
            .parse()
            .map_err(Into::into)
            .map_err(FromJsonError::Parsing)
    }
}

impl From<BirthDate> for ciborium::Value {
    fn from(b: BirthDate) -> ciborium::Value {
        // "birth_date" sorts before "approximate_mask" under both the length-first and the
        // bytewise key orderings, so this order is already deterministic.
        let mut map = vec![(
            ciborium::Value::Text("birth_date".to_string()),
            b.birth_date.into(),
        )];
        if let Some(mask) = b.approximate_mask {
            map.push((
                ciborium::Value::Text("approximate_mask".to_string()),
                ciborium::Value::Text(mask.0),
            ));
        }
        ciborium::Value::Map(map)
    }
}

impl FromJson for BirthDate {
    fn from_json(v: &Json) -> Result<Self, FromJsonError> {
        match v {
            Json::String(_) => Ok(BirthDate {
                birth_date: FullDate::from_json(v)?,
                approximate_mask: None,
            }),
            Json::Object(map) => {
                let birth_date = FullDate::from_json_opt(map.get("birth_date"))
                    .map_err(|e| FromJsonError::WithContext("birth_date", Box::new(e)))?;
                let approximate_mask =
                    Option::<ApproximateMask>::from_json_opt(map.get("approximate_mask"))
                        .map_err(|e| FromJsonError::WithContext("approximate_mask", Box::new(e)))?;
                Ok(BirthDate {
                    birth_date,
                    approximate_mask,
                })
            }
            _ => Err(anyhow!("birth_date must be a full-date string or an object").into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_plain_string() {
        let bd = BirthDate::from_json(&serde_json::json!("1980-01-15")).unwrap();
        assert!(bd.approximate_mask.is_none());
    }

    #[test]
    fn from_object_with_mask() {
        let bd = BirthDate::from_json(&serde_json::json!({
            "birth_date": "1980-01-01",
            "approximate_mask": "00000011"
        }))
        .unwrap();
        assert_eq!(bd.approximate_mask.unwrap().as_str(), "00000011");
    }

    #[test]
    fn from_object_without_mask() {
        let bd = BirthDate::from_json(&serde_json::json!({ "birth_date": "1995-06-20" })).unwrap();
        assert!(bd.approximate_mask.is_none());
    }

    #[test]
    fn rejects_a_malformed_mask() {
        for mask in ["0000001", "000000011", "00000012", "0000000a", ""] {
            let json = serde_json::json!({ "birth_date": "1980-01-01", "approximate_mask": mask });
            assert!(
                BirthDate::from_json(&json).is_err(),
                "{mask:?} was accepted"
            );
        }
    }

    #[test]
    fn rejects_an_object_without_a_date() {
        let json = serde_json::json!({ "approximate_mask": "00000011" });
        assert!(BirthDate::from_json(&json).is_err());
    }

    #[test]
    fn encodes_as_a_map_with_a_full_date() {
        let bd = BirthDate::from_json(&serde_json::json!({
            "birth_date": "1980-01-01",
            "approximate_mask": "00000011"
        }))
        .unwrap();
        assert_eq!(
            ciborium::Value::from(bd),
            ciborium::Value::Map(vec![
                (
                    ciborium::Value::Text("birth_date".into()),
                    ciborium::Value::Tag(
                        1004,
                        Box::new(ciborium::Value::Text("1980-01-01".into()))
                    )
                ),
                (
                    ciborium::Value::Text("approximate_mask".into()),
                    ciborium::Value::Text("00000011".into())
                ),
            ])
        );
    }
}
