use serde_json::Value as Json;
use std::{fmt, ops::Deref, str::FromStr};

use crate::definitions::traits::{FromJson, FromJsonError};

/// A CBOR text string (`tstr`) of at most 150 Unicode characters.
///
/// ISO/IEC TS 23220-2 puts no length limit on `tstr` data elements unless a profile says
/// otherwise, and the photo ID profile does: ISO/IEC TS 23220-4 Annex C, C.2.1, requires
/// every `tstr` data element in Tables C.1, C.2 and C.3 to be at most 150 characters.
///
/// Unlike [`Latin1`](super::latin1::Latin1), any Unicode character is allowed: 23220-2
/// defines the unsuffixed identifiers (`family_name`, `given_name`, ...) for national
/// characters, and the `_latin1` variants for when Latin 1 is mandated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tstr150(String);

impl Tstr150 {
    /// The maximum length, in characters.
    pub const MAX_CHARS: usize = 150;

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("contains more than 150 characters: {0}")]
    TooLong(usize),
}

impl Deref for Tstr150 {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tstr150 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Tstr150> for ciborium::Value {
    fn from(t: Tstr150) -> ciborium::Value {
        t.0.into()
    }
}

impl FromStr for Tstr150 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let chars = s.chars().count();
        if chars > Self::MAX_CHARS {
            return Err(Error::TooLong(chars));
        }
        Ok(Self(s.to_string()))
    }
}

impl FromJson for Tstr150 {
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
    fn counts_characters_not_bytes() {
        // 150 two-byte characters: 300 bytes, but within the limit.
        let s = "é".repeat(150);
        assert!(s.parse::<Tstr150>().is_ok());
        assert!("é".repeat(151).parse::<Tstr150>().is_err());
    }

    #[test]
    fn accepts_non_latin1() {
        assert_eq!("山田".parse::<Tstr150>().unwrap().as_str(), "山田");
    }
}
