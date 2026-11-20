use super::*;

fn iso_23220() -> Json {
    serde_json::json!({
        "family_name": "Smith",
        "given_name": "Alice",
        "birth_date": "1990-04-15",
        "portrait": include_str!("../../../../test/issuance/portrait.b64"),
        "issue_date": "2020-01-01",
        "expiry_date": "2030-01-01",
        "issuing_authority": "Utopia Department of Identity",
        "issuing_country": "UT",
        "age_over_18": true
    })
}

fn namespace_ids(namespaces: &BTreeMap<String, BTreeMap<String, ciborium::Value>>) -> Vec<&str> {
    namespaces.keys().map(String::as_str).collect()
}

#[test]
fn the_doc_type_is_annex_c_s() {
    assert_eq!(DOC_TYPE, "org.iso.23220.photoid.1");
    assert_eq!(org_iso_23220_photoid_1::NAMESPACE, DOC_TYPE);
}

#[test]
fn only_org_iso_23220_1_is_required() {
    let photo_id = PhotoId::from_json(&serde_json::json!({ "org.iso.23220.1": iso_23220() }))
        .unwrap()
        .into_namespaces();
    assert_eq!(namespace_ids(&photo_id), ["org.iso.23220.1"]);
}

#[test]
fn a_missing_org_iso_23220_1_is_an_error() {
    let err = PhotoId::from_json(&serde_json::json!({
        "org.iso.23220.photoid.1": { "person_id": "1" }
    }))
    .unwrap_err();
    assert!(
        matches!(err, FromJsonError::WithContext("org.iso.23220.1", _)),
        "{err:?}"
    );
}

#[test]
fn errors_name_the_namespace() {
    let err = PhotoId::from_json(&serde_json::json!({
        "org.iso.23220.1": iso_23220(),
        "org.iso.23220.photoid.1": { "birth_country": "not a country" }
    }))
    .unwrap_err();
    assert!(
        matches!(
            err,
            FromJsonError::WithContext("org.iso.23220.photoid.1", _)
        ),
        "{err:?}"
    );
}

#[test]
fn an_empty_photo_id_namespace_is_left_out() {
    let photo_id = PhotoId::from_json(&serde_json::json!({
        "org.iso.23220.1": iso_23220(),
        "org.iso.23220.photoid.1": {}
    }))
    .unwrap()
    .into_namespaces();
    assert_eq!(namespace_ids(&photo_id), ["org.iso.23220.1"]);
}

#[test]
fn both_namespaces() {
    let photo_id = PhotoId::from_json(&serde_json::json!({
        "org.iso.23220.1": iso_23220(),
        "org.iso.23220.photoid.1": {
            "travel_document_type": "P",
            "family_name_viz": "SMITH",
            "given_name_viz": "ALICE"
        }
    }))
    .unwrap();

    assert_eq!(
        photo_id
            .photo_id
            .as_ref()
            .and_then(|p| p.family_name_viz.as_deref()),
        Some("SMITH")
    );

    let namespaces = photo_id.into_namespaces();
    assert_eq!(
        namespace_ids(&namespaces),
        ["org.iso.23220.1", "org.iso.23220.photoid.1"]
    );
    assert_eq!(
        namespaces["org.iso.23220.photoid.1"]["given_name_viz"],
        ciborium::Value::Text("ALICE".into())
    );
}
