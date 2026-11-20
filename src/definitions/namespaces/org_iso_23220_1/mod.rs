//! The `org.iso.23220.1` namespace, as the photo ID uses it.
//!
//! ISO/IEC TS 23220-2 defines the data elements of `org.iso.23220.1` as building blocks and
//! leaves it to each doc type to choose which it uses and which are mandatory. This module
//! models the choice made by the photo ID profile of ISO/IEC TS 23220-4 Annex C: the
//! elements of Table C.1 with the presence listed there.
//!
//! See [`photo_id`](super::photo_id) for the whole document, including the
//! [`org.iso.23220.photoid.1`](super::org_iso_23220_photoid_1) namespace.

pub mod age_over;
pub mod birth_date;
pub mod country_code;
pub mod issuing_subdivision;

pub use super::{
    age_over::AgeOver, fulldate::FullDate, latin1::Latin1, org_iso_18013_5_1::Alpha2, sex::Sex,
    tdate::TDate, tstr150::Tstr150,
};
pub use age_over::PhotoIdAgeOver;
pub use birth_date::{ApproximateMask, BirthDate};
pub use country_code::CountryCode;
pub use issuing_subdivision::IssuingSubdivision;

use crate::{
    definitions::helpers::ByteStr,
    macros::{FromJson, ToCbor},
};

/// The `org.iso.23220.1` namespace identifier.
pub const NAMESPACE: &str = "org.iso.23220.1";

/// The `org.iso.23220.1` data elements of a photo ID.
///
/// Fields follow ISO/IEC TS 23220-4:2026 Annex C Table C.1 for presence and
/// ISO/IEC TS 23220-2:2026 (Tables 2, 5, 7 and 8) for encoding. Where these differ from `org.iso.18013.5.1`:
///
/// - Names, addresses and `issuing_authority` are Unicode `tstr`, with separate `_latin1`
///   variants. The first edition of 23220-2 suffixed the Unicode ones with `_unicode`; the
///   current edition deprecates that, and so does this struct.
/// - `birth_date` is a map that can carry an approximate mask ([`BirthDate`]).
/// - `issuing_country` and `nationality` may be alpha-2 or alpha-3 ([`CountryCode`]).
/// - `issue_date` and `expiry_date` are `full-date` only, never `tdate`.
/// - `issuing_subdivision` (ISO 3166-2) takes the place of `issuing_jurisdiction`.
/// - `sex` value 9 (encoded by [`Sex::NotApplicable`]) means "not specified", and is what
///   a photo ID uses for `X`.
/// - Every `tstr` is at most 150 characters (Annex C, C.2.1).
/// - `document_number` is optional.
#[derive(Debug, Clone, FromJson, ToCbor)]
#[isomdl(crate = "crate")]
pub struct OrgIso232201 {
    // Mandatory (Table C.1, M).
    pub family_name: Tstr150,
    pub given_name: Tstr150,
    pub birth_date: BirthDate,
    /// JPEG or JPEG 2000 image data, not wrapped in a TLV structure (23220-2 Table 2,
    /// footnote d).
    pub portrait: ByteStr,
    pub issue_date: FullDate,
    pub expiry_date: FullDate,
    pub issuing_authority: Tstr150,
    pub issuing_country: CountryCode,
    /// Every `age_over_NN`. `age_over_18` is mandatory; the others are recommended.
    #[isomdl(many)]
    pub age_over_xx: PhotoIdAgeOver,

    // Recommended (Table C.1, R).
    pub age_in_years: Option<u32>,
    pub age_birth_year: Option<u32>,

    // Optional (Table C.1, O).
    pub portrait_capture_date: Option<TDate>,
    pub birthplace: Option<Tstr150>,
    pub name_at_birth: Option<Tstr150>,
    pub resident_address: Option<Tstr150>,
    pub resident_city: Option<Tstr150>,
    pub resident_postal_code: Option<Tstr150>,
    pub resident_country: Option<Alpha2>,
    pub resident_city_latin1: Option<Latin1>,
    pub sex: Option<Sex>,
    pub nationality: Option<CountryCode>,
    pub document_number: Option<Tstr150>,
    pub issuing_subdivision: Option<IssuingSubdivision>,
    pub family_name_latin1: Option<Latin1>,
    pub given_name_latin1: Option<Latin1>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::definitions::traits::{FromJson, ToNamespaceMap};

    fn mandatory() -> serde_json::Value {
        serde_json::json!({
            "family_name": "Smith",
            "given_name": "Alice",
            "birth_date": "1990-04-15",
            "portrait": include_str!("../../../../test/issuance/portrait.b64"),
            "issue_date": "2020-01-01",
            "expiry_date": "2030-01-01",
            "issuing_authority": "Utopia Department of Identity",
            "issuing_country": "US",
            "age_over_18": true
        })
    }

    #[test]
    fn mandatory_elements_only() {
        let ns = OrgIso232201::from_json(&mandatory()).unwrap();
        assert_eq!(ns.family_name.as_str(), "Smith");
        assert!(ns.age_over_xx.age_over_18());
        assert!(ns.document_number.is_none());
    }

    #[test]
    fn each_mandatory_element_is_required() {
        for element in [
            "family_name",
            "given_name",
            "birth_date",
            "portrait",
            "issue_date",
            "expiry_date",
            "issuing_authority",
            "issuing_country",
            "age_over_18",
        ] {
            let mut json = mandatory();
            json.as_object_mut().unwrap().remove(element);
            assert!(
                OrgIso232201::from_json(&json).is_err(),
                "{element} should be mandatory"
            );
        }
    }

    #[test]
    fn deprecated_unicode_identifiers_are_not_read() {
        let mut json = mandatory();
        let map = json.as_object_mut().unwrap();
        map.remove("issuing_authority");
        map.insert(
            "issuing_authority_unicode".into(),
            serde_json::json!("Utopia Department of Identity"),
        );
        assert!(OrgIso232201::from_json(&json).is_err());
    }

    #[test]
    fn tstr_is_limited_to_150_characters() {
        let mut json = mandatory();
        json["family_name"] = serde_json::json!("a".repeat(151));
        assert!(OrgIso232201::from_json(&json).is_err());
    }

    #[test]
    fn all_elements() {
        let json = serde_json::json!({
            "family_name": "Müller",
            "given_name": "Hans",
            "family_name_latin1": "Müller",
            "given_name_latin1": "Hans",
            "birth_date": { "birth_date": "1975-03-01", "approximate_mask": "00000011" },
            "portrait": include_str!("../../../../test/issuance/portrait.b64"),
            "portrait_capture_date": "2021-06-01T09:00:00Z",
            "issue_date": "2021-06-01",
            "expiry_date": "2031-06-01",
            "issuing_authority": "Landesamt Berlin",
            "issuing_country": "DEU",
            "issuing_subdivision": "DE-BE",
            "document_number": "D123456789",
            "sex": 9,
            "birthplace": "Berlin",
            "name_at_birth": "Müller",
            "nationality": "DEU",
            "resident_address": "Unter den Linden 1, 10117 Berlin",
            "resident_city": "Berlin",
            "resident_city_latin1": "Berlin",
            "resident_postal_code": "10117",
            "resident_country": "DE",
            "age_in_years": 49,
            "age_birth_year": 1975,
            "age_over_18": true,
            "age_over_65": false
        });
        let ns = OrgIso232201::from_json(&json).unwrap();
        assert_eq!(ns.issuing_country.as_str(), "DEU");
        assert_eq!(ns.issuing_subdivision.as_ref().unwrap().as_str(), "DE-BE");
        assert!(matches!(ns.sex, Some(Sex::NotApplicable)));

        let encoded = ns.to_ns_map();
        let keys: Vec<&str> = encoded.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "age_birth_year",
                "age_in_years",
                "age_over_18",
                "age_over_65",
                "birth_date",
                "birthplace",
                "document_number",
                "expiry_date",
                "family_name",
                "family_name_latin1",
                "given_name",
                "given_name_latin1",
                "issue_date",
                "issuing_authority",
                "issuing_country",
                "issuing_subdivision",
                "name_at_birth",
                "nationality",
                "portrait",
                "portrait_capture_date",
                "resident_address",
                "resident_city",
                "resident_city_latin1",
                "resident_country",
                "resident_postal_code",
                "sex",
            ]
        );
        assert!(matches!(encoded["portrait"], ciborium::Value::Bytes(_)));
        assert!(matches!(encoded["birth_date"], ciborium::Value::Map(_)));
        assert!(matches!(
            encoded["issue_date"],
            ciborium::Value::Tag(1004, _)
        ));
        assert!(matches!(
            encoded["portrait_capture_date"],
            ciborium::Value::Tag(0, _)
        ));
    }

    /// 23220-2 Table 2 encodes `portrait_capture_date` as a `tdate`, not a `full-date`.
    #[test]
    fn portrait_capture_date_is_a_tdate() {
        let mut json = mandatory();
        json["portrait_capture_date"] = serde_json::json!("2021-06-01");
        assert!(OrgIso232201::from_json(&json).is_err());
    }

    /// Table C.2 defines the VIZ names in `org.iso.23220.photoid.1`, so they are not read here.
    #[test]
    fn viz_names_are_not_read_here() {
        let mut json = mandatory();
        json["family_name_viz"] = serde_json::json!("SMITH");
        let encoded = OrgIso232201::from_json(&json).unwrap().to_ns_map();
        assert!(!encoded.contains_key("family_name_viz"));
    }
}
