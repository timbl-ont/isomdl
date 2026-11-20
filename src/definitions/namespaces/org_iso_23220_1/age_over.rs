use crate::definitions::{
    namespaces::age_over::{Age, AgeOver},
    traits::{FromJsonError, FromJsonMap, ToNamespaceMap},
};
use serde_json::{Map, Value as Json};
use std::{collections::BTreeMap, ops::Deref};

/// The `age_over_NN` attestations of a photo ID, of which `age_over_18` is mandatory.
///
/// ISO/IEC TS 23220-4 Annex C Table C.1 marks `age_over_18` mandatory and the other
/// `age_over_NN` recommended. Every `age_over_NN` sits in one map, so the requirement is
/// checked here rather than by the type of a field.
#[derive(Debug, Clone)]
pub struct PhotoIdAgeOver(AgeOver);

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("age_over_18 is mandatory in a photo ID (ISO/IEC TS 23220-4 Table C.1)")]
    MissingAgeOver18,
}

impl PhotoIdAgeOver {
    pub fn new(age_over: AgeOver) -> Result<Self, Error> {
        if age_over.contains_key(&eighteen()) {
            Ok(Self(age_over))
        } else {
            Err(Error::MissingAgeOver18)
        }
    }

    /// The value of the mandatory `age_over_18`.
    pub fn age_over_18(&self) -> bool {
        // Unwrap safety: `new` refuses a map without it.
        *self.0.get(&eighteen()).unwrap()
    }

    pub fn into_inner(self) -> AgeOver {
        self.0
    }
}

fn eighteen() -> Age {
    // Unwrap safety: 18 is a valid two-digit age.
    Age::try_from(18).unwrap()
}

impl Deref for PhotoIdAgeOver {
    type Target = AgeOver;

    fn deref(&self) -> &AgeOver {
        &self.0
    }
}

impl FromJsonMap for PhotoIdAgeOver {
    fn from_map(m: &Map<String, Json>) -> Result<Self, FromJsonError> {
        Self::new(AgeOver::from_map(m)?).map_err(|e| FromJsonError::Parsing(e.into()))
    }
}

impl ToNamespaceMap for PhotoIdAgeOver {
    fn to_ns_map(self) -> BTreeMap<String, ciborium::Value> {
        self.0.to_ns_map()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn requires_age_over_18() {
        let without = serde_json::json!({ "age_over_21": true });
        assert!(PhotoIdAgeOver::from_map(without.as_object().unwrap()).is_err());

        let with = serde_json::json!({ "age_over_18": false, "age_over_21": false });
        let ages = PhotoIdAgeOver::from_map(with.as_object().unwrap()).unwrap();
        assert!(!ages.age_over_18());
        assert_eq!(ages.len(), 2);
    }
}
