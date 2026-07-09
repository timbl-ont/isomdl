pub mod birth_date;
pub mod country_code;

pub use super::{
    age_over::AgeOver,
    fulldate::FullDate,
    latin1::Latin1,
    sex::Sex,
    tdate::TDate,
};
pub use birth_date::BirthDate;
pub use country_code::CountryCode;

// Alpha2 is reused from org.iso.18013.5.1 for fields that are strictly alpha-2
// (e.g. resident_country per ISO/IEC TS 23220-2 Table 2).
pub use super::org_iso_18013_5_1::Alpha2;

use crate::{
    definitions::helpers::ByteStr,
    macros::{FromJson, ToCbor},
};

/// The `org.iso.23220.1` namespace, as defined in ISO/IEC TS 23220-2.
///
/// Namespace string : `"org.iso.23220.1"`
/// DocType          : `"org.iso.23220.1.mID"`
///
/// Key differences from `org.iso.18013.5.1`:
/// - `birth_date` is a structured map (see [`BirthDate`]) rather than a plain `full-date`.
/// - `issuing_country` and `nationality` accept alpha-2 **or** alpha-3 codes ([`CountryCode`]).
/// - `issuing_authority` is split into `issuing_authority_unicode` (tstr) and
///   `issuing_authority_latin1` (Latin1).
/// - `issuing_subdivision` (ISO 3166-2) replaces `issuing_jurisdiction`.
/// - `issue_date` / `expiry_date` are `full-date`, not `tdate`-or-`full-date`.
/// - No driving-licence-specific fields (`driving_privileges`, `un_distinguishing_sign`).
/// - Adds `birthplace`, `resident_street`, `biometric_template_face`, `fingerprint`,
///   `name_at_birth`, `issuing_subdivision`, and `document_type`.
/// - Resident-address fields have both a Unicode tstr variant and a `_latin1` variant.
#[derive(Debug, Clone, FromJson, ToCbor)]
#[isomdl(crate = "crate")]
pub struct OrgIso232201 {
    // ── Mandatory personal attributes ──────────────────────────────────────
    pub family_name: String,
    pub given_name: String,
    pub birth_date: BirthDate,
    pub portrait: ByteStr,

    // ── Mandatory document / issuer attributes ──────────────────────────────
    pub issue_date: FullDate,
    pub expiry_date: FullDate,
    pub issuing_country: CountryCode,
    pub issuing_authority_unicode: String,
    pub document_number: String,

    // ── Optional personal attributes ───────────────────────────────────────
    pub family_name_latin1: Option<Latin1>,
    pub given_name_latin1: Option<Latin1>,
    pub sex: Option<Sex>,
    /// Height in centimetres.
    pub height: Option<u32>,
    /// Weight in kilograms.
    pub weight: Option<u32>,
    /// Country and municipality / state where the holder was born (tstr).
    pub birthplace: Option<String>,
    pub name_at_birth: Option<String>,
    pub nationality: Option<CountryCode>,

    // ── Optional residence attributes ───────────────────────────────────────
    pub resident_address: Option<String>,
    pub resident_address_latin1: Option<Latin1>,
    pub resident_city: Option<String>,
    pub resident_city_latin1: Option<Latin1>,
    pub resident_state: Option<String>,
    pub resident_state_latin1: Option<Latin1>,
    pub resident_street: Option<String>,
    pub resident_street_latin1: Option<Latin1>,
    pub resident_postal_code: Option<String>,
    /// Alpha-2 country code only (ISO/IEC TS 23220-2 Table 2).
    pub resident_country: Option<Alpha2>,

    // ── Optional biometric / portrait attributes ────────────────────────────
    /// Face biometric template (CBEFF/ISO 19794-5 encoded, wrapped in BIT).
    pub biometric_template_face: Option<ByteStr>,
    pub portrait_capture_date: Option<TDate>,
    /// Fingerprint biometric data (CBEFF/ISO 19794-4 encoded).
    pub fingerprint: Option<ByteStr>,

    // ── Optional issuer / document attributes ───────────────────────────────
    pub issuing_authority_latin1: Option<Latin1>,
    /// Issuing subdivision code as defined in ISO 3166-2.
    pub issuing_subdivision: Option<String>,
    /// Free-form document type string (e.g. `"ID"`, `"eID"`).
    pub document_type: Option<String>,

    // ── Age attestations (Table 5) ──────────────────────────────────────────
    pub age_in_years: Option<u32>,
    pub age_birth_year: Option<u32>,
    #[isomdl(many)]
    pub age_over_xx: AgeOver,
    // ── Optional professional attributes ─────────────────────────────────────
    pub profession: Option<String>,
    /// Academic title of the holder.
    pub title: Option<String>,
}

/// Namespace identifier for `org.iso.23220.1`.
pub const NAMESPACE: &str = "org.iso.23220.1";

/// DocType identifier for `org.iso.23220.1.mID`.
pub const DOC_TYPE: &str = "org.iso.23220.1.mID";

#[cfg(test)]
mod test {
    use super::*;
    use crate::definitions::traits::FromJson;

    #[test]
    fn minimal() {
        let json = serde_json::json!({
            "family_name": "Smith",
            "given_name": "Alice",
            "birth_date": "1990-04-15",
            "portrait": include_str!("../../../../test/issuance/portrait.b64"),
            "issue_date": "2020-01-01",
            "expiry_date": "2030-01-01",
            "issuing_country": "US",
            "issuing_authority_unicode": "NY DMV",
            "document_number": "ID12345678"
        });
        let ns = OrgIso232201::from_json(&json).unwrap();
        assert_eq!(ns.family_name, "Smith");
        assert_eq!(ns.given_name, "Alice");
        assert!(ns.sex.is_none());
        assert!(ns.age_over_xx.is_empty());
    }

    #[test]
    fn full() {
        let json = serde_json::json!({
            "family_name": "Müller",
            "given_name": "Hans",
            "family_name_latin1": "Muller",
            "given_name_latin1": "Hans",
            "birth_date": {
                "birth_date": "1975-03-20",
                "approximate_mask": "00000000"
            },
            "portrait": include_str!("../../../../test/issuance/portrait.b64"),
            "issue_date": "2021-06-01",
            "expiry_date": "2031-06-01",
            "issuing_country": "DEU",
            "issuing_authority_unicode": "Einwohnermeldeamt Berlin",
            "issuing_authority_latin1": "Einwohnermeldeamt Berlin",
            "issuing_subdivision": "DE-BE",
            "document_number": "D123456789",
            "document_type": "ID",
            "sex": 1,
            "height": 182,
            "weight": 80,
            "birthplace": "Berlin",
            "name_at_birth": "Müller",
            "nationality": "DEU",
            "resident_address": "Unter den Linden 1",
            "resident_address_latin1": "Unter den Linden 1",
            "resident_city": "Berlin",
            "resident_city_latin1": "Berlin",
            "resident_state": "Berlin",
            "resident_state_latin1": "Berlin",
            "resident_street": "Unter den Linden 1",
            "resident_street_latin1": "Unter den Linden 1",
            "resident_postal_code": "10117",
            "resident_country": "DE",
            "portrait_capture_date": "2021-06-01T09:00:00Z",
            "age_in_years": 49,
            "age_birth_year": 1975,
            "age_over_18": true,
            "age_over_21": true
        });
        let ns = OrgIso232201::from_json(&json).unwrap();
        assert!(ns.family_name_latin1.is_some());
        assert!(ns.given_name_latin1.is_some());
        assert!(ns.sex.is_some());
        assert!(ns.height.is_some());
        assert!(ns.weight.is_some());
        assert!(ns.birthplace.is_some());
        assert!(ns.name_at_birth.is_some());
        assert!(ns.nationality.is_some());
        assert!(ns.resident_address.is_some());
        assert!(ns.resident_address_latin1.is_some());
        assert!(ns.resident_city.is_some());
        assert!(ns.resident_state.is_some());
        assert!(ns.resident_street.is_some());
        assert!(ns.resident_postal_code.is_some());
        assert!(ns.resident_country.is_some());
        assert!(ns.portrait_capture_date.is_some());
        assert!(ns.issuing_authority_latin1.is_some());
        assert!(ns.issuing_subdivision.is_some());
        assert!(ns.document_type.is_some());
        assert!(ns.age_in_years.is_some());
        assert!(ns.age_birth_year.is_some());
        assert!(ns.age_over_xx.get(&18.try_into().unwrap()).unwrap());
        assert!(ns.age_over_xx.get(&21.try_into().unwrap()).unwrap());
        // issuing_country with alpha-3
        assert_eq!(ns.issuing_country.as_str(), "DEU");
        // birth_date with approximate_mask
        assert_eq!(
            ns.birth_date.approximate_mask.as_deref(),
            Some("00000000")
        );
    }
}
