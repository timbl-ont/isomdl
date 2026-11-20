use crate::definitions::{
    namespaces::fulldate::FullDate,
    traits::{FromJson, FromJsonError},
};
use anyhow::anyhow;
use serde_json::Value as Json;

/// `birth_date` in the `org.iso.23220.1` namespace.
///
/// Encoded as a CBOR map per ISO/IEC TS 23220-2 §6.3.1.3:
///
/// ```text
/// "birth_date" = {
///     "birth_date" : full-date,     ; YYYY-MM-DD
///     ? "approximate_mask" : tstr   ; 8-digit flag (YYYYMMDD), 1 = masked
/// }
/// ```
///
/// `approximate_mask` is an 8-digit string indicating which date components are
/// unknown (e.g. `"00000100"` masks the day). It must not be used for age
/// calculation; `age_over_NN` and `age_in_years` are derived from the exact
/// `birth_date` value chosen by the issuing authority.
#[derive(Debug, Clone)]
pub struct BirthDate {
    pub birth_date: FullDate,
    pub approximate_mask: Option<String>,
}

impl From<BirthDate> for ciborium::Value {
    fn from(b: BirthDate) -> ciborium::Value {
        let mut map = vec![(
            ciborium::Value::Text("birth_date".to_string()),
            b.birth_date.into(),
        )];
        if let Some(mask) = b.approximate_mask {
            map.push((
                ciborium::Value::Text("approximate_mask".to_string()),
                ciborium::Value::Text(mask),
            ));
        }
        ciborium::Value::Map(map)
    }
}

impl FromJson for BirthDate {
    fn from_json(v: &Json) -> Result<Self, FromJsonError> {
        match v {
            // Accept a plain date string for convenience (no approximate mask).
            Json::String(_) => Ok(BirthDate {
                birth_date: FullDate::from_json(v)?,
                approximate_mask: None,
            }),
            Json::Object(map) => {
                let birth_date = map
                    .get("birth_date")
                    .ok_or(FromJsonError::Missing)
                    .and_then(FullDate::from_json)?;
                let approximate_mask = map
                    .get("approximate_mask")
                    .map(String::from_json)
                    .transpose()?;
                Ok(BirthDate {
                    birth_date,
                    approximate_mask,
                })
            }
            _ => Err(anyhow!("birth_date must be a date string or an object").into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::definitions::traits::FromJson;

    #[test]
    fn from_plain_string() {
        let v = serde_json::json!("1980-01-15");
        let bd = BirthDate::from_json(&v).unwrap();
        assert!(bd.approximate_mask.is_none());
    }

    #[test]
    fn from_object_with_mask() {
        let v = serde_json::json!({
            "birth_date": "1980-01-01",
            "approximate_mask": "00000100"
        });
        let bd = BirthDate::from_json(&v).unwrap();
        assert_eq!(bd.approximate_mask.as_deref(), Some("00000100"));
    }

    #[test]
    fn from_object_no_mask() {
        let v = serde_json::json!({ "birth_date": "1995-06-20" });
        let bd = BirthDate::from_json(&v).unwrap();
        assert!(bd.approximate_mask.is_none());
    }
}
