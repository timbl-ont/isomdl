use crate::definitions::traits::{FromJson, FromJsonError};
use serde_json::Value as Json;
use std::str::FromStr;

/// An ISO 3166-1 alpha-2 or alpha-3 country code.
///
/// ISO/IEC TS 23220-2 allows either form for `issuing_country` (Table 7) and `nationality`
/// (Table 2), where ISO/IEC 18013-5 allows only alpha-2. `resident_country` stays alpha-2
/// only, so it uses [`Alpha2`](super::Alpha2).
///
/// Only the shape is checked — two or three upper-case ASCII letters — because `nationality`
/// may also carry the ICAO Doc 9303-3 codes for persons without a defined nationality
/// (`XXA`, `XXB`, `XXC`, `XXX`), which are not ISO 3166-1 assignments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountryCode(String);

impl CountryCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid country code {0:?}: must be 2 or 3 uppercase ASCII letters")]
    Invalid(String),
}

impl From<CountryCode> for ciborium::Value {
    fn from(c: CountryCode) -> ciborium::Value {
        ciborium::Value::Text(c.0)
    }
}

impl FromStr for CountryCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let len = s.len();
        if (len == 2 || len == 3) && s.bytes().all(|c| c.is_ascii_uppercase()) {
            Ok(CountryCode(s.to_string()))
        } else {
            Err(Error::Invalid(s.to_string()))
        }
    }
}

impl FromJson for CountryCode {
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
    fn alpha2() {
        assert!("US".parse::<CountryCode>().is_ok());
        assert!("NL".parse::<CountryCode>().is_ok());
    }

    #[test]
    fn alpha3() {
        assert!("USA".parse::<CountryCode>().is_ok());
        assert!("NLD".parse::<CountryCode>().is_ok());
    }

    #[test]
    fn icao_stateless() {
        assert!("XXA".parse::<CountryCode>().is_ok());
    }

    #[test]
    fn invalid() {
        assert!("us".parse::<CountryCode>().is_err()); // lowercase
        assert!("USAA".parse::<CountryCode>().is_err()); // 4 letters
        assert!("U".parse::<CountryCode>().is_err()); // 1 letter
        assert!("U1".parse::<CountryCode>().is_err()); // digit
        assert!("ÜS".parse::<CountryCode>().is_err()); // non-ASCII
    }
}
