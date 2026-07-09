use crate::definitions::traits::{FromJson, FromJsonError};
use serde_json::Value as Json;
use std::str::FromStr;

/// ISO 3166-1 country code: alpha-2 (2 uppercase letters) or alpha-3 (3 uppercase letters).
///
/// Used in `org.iso.23220.1` for `issuing_country` and `nationality`, which allow either
/// alpha-2 or alpha-3 codes per ISO/IEC TS 23220-2.
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
        if (len == 2 || len == 3) && s.chars().all(|c| c.is_ascii_uppercase()) {
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
    fn invalid() {
        assert!("us".parse::<CountryCode>().is_err());   // lowercase
        assert!("USAA".parse::<CountryCode>().is_err()); // 4 letters
        assert!("U".parse::<CountryCode>().is_err());    // 1 letter
        assert!("U1".parse::<CountryCode>().is_err());   // digit
    }
}
