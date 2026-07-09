# isomdl-uniffi Changes Required for ISO 23220-1 (Photo ID)

Repository: https://github.com/Indicio-tech/isomdl-uniffi

## What does not need to change

The reader and holder sides are fully namespace-agnostic and require no
modifications:

- `establish_session` — accepts any `HashMap<String, HashMap<String, bool>>`
  for requested items
- `handle_response` — returns generic `HashMap<String, HashMap<String, MDocItem>>`
- `verify_oid4vp_response` — parses any `DeviceResponse` regardless of doctype
- `MdlPresentationSession` — holder side works for any doctype/namespace
- `Mdoc::from_*` constructors — work for any `IssuerSigned` / `Document`

## What needs to change: issuance

The wrapper has two tiers for issuance:

| Tier | Methods | How it works |
|------|---------|--------------|
| **Generic** | `Mdoc::create_and_sign`, `PreparedMdoc::new` | `convert_namespaces_json`: JSON values → CBOR naively (strings stay as `tstr`) |
| **Typed** | `Mdoc::create_and_sign_mdl`, `PreparedMdoc::new_mdl` | `OrgIso1801351::from_json` + `to_ns_map()`: produces correctly-typed CBOR |

Without a typed path for ISO 23220-1, callers fall back to the generic path,
which mis-encodes several required fields:

| Field | Generic encoding | Correct encoding |
|-------|-----------------|-----------------|
| `portrait` | CBOR `tstr` (base64 string) | CBOR `bstr` (bytes) |
| `birth_date` | CBOR `tstr` | CBOR map `{ "birth_date": tag(1004, "YYYY-MM-DD"), ? "approximate_mask": tstr }` |
| `issue_date` / `expiry_date` | CBOR `tstr` | CBOR `full-date` (tag 1004) |
| `biometric_template_face`, `fingerprint` | CBOR `tstr` | CBOR `bstr` |

## Required additions to `rust/src/mdl/mdoc.rs`

Add the following import at the top of the file alongside the existing namespace
imports:

```rust
use isomdl::definitions::namespaces::org_iso_23220_1::OrgIso232201;
```

### 1. `Mdoc::create_and_sign_photoid`

Parallel to `create_and_sign_mdl`. Add inside `impl Mdoc`:

```rust
/// Create and sign an ISO 23220-1 photo ID (`org.iso.23220.1.mID`) document.
///
/// Uses [`OrgIso232201::from_json`] to ensure correct CBOR field types
/// (e.g. `birth_date` as a CBOR map, `portrait` as `bstr`, dates as
/// `full-date` tag 1004).
///
/// * `photoid_items` — JSON object string with `org.iso.23220.1` namespace elements.
/// * `holder_jwk` — P-256 JWK of the holder's device key.
/// * `iaca_cert_pem` — PEM-encoded IACA certificate.
/// * `iaca_key_pem` — PEM-encoded IACA private key.
#[uniffi::constructor]
pub fn create_and_sign_photoid(
    photoid_items: String,
    holder_jwk: String,
    iaca_cert_pem: String,
    iaca_key_pem: String,
) -> Result<Arc<Self>, MdocInitError> {
    let pub_key: PublicKey =
        PublicKey::from_jwk_str(&holder_jwk).map_err(|_| MdocInitError::InvalidJwk)?;

    let json_value: serde_json::Value = serde_json::from_str(&photoid_items)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("photoid_items JSON parse: {e}")))?;

    let ns_data = OrgIso232201::from_json(&json_value)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!(
            "photo ID namespace parse (org.iso.23220.1): {e}"
        )))?
        .to_ns_map();

    let mut namespaces = BTreeMap::new();
    namespaces.insert("org.iso.23220.1".to_string(), ns_data);

    let doc_type = "org.iso.23220.1.mID".to_string();
    let builder = prepare_builder(pub_key, namespaces, doc_type)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("prepare_builder: {e}")))?;

    let (certificate, iaca_certs, signer) =
        setup_certificate_chain(iaca_cert_pem, iaca_key_pem, None)
            .map_err(|e| MdocInitError::GeneralConstructionError(format!("certificate chain: {e}")))?;

    let mut x5chain_builder = X5Chain::builder()
        .with_certificate(certificate)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("x5chain certificate: {e}")))?;

    for cert in iaca_certs {
        x5chain_builder = x5chain_builder
            .with_certificate(cert)
            .map_err(|e| MdocInitError::GeneralConstructionError(format!("x5chain intermediate: {e}")))?;
    }

    let x5chain = x5chain_builder
        .build()
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("x5chain build: {e}")))?;

    let mdoc = builder
        .issue::<p256::ecdsa::SigningKey, p256::ecdsa::Signature>(x5chain, signer)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("issue: {e}")))?;

    let doc = issuance_mdoc_to_document(mdoc)?;
    Ok(Arc::new(Self::new_from_parts(doc, KeyAlias(Uuid::new_v4().to_string()))))
}
```

### 2. `PreparedMdoc::new_photoid`

Parallel to `PreparedMdoc::new_mdl`. Add inside `impl PreparedMdoc`:

```rust
/// Prepare an ISO 23220-1 photo ID (`org.iso.23220.1.mID`) document for
/// external signing.
///
/// Uses [`OrgIso232201::from_json`] to ensure correct CBOR field types.
/// Call [`signature_payload()`] to get the bytes to sign, then
/// [`complete()`] to finalize.
///
/// * `photoid_items` — JSON object string with `org.iso.23220.1` namespace elements.
/// * `holder_jwk` — P-256 JWK of the holder's device key.
/// * `signature_algorithm` — one of `"ES256"`, `"ES384"`, `"ES512"`.
#[uniffi::constructor]
pub fn new_photoid(
    photoid_items: String,
    holder_jwk: String,
    signature_algorithm: String,
) -> Result<Arc<Self>, MdocInitError> {
    let pub_key: PublicKey =
        PublicKey::from_jwk_str(&holder_jwk).map_err(|_| MdocInitError::InvalidJwk)?;

    let json_value: serde_json::Value = serde_json::from_str(&photoid_items)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("photoid_items JSON parse: {e}")))?;

    let ns_data = OrgIso232201::from_json(&json_value)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!(
            "photo ID namespace parse (org.iso.23220.1): {e}"
        )))?
        .to_ns_map();

    let mut namespaces = BTreeMap::new();
    namespaces.insert("org.iso.23220.1".to_string(), ns_data);

    let doc_type = "org.iso.23220.1.mID".to_string();
    let builder = prepare_builder(pub_key, namespaces, doc_type)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("prepare_builder: {e}")))?;

    let algorithm = parse_signature_algorithm(&signature_algorithm)?;
    let prepared = builder
        .prepare(algorithm)
        .map_err(|e| MdocInitError::GeneralConstructionError(format!("prepare: {e}")))?;

    Ok(Arc::new(Self {
        inner: std::sync::Mutex::new(Some(prepared)),
    }))
}
```

## Example `photoid_items` JSON

The JSON passed to either constructor must match the `OrgIso232201` struct.
Mandatory fields:

```json
{
    "family_name": "Smith",
    "given_name": "Alice",
    "birth_date": "1990-04-15",
    "portrait": "<base64-encoded JPEG>",
    "issue_date": "2024-01-01",
    "expiry_date": "2034-01-01",
    "issuing_country": "US",
    "issuing_authority_unicode": "Department of State",
    "document_number": "ID12345678"
}
```

`birth_date` may also be supplied as an object with an optional mask for
partially unknown dates:

```json
{
    "birth_date": {
        "birth_date": "1990-01-01",
        "approximate_mask": "00000100"
    }
}
```

`issuing_country` and `nationality` accept both alpha-2 (`"US"`) and
alpha-3 (`"USA"`) codes. All other string fields are full Unicode (`tstr`);
the `_latin1` variants (e.g. `"family_name_latin1"`) are optional and
constrained to Latin-1 characters.
