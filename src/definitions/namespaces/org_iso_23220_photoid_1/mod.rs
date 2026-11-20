//! The `org.iso.23220.photoid.1` namespace of ISO/IEC TS 23220-4 Annex C Table C.2.
//!
//! The namespace identifier is also the photo ID doc type,
//! [`PHOTO_ID_DOC_TYPE`](crate::definitions::PHOTO_ID_DOC_TYPE).

pub use super::{latin1::Latin1, org_iso_18013_5_1::Alpha2, tstr150::Tstr150};

use crate::{
    definitions::helpers::ByteStr,
    macros::{FromJson, ToCbor},
};

/// The `org.iso.23220.photoid.1` namespace identifier.
pub const NAMESPACE: &str = "org.iso.23220.photoid.1";

/// The data elements ISO/IEC TS 23220-4 Annex C defines specifically for the photo ID.
///
/// Every element is optional. Table C.2 makes `travel_document_type` and
/// `travel_document_mrz` conditional on `dg1` in `org.iso.23220.datagroups.1`, a namespace
/// this crate does not model; without it they are optional.
///
/// `family_name_viz` and `given_name_viz` are defined here, in Table C.2, rather than in
/// `org.iso.23220.1`.
#[derive(Debug, Clone, FromJson, ToCbor)]
#[isomdl(crate = "crate")]
pub struct OrgIso23220Photoid1 {
    /// Person identifier of the photo ID holder.
    pub person_id: Option<Tstr150>,
    /// The country where the holder was born, as an ISO 3166-1 alpha-2 code.
    pub birth_country: Option<Alpha2>,
    /// The state, province, district or local area where the holder was born.
    pub birth_state: Option<Tstr150>,
    /// The municipality, city, town or village where the holder was born.
    pub birth_city: Option<Tstr150>,
    /// A number assigned by the issuer for audit control or other purposes.
    pub administrative_number: Option<Tstr150>,
    /// The name of the street where the holder currently resides.
    pub resident_street: Option<Tstr150>,
    /// The house number where the holder currently resides, including any affix or suffix.
    pub resident_house_number: Option<Tstr150>,
    /// The type of the source travel document, e.g. the two letters of the MRZ as defined
    /// by ICAO 9303.
    pub travel_document_type: Option<Tstr150>,
    /// The number of the travel document the photo ID is associated with or derived from.
    pub travel_document_number: Option<Tstr150>,
    /// The state, province or district where the holder lives, in Latin 1 characters only.
    pub resident_state: Option<Latin1>,
    /// The machine readable zone as printed on the physical document.
    pub travel_document_mrz: Option<Tstr150>,
    /// Family name as defined for the visual inspection zone in ICAO 9303.
    pub family_name_viz: Option<Tstr150>,
    /// Given name as defined for the visual inspection zone in ICAO 9303.
    pub given_name_viz: Option<Tstr150>,
    /// The portrait taken at enrolment, encoded as `portrait` is in ISO/IEC TS 23220-2.
    pub enrolment_portrait_image: Option<ByteStr>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::definitions::traits::{FromJson, ToNamespaceMap};

    #[test]
    fn empty() {
        let ns = OrgIso23220Photoid1::from_json(&serde_json::json!({})).unwrap();
        assert!(ns.to_ns_map().is_empty());
    }

    #[test]
    fn all_elements() {
        let json = serde_json::json!({
            "person_id": "1234567890",
            "birth_country": "DE",
            "birth_state": "Berlin",
            "birth_city": "Berlin",
            "administrative_number": "A-1",
            "resident_street": "Unter den Linden",
            "resident_house_number": "1a",
            "travel_document_type": "P",
            "travel_document_number": "C01X00T47",
            "resident_state": "Berlin",
            "travel_document_mrz": "P<D<<MUSTERMANN<<ERIKA<<<<<<<<<<<<<<<<<<<<<<",
            "family_name_viz": "MUSTERMANN",
            "given_name_viz": "ERIKA",
            "enrolment_portrait_image": include_str!("../../../../test/issuance/portrait.b64"),
        });
        let encoded = OrgIso23220Photoid1::from_json(&json).unwrap().to_ns_map();
        assert_eq!(encoded.len(), 14);
        assert!(matches!(
            encoded["enrolment_portrait_image"],
            ciborium::Value::Bytes(_)
        ));
        assert_eq!(
            encoded["birth_country"],
            ciborium::Value::Text("DE".to_string())
        );
    }

    #[test]
    fn resident_state_is_latin1() {
        let json = serde_json::json!({ "resident_state": "東京都" });
        assert!(OrgIso23220Photoid1::from_json(&json).is_err());
    }

    #[test]
    fn birth_country_is_alpha2() {
        let json = serde_json::json!({ "birth_country": "DEU" });
        assert!(OrgIso23220Photoid1::from_json(&json).is_err());
    }
}
