use crate::definitions::traits::{FromJson, FromJsonError};
use serde_json::Value as Json;
use std::str::FromStr;

/// `issuing_subdivision` in the `org.iso.23220.1` namespace: an ISO 3166-2 subdivision code
/// (ISO/IEC TS 23220-2 Table 7).
///
/// An ISO 3166-2 code is the alpha-2 country code, a hyphen, and one to three letters or
/// digits — `US-IA`, `DE-BE`, `FR-75C`. Only that shape is checked.
///
/// Unlike `issuing_jurisdiction` in `org.iso.18013.5.1`, it is not checked against
/// `issuing_country`: 23220-2 allows an alpha-3 `issuing_country`, which an ISO 3166-2 code
/// never starts with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuingSubdivision(String);

impl IssuingSubdivision {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid ISO 3166-2 subdivision code {0:?}: expected e.g. \"US-IA\"")]
    Invalid(String),
}

impl From<IssuingSubdivision> for ciborium::Value {
    fn from(s: IssuingSubdivision) -> ciborium::Value {
        ciborium::Value::Text(s.0)
    }
}

impl FromStr for IssuingSubdivision {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let valid = s.split_once('-').is_some_and(|(country, subdivision)| {
            country.len() == 2
                && country.bytes().all(|b| b.is_ascii_uppercase())
                && (1..=3).contains(&subdivision.len())
                && subdivision
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        });
        if valid {
            Ok(Self(s.to_string()))
        } else {
            Err(Error::Invalid(s.to_string()))
        }
    }
}

impl FromJson for IssuingSubdivision {
    fn from_json(v: &Json) -> Result<Self, FromJsonError> {
        String::from_json(v)?
            .parse()
            .map_err(Into::into)
            .map_err(FromJsonError::Parsing)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid() {
        for code in ["US-IA", "DE-BE", "FR-75C", "GB-ENG", "CA-ON"] {
            assert!(code.parse::<IssuingSubdivision>().is_ok(), "{code}");
        }
    }

    #[test]
    fn invalid() {
        for code in ["US", "USA-IA", "us-ia", "US-", "US-ABCD", "US_IA", "US-I-A"] {
            assert!(code.parse::<IssuingSubdivision>().is_err(), "{code}");
        }
    }
}
