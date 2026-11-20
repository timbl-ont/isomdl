//! The photo ID of ISO/IEC TS 23220-4 Annex C: doc type `org.iso.23220.photoid.1`.
//!
//! A photo ID draws on two namespaces:
//!
//! | Namespace                 | Annex C   | Type                    |
//! |---------------------------|-----------|-------------------------|
//! | `org.iso.23220.1`         | Table C.1 | [`OrgIso232201`]        |
//! | `org.iso.23220.photoid.1` | Table C.2 | [`OrgIso23220Photoid1`] |
//!
//! [`PhotoId`] bundles them and turns the pair into the namespaces [`Mdoc::builder`] takes.
//! The eMRTD data groups of Table C.3 (`org.iso.23220.datagroups.1`) are not modelled.
//!
//! The types follow ISO/IEC TS 23220-2:2026 for encoding and ISO/IEC TS 23220-4:2026 Annex C
//! for presence.
//!
//! The certificate side of the profile is [`MdocProfile::ISO_23220`].
//!
//! [`Mdoc::builder`]: crate::issuance::Mdoc::builder
//! [`MdocProfile::ISO_23220`]: crate::definitions::x509::validation::MdocProfile::ISO_23220

use std::collections::BTreeMap;

use serde_json::Value as Json;

use super::{
    org_iso_23220_1::{self, OrgIso232201},
    org_iso_23220_photoid_1::{self, OrgIso23220Photoid1},
};
use crate::definitions::traits::{FromJson, FromJsonError, ToNamespaceMap};

pub use crate::definitions::PHOTO_ID_DOC_TYPE as DOC_TYPE;

/// The data of a photo ID, across the `org.iso.23220.1` and `org.iso.23220.photoid.1`
/// namespaces of ISO/IEC TS 23220-4 Annex C.
///
/// Annex C lets an issuer add namespaces of its own (national ones such as
/// `org.iso.23220.photoid.US-IA.1`, or reverse-domain ones). Insert those into the map
/// [`into_namespaces`](Self::into_namespaces) returns.
///
/// [`FromJson`] reads an object keyed by namespace identifier:
///
/// ```
/// use isomdl::definitions::{namespaces::photo_id::PhotoId, traits::FromJson};
///
/// let json = serde_json::json!({
///     "org.iso.23220.1": {
///         "family_name": "Smith",
///         "given_name": "Alice",
///         "birth_date": { "birth_date": "1990-04-01", "approximate_mask": "00000011" },
///         "portrait": "/9j/4AAQ",
///         "issue_date": "2025-01-01",
///         "expiry_date": "2035-01-01",
///         "issuing_authority": "Utopia Department of Identity",
///         "issuing_country": "UT",
///         "age_over_18": true
///     },
///     "org.iso.23220.photoid.1": {
///         "person_id": "1234567890"
///     }
/// });
///
/// let namespaces = PhotoId::from_json(&json).unwrap().into_namespaces();
/// assert!(namespaces.contains_key("org.iso.23220.1"));
/// assert!(namespaces.contains_key("org.iso.23220.photoid.1"));
/// ```
#[derive(Debug, Clone)]
pub struct PhotoId {
    /// The `org.iso.23220.1` data elements.
    pub iso_23220: OrgIso232201,
    /// The `org.iso.23220.photoid.1` data elements, if any.
    pub photo_id: Option<OrgIso23220Photoid1>,
}

impl PhotoId {
    /// The issuance namespaces, keyed by namespace identifier, as
    /// [`Builder::namespaces`](crate::issuance::mdoc::Builder::namespaces) takes them.
    ///
    /// `org.iso.23220.photoid.1` is left out when it has no data elements, since an mdoc
    /// cannot carry an empty namespace.
    pub fn into_namespaces(self) -> BTreeMap<String, BTreeMap<String, ciborium::Value>> {
        let mut namespaces = BTreeMap::new();
        namespaces.insert(
            org_iso_23220_1::NAMESPACE.to_string(),
            self.iso_23220.to_ns_map(),
        );
        if let Some(photo_id) = self.photo_id {
            let elements = photo_id.to_ns_map();
            if !elements.is_empty() {
                namespaces.insert(org_iso_23220_photoid_1::NAMESPACE.to_string(), elements);
            }
        }
        namespaces
    }
}

impl FromJson for PhotoId {
    fn from_json(v: &Json) -> Result<Self, FromJsonError> {
        let map = match v {
            Json::Object(map) => map,
            Json::Null => return Err(FromJsonError::UnexpectedType("null", "object")),
            Json::Bool(_) => return Err(FromJsonError::UnexpectedType("boolean", "object")),
            Json::Number(_) => return Err(FromJsonError::UnexpectedType("number", "object")),
            Json::String(_) => return Err(FromJsonError::UnexpectedType("string", "object")),
            Json::Array(_) => return Err(FromJsonError::UnexpectedType("array", "object")),
        };

        Ok(Self {
            iso_23220: OrgIso232201::from_json_opt(map.get(org_iso_23220_1::NAMESPACE))
                .map_err(|e| FromJsonError::WithContext(org_iso_23220_1::NAMESPACE, Box::new(e)))?,
            photo_id: Option::<OrgIso23220Photoid1>::from_json_opt(
                map.get(org_iso_23220_photoid_1::NAMESPACE),
            )
            .map_err(|e| {
                FromJsonError::WithContext(org_iso_23220_photoid_1::NAMESPACE, Box::new(e))
            })?,
        })
    }
}

#[cfg(test)]
#[path = "tests/photo_id.rs"]
mod tests;
