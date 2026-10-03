use crate::{Error, Result, compute_sha256, read_bundle_manifest, safe_join_existing};
use cmpv2::status::PkiStatus;
use cms::signed_data::SignedData;
use der::asn1::{Any, Int, OctetString};
use der::oid::ObjectIdentifier;
use der::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use x509_tsp::{MessageImprint, TimeStampReq, TimeStampResp, TspVersion, TstInfo};

pub const PLAIN_ROOT_VERSION: &str = "tatacoa.plain-root.v1";
pub const MAX_TIMESTAMP_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const DOMAIN: &[u8] = b"tatacoa.plain-root.v1\0";
const SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
const SIGNED_DATA_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.2");
const TST_INFO_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.1.4");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainRootDigest {
    pub algorithm: &'static str,
    pub value: String,
    pub entry_count: u32,
}

impl PlainRootDigest {
    /// Returns the RFC 3161 SHA-256 messageImprint bytes directly.
    /// Callers must not hash this value a second time.
    pub fn message_imprint(&self) -> Result<[u8; 32]> {
        decode_sha256(&self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampObject<'a> {
    PlainBundle(&'a Path),
    EncryptedBundle(&'a Path),
}

impl TimestampObject<'_> {
    fn message_imprint(self) -> Result<[u8; 32]> {
        match self {
            Self::PlainBundle(path) => compute_plain_root(path)?.message_imprint(),
            Self::EncryptedBundle(path) => {
                let (_, digest) = compute_sha256(path)?;
                decode_sha256(&digest)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TsaConfig {
    endpoint: String,
    timeout: Duration,
    trust_policy: Option<TsaTrustPolicy>,
}

impl TsaConfig {
    pub fn new(endpoint: String, timeout: Duration) -> Result<Self> {
        let endpoint = endpoint.trim();
        if !endpoint.starts_with("https://")
            || endpoint.len() <= "https://".len()
            || endpoint.bytes().any(|byte| byte.is_ascii_whitespace())
            || endpoint.contains(['@', '?', '#'])
        {
            return Err(Error::Timestamp(
                "TSA endpoint must be explicit HTTPS without credentials, query or fragment"
                    .to_owned(),
            ));
        }
        if timeout.is_zero() || timeout > Duration::from_secs(120) {
            return Err(Error::Timestamp(
                "TSA timeout must be between 1 ms and 120 seconds".to_owned(),
            ));
        }
        Ok(Self {
            endpoint: endpoint.to_owned(),
            timeout,
            trust_policy: None,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn with_trust_policy(mut self, trust_policy: TsaTrustPolicy) -> Self {
        self.trust_policy = Some(trust_policy);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsaTrustPolicy {
    pub(crate) trust_anchors: Vec<Vec<u8>>,
    pub(crate) intermediates: Vec<Vec<u8>>,
    pub(crate) accepted_policy_oids: Vec<ObjectIdentifier>,
}

impl TsaTrustPolicy {
    pub fn new(
        trust_anchors_der: Vec<Vec<u8>>,
        intermediates_der: Vec<Vec<u8>>,
        accepted_policy_oids: Vec<String>,
    ) -> Result<Self> {
        if trust_anchors_der.is_empty() || accepted_policy_oids.is_empty() {
            return Err(Error::Timestamp(
                "TSA trust requires at least one explicit anchor and accepted policy".to_owned(),
            ));
        }
        for certificate in trust_anchors_der.iter().chain(&intermediates_der) {
            x509_cert::Certificate::from_der(certificate).map_err(|error| {
                Error::Timestamp(format!("decode explicit TSA trust certificate: {error}"))
            })?;
        }
        let accepted_policy_oids = accepted_policy_oids
            .into_iter()
            .map(|value| {
                ObjectIdentifier::new(&value).map_err(|_| {
                    Error::Timestamp("accepted TSA policy must be a valid OID".to_owned())
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            trust_anchors: trust_anchors_der,
            intermediates: intermediates_der,
            accepted_policy_oids,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimestampAssurance {
    Present,
    Bound,
    SignatureValid,
    Trusted,
    HistoricallyValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimestampCheckStatus {
    Pass,
    Fail,
    NotEvaluated,
    Indeterminate,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimestampCheck {
    pub name: String,
    pub status: TimestampCheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimestampReport {
    pub assurance: TimestampAssurance,
    pub policy_oid: Option<String>,
    pub checks: Vec<TimestampCheck>,
}

/// Performs the only networked timestamp operation. The TSA and destination are explicit.
/// The raw DER response is committed as a new `.tsr` only after local binding succeeds.
pub fn request_timestamp(
    object: TimestampObject<'_>,
    sidecar: &Path,
    config: &TsaConfig,
) -> Result<TimestampReport> {
    if sidecar.extension().and_then(|value| value.to_str()) != Some("tsr") {
        return Err(Error::InvalidPath(
            "RFC 3161 sidecar must use the .tsr extension".to_owned(),
        ));
    }
    if sidecar.exists() {
        return Err(Error::Conflict(format!(
            "timestamp sidecar already exists: {}",
            sidecar.display()
        )));
    }
    let imprint = object.message_imprint()?;
    let nonce = random_nonce()?;
    let request = encode_request(&imprint, &nonce)?;
    let response = send_request(config, &request)?;
    let report = inspect_response(
        &response,
        &imprint,
        Some(&nonce),
        config.trust_policy.as_ref(),
    )?;
    if !matches!(
        report.assurance,
        TimestampAssurance::SignatureValid | TimestampAssurance::Trusted
    ) {
        return Err(Error::Timestamp(
            "TSA response did not satisfy the SIGNATURE_VALID contract".to_owned(),
        ));
    }
    write_sidecar_new(sidecar, &response)?;
    Ok(report)
}

/// Verifies a sidecar without network access. The original nonce is unavailable from a `.tsr`
/// alone, so nonce binding is explicitly NOT_EVALUATED in this entry point.
pub fn verify_timestamp_sidecar(
    object: TimestampObject<'_>,
    sidecar: &Path,
) -> Result<TimestampReport> {
    let metadata =
        fs::metadata(sidecar).map_err(|source| Error::io("inspect RFC 3161 sidecar", source))?;
    if !metadata.is_file() || metadata.len() > MAX_TIMESTAMP_RESPONSE_BYTES {
        return Err(Error::Timestamp(
            "RFC 3161 sidecar exceeds the supported reader limit".to_owned(),
        ));
    }
    let response =
        fs::read(sidecar).map_err(|source| Error::io("read RFC 3161 sidecar", source))?;
    let imprint = object.message_imprint()?;
    inspect_response(&response, &imprint, None, None)
}

pub fn verify_timestamp_sidecar_with_trust(
    object: TimestampObject<'_>,
    sidecar: &Path,
    trust_policy: &TsaTrustPolicy,
) -> Result<TimestampReport> {
    let metadata =
        fs::metadata(sidecar).map_err(|source| Error::io("inspect RFC 3161 sidecar", source))?;
    if !metadata.is_file() || metadata.len() > MAX_TIMESTAMP_RESPONSE_BYTES {
        return Err(Error::Timestamp(
            "RFC 3161 sidecar exceeds the supported reader limit".to_owned(),
        ));
    }
    let response =
        fs::read(sidecar).map_err(|source| Error::io("read RFC 3161 sidecar", source))?;
    let imprint = object.message_imprint()?;
    inspect_response(&response, &imprint, None, Some(trust_policy))
}

fn encode_request(imprint: &[u8; 32], nonce: &[u8]) -> Result<Vec<u8>> {
    let request = TimeStampReq {
        version: TspVersion::V1,
        message_imprint: MessageImprint {
            hash_algorithm: cms::cert::x509::spki::AlgorithmIdentifier {
                oid: SHA256_OID,
                parameters: Some(Any::null()),
            },
            hashed_message: OctetString::new(imprint)
                .map_err(|error| Error::Timestamp(format!("encode messageImprint: {error}")))?,
        },
        req_policy: None,
        nonce: Some(
            Int::new(nonce)
                .map_err(|error| Error::Timestamp(format!("encode request nonce: {error}")))?,
        ),
        cert_req: true,
        extensions: None,
    };
    request
        .to_der()
        .map_err(|error| Error::Timestamp(format!("encode RFC 3161 request: {error}")))
}

fn random_nonce() -> Result<[u8; 16]> {
    let mut nonce = [0_u8; 16];
    getrandom::fill(&mut nonce)
        .map_err(|_| Error::Cryptography("operating system random source unavailable"))?;
    nonce[0] &= 0x7f;
    if nonce.iter().all(|byte| *byte == 0) {
        nonce[15] = 1;
    }
    Ok(nonce)
}

fn send_request(config: &TsaConfig, request: &[u8]) -> Result<Vec<u8>> {
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::Rustls)
        .root_certs(ureq::tls::RootCerts::WebPki)
        .disable_verification(false)
        .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .build();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(config.timeout))
        .tls_config(tls)
        .build()
        .into();
    let mut response = agent
        .post(config.endpoint())
        .header("content-type", "application/timestamp-query")
        .header("accept", "application/timestamp-reply")
        .send(request)
        .map_err(|_| Error::Timestamp("TSA request failed".to_owned()))?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if !content_type.is_some_and(|value| value.eq_ignore_ascii_case("application/timestamp-reply"))
    {
        return Err(Error::Timestamp(
            "TSA response has an unsupported Content-Type".to_owned(),
        ));
    }
    response
        .body_mut()
        .with_config()
        .limit(MAX_TIMESTAMP_RESPONSE_BYTES)
        .read_to_vec()
        .map_err(|error| Error::Timestamp(format!("read TSA response: {error}")))
}

fn inspect_response(
    bytes: &[u8],
    expected_imprint: &[u8; 32],
    expected_nonce: Option<&[u8]>,
    trust_policy: Option<&TsaTrustPolicy>,
) -> Result<TimestampReport> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_TIMESTAMP_RESPONSE_BYTES {
        return Err(Error::Timestamp(
            "RFC 3161 response exceeds the supported reader limit".to_owned(),
        ));
    }
    let response = TimeStampResp::from_der(bytes)
        .map_err(|error| Error::Timestamp(format!("decode RFC 3161 response: {error}")))?;
    if !matches!(
        response.status.status,
        PkiStatus::Accepted | PkiStatus::GrantedWithMods
    ) {
        return Err(Error::Timestamp(format!(
            "TSA rejected the request with status {:?}",
            response.status.status
        )));
    }
    let token = response.time_stamp_token.ok_or_else(|| {
        Error::Timestamp("successful TSA response is missing TimeStampToken".to_owned())
    })?;
    if token.content_type != SIGNED_DATA_OID {
        return Err(Error::Timestamp(
            "TimeStampToken is not CMS SignedData".to_owned(),
        ));
    }
    let signed = SignedData::from_der(
        &token
            .content
            .to_der()
            .map_err(|error| Error::Timestamp(format!("decode CMS content: {error}")))?,
    )
    .map_err(|error| Error::Timestamp(format!("decode CMS SignedData: {error}")))?;
    if signed.encap_content_info.econtent_type != TST_INFO_OID {
        return Err(Error::Timestamp(
            "CMS content is not id-ct-TSTInfo".to_owned(),
        ));
    }
    let content = signed
        .encap_content_info
        .econtent
        .as_ref()
        .ok_or_else(|| Error::Timestamp("CMS token has detached TSTInfo".to_owned()))?;
    let tst = TstInfo::from_der(content.value())
        .map_err(|error| Error::Timestamp(format!("decode TSTInfo: {error}")))?;
    let signature =
        crate::timestamp_signature::evaluate_signature(&signed, content.value(), TST_INFO_OID);

    let algorithm = if tst.message_imprint.hash_algorithm.oid == SHA256_OID {
        TimestampCheckStatus::Pass
    } else {
        TimestampCheckStatus::Unsupported
    };
    let (imprint, nonce) = evaluate_binding(
        algorithm,
        tst.message_imprint.hashed_message.as_bytes(),
        expected_imprint,
        tst.nonce.as_ref().map(Int::as_bytes),
        expected_nonce,
    );
    let bound = algorithm == TimestampCheckStatus::Pass
        && imprint == TimestampCheckStatus::Pass
        && !matches!(nonce, TimestampCheckStatus::Fail);
    let signature_valid = signature.is_valid();
    let trust = trust_policy.map(|policy| {
        crate::timestamp_trust::evaluate_trust(&signed, &tst, policy, signature_valid)
    });
    let checks = vec![
        check(
            "structure",
            TimestampCheckStatus::Pass,
            "DER/CMS/TSTInfo parsed",
        ),
        check("message_imprint_algorithm", algorithm, "expected SHA-256"),
        check(
            "message_imprint",
            imprint,
            "compared with the selected object",
        ),
        check(
            "nonce",
            nonce,
            "compared when the original request is present",
        ),
        check(
            "policy",
            trust
                .as_ref()
                .map_or(TimestampCheckStatus::NotEvaluated, |value| value.policy),
            "compared with the explicit TSA evidence policy",
        ),
        check(
            "cms_signature",
            signature.signature,
            "cryptographic signature over CMS signedAttrs",
        ),
        check(
            "signer_certificate",
            signature.signer_certificate,
            "certificate selected unambiguously by SignerInfo",
        ),
        check(
            "ess_certificate",
            signature.ess_certificate,
            "SigningCertificateV2 binds the signer certificate",
        ),
        check(
            "signed_attributes",
            signature.signed_attributes,
            "signedAttrs are present and DER-encoded for verification",
        ),
        check(
            "content_type",
            signature.content_type,
            "signed content-type matches id-ct-TSTInfo",
        ),
        check(
            "content_message_digest",
            signature.message_digest,
            "signed message-digest matches local TSTInfo digest",
        ),
        TimestampCheck {
            name: "signature_algorithm".to_owned(),
            status: signature.algorithm,
            detail: signature.algorithm_detail,
        },
        check(
            "tsa_trust",
            trust
                .as_ref()
                .map_or(TimestampCheckStatus::NotEvaluated, |value| value.path),
            "TSA trust is separate from TLS trust",
        ),
        check(
            "timestamping_eku",
            trust
                .as_ref()
                .map_or(TimestampCheckStatus::NotEvaluated, |value| value.eku),
            "critical and exclusive id-kp-timeStamping",
        ),
        check(
            "historical_revocation",
            TimestampCheckStatus::Indeterminate,
            "historical validation contract remains open",
        ),
    ];
    Ok(TimestampReport {
        assurance: if bound && signature_valid && trust.is_some_and(|value| value.is_trusted()) {
            TimestampAssurance::Trusted
        } else if bound && signature_valid {
            TimestampAssurance::SignatureValid
        } else if bound {
            TimestampAssurance::Bound
        } else {
            TimestampAssurance::Present
        },
        policy_oid: Some(tst.policy.to_string()),
        checks,
    })
}

fn evaluate_binding(
    algorithm: TimestampCheckStatus,
    actual_imprint: &[u8],
    expected_imprint: &[u8; 32],
    actual_nonce: Option<&[u8]>,
    expected_nonce: Option<&[u8]>,
) -> (TimestampCheckStatus, TimestampCheckStatus) {
    let imprint = if algorithm == TimestampCheckStatus::Pass && actual_imprint == expected_imprint {
        TimestampCheckStatus::Pass
    } else if algorithm == TimestampCheckStatus::Unsupported {
        TimestampCheckStatus::Unsupported
    } else {
        TimestampCheckStatus::Fail
    };
    let nonce = match (expected_nonce, actual_nonce) {
        (Some(expected), Some(actual)) if actual == expected => TimestampCheckStatus::Pass,
        (Some(_), _) => TimestampCheckStatus::Fail,
        (None, _) => TimestampCheckStatus::NotEvaluated,
    };
    (imprint, nonce)
}

fn check(name: &'static str, status: TimestampCheckStatus, detail: &'static str) -> TimestampCheck {
    TimestampCheck {
        name: name.to_owned(),
        status,
        detail: detail.to_owned(),
    }
}

fn write_sidecar_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        Error::InvalidPath("timestamp sidecar has no parent directory".to_owned())
    })?;
    fs::create_dir_all(parent)
        .map_err(|source| Error::io("create timestamp sidecar directory", source))?;
    let temporary: PathBuf = parent.join(format!(
        ".{}.{}.partial",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("timestamp.tsr"),
        uuid::Uuid::new_v4().simple()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| Error::io("create timestamp sidecar staging", source))?;
        file.write_all(bytes)
            .map_err(|source| Error::io("write timestamp sidecar", source))?;
        file.sync_all()
            .map_err(|source| Error::io("sync timestamp sidecar", source))?;
        fs::rename(&temporary, path).map_err(|source| Error::io("commit timestamp sidecar", source))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn compute_plain_root(bundle_root: &Path) -> Result<PlainRootDigest> {
    let manifest = read_bundle_manifest(bundle_root)?;
    let mut entries = vec![("manifest.json".to_owned(), None)];
    entries.extend(manifest.artifacts.iter().map(|artifact| {
        (
            artifact.path.clone(),
            Some((artifact.size_bytes, artifact.digest.value.clone())),
        )
    }));
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(Error::InvalidManifest(
            "duplicate Plain root path".to_owned(),
        ));
    }
    let entry_count = u32::try_from(entries.len())
        .map_err(|_| Error::InvalidManifest("too many Plain root entries".to_owned()))?;
    let mut root = Sha256::new();
    root.update(DOMAIN);
    root.update(entry_count.to_be_bytes());
    for (relative, expected) in entries {
        let path_bytes = relative.as_bytes();
        let path_len = u32::try_from(path_bytes.len())
            .map_err(|_| Error::InvalidPath("Plain root path is too long".to_owned()))?;
        let path = safe_join_existing(bundle_root, Path::new(&relative))?;
        let (size, digest) = compute_sha256(&path)?;
        if let Some((expected_size, expected_digest)) = expected.as_ref()
            && (size != *expected_size || digest != *expected_digest)
        {
            return Err(Error::InvalidManifest(format!(
                "Plain root artifact integrity failed: {relative}"
            )));
        }
        let digest = decode_sha256(&digest)?;
        root.update(path_len.to_be_bytes());
        root.update(path_bytes);
        root.update(size.to_be_bytes());
        root.update(digest);
    }
    let mut value = String::with_capacity(64);
    for byte in root.finalize() {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}")
            .map_err(|error| Error::Execution(format!("format Plain root digest: {error}")))?;
    }
    Ok(PlainRootDigest {
        algorithm: "SHA-256",
        value,
        entry_count,
    })
}

fn decode_sha256(value: &str) -> Result<[u8; 32]> {
    if value.len() != 64 {
        return Err(Error::InvalidManifest(
            "invalid SHA-256 length in Plain root".to_owned(),
        ));
    }
    let mut out = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let text = std::str::from_utf8(pair)
            .map_err(|_| Error::InvalidManifest("invalid SHA-256 encoding".to_owned()))?;
        out[index] = u8::from_str_radix(text, 16)
            .map_err(|_| Error::InvalidManifest("invalid SHA-256 encoding".to_owned()))?;
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::chunks_exact_to_as_chunks, clippy::expect_used)]
mod tests {
    use super::*;

    fn decode_hex(value: &str) -> Vec<u8> {
        value
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16)
                    .expect("valid hex")
            })
            .collect()
    }

    fn signature_fixture() -> Vec<u8> {
        decode_hex(include_str!("../tests/fixtures/rfc3161-rsa-sha256.tsr.hex"))
    }

    fn fixture_imprint() -> [u8; 32] {
        [
            0xf2, 0xb0, 0x4e, 0x5a, 0x87, 0xe6, 0xca, 0x68, 0x40, 0x67, 0x94, 0x71, 0xc7, 0xaa,
            0x09, 0x57, 0xcb, 0xe5, 0xdd, 0xad, 0x1a, 0x9c, 0x64, 0xce, 0xff, 0x94, 0x50, 0xca,
            0x3a, 0x34, 0xfc, 0x98,
        ]
    }

    fn fixture_signer_certificate() -> Result<Vec<u8>> {
        let fixture = signature_fixture();
        let response = TimeStampResp::from_der(&fixture)
            .map_err(|error| Error::Timestamp(format!("decode fixture response: {error}")))?;
        let token = response
            .time_stamp_token
            .ok_or_else(|| Error::Timestamp("fixture token missing".to_owned()))?;
        let signed = SignedData::from_der(
            &token
                .content
                .to_der()
                .map_err(|error| Error::Timestamp(format!("decode fixture CMS: {error}")))?,
        )
        .map_err(|error| Error::Timestamp(format!("decode fixture SignedData: {error}")))?;
        let certificate = signed
            .certificates
            .as_ref()
            .and_then(|values| {
                values.0.iter().find_map(|choice| match choice {
                    cms::cert::CertificateChoices::Certificate(certificate) => Some(certificate),
                    _ => None,
                })
            })
            .ok_or_else(|| Error::Timestamp("fixture signer certificate missing".to_owned()))?;
        certificate
            .to_der()
            .map_err(|error| Error::Timestamp(format!("encode fixture certificate: {error}")))
    }

    #[test]
    fn request_is_sha256_certreq_and_contains_nonce() -> Result<()> {
        let imprint = [0x5a; 32];
        let nonce = [0x11; 16];
        let encoded = encode_request(&imprint, &nonce)?;
        let decoded = TimeStampReq::from_der(&encoded)
            .map_err(|error| Error::Timestamp(format!("decode test request: {error}")))?;
        assert_eq!(decoded.version, TspVersion::V1);
        assert_eq!(decoded.message_imprint.hash_algorithm.oid, SHA256_OID);
        assert_eq!(decoded.message_imprint.hashed_message.as_bytes(), imprint);
        assert_eq!(
            decoded.nonce.as_ref().map(Int::as_bytes),
            Some(nonce.as_slice())
        );
        assert!(decoded.cert_req);
        assert!(decoded.req_policy.is_none());
        Ok(())
    }

    #[test]
    fn tsa_configuration_requires_explicit_https_and_bounded_timeout() {
        assert!(TsaConfig::new("".to_owned(), Duration::from_secs(10)).is_err());
        assert!(TsaConfig::new("http://tsa.invalid".to_owned(), Duration::from_secs(10)).is_err());
        assert!(
            TsaConfig::new(
                "https://user:secret@tsa.invalid".to_owned(),
                Duration::from_secs(10)
            )
            .is_err()
        );
        assert!(
            TsaConfig::new(
                "https://tsa.invalid/?token=secret".to_owned(),
                Duration::from_secs(10)
            )
            .is_err()
        );
        assert!(TsaConfig::new("https://tsa.invalid".to_owned(), Duration::ZERO).is_err());
        assert!(
            TsaConfig::new("https://tsa.invalid".to_owned(), Duration::from_secs(121)).is_err()
        );
        assert!(TsaConfig::new("https://tsa.invalid".to_owned(), Duration::from_secs(10)).is_ok());
        assert!(TsaTrustPolicy::new(vec![], vec![], vec![]).is_err());
        assert!(
            TsaTrustPolicy::new(
                vec![b"not a certificate".to_vec()],
                vec![],
                vec!["1.2.3.4".to_owned()]
            )
            .is_err()
        );
        assert!(
            TsaTrustPolicy::new(
                vec![fixture_signer_certificate().unwrap_or_default()],
                vec![],
                vec!["not-an-oid".to_owned()]
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_and_oversized_responses_fail_closed() {
        assert!(inspect_response(b"not DER", &[0; 32], None, None).is_err());
        let oversized = vec![0; usize::try_from(MAX_TIMESTAMP_RESPONSE_BYTES + 1).unwrap_or(0)];
        assert!(inspect_response(&oversized, &[0; 32], None, None).is_err());
    }

    #[test]
    fn valid_cms_contract_reaches_signature_valid_but_not_trusted() -> Result<()> {
        let report = inspect_response(&signature_fixture(), &fixture_imprint(), None, None)?;
        assert_eq!(report.assurance, TimestampAssurance::SignatureValid);
        for name in [
            "cms_signature",
            "signer_certificate",
            "ess_certificate",
            "signed_attributes",
            "content_type",
            "content_message_digest",
            "signature_algorithm",
        ] {
            assert_eq!(
                report
                    .checks
                    .iter()
                    .find(|check| check.name == name)
                    .map(|check| check.status),
                Some(TimestampCheckStatus::Pass),
                "check {name}"
            );
        }
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "tsa_trust")
                .map(|check| check.status),
            Some(TimestampCheckStatus::NotEvaluated)
        );
        Ok(())
    }

    #[test]
    fn tampered_supported_signature_is_fail_not_unsupported() -> Result<()> {
        let mut response = signature_fixture();
        let last = response
            .last_mut()
            .expect("the timestamp fixture is non-empty");
        *last ^= 1;
        let report = inspect_response(&response, &fixture_imprint(), None, None)?;
        assert_eq!(report.assurance, TimestampAssurance::Bound);
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "cms_signature")
                .map(|check| check.status),
            Some(TimestampCheckStatus::Fail)
        );
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "signature_algorithm")
                .map(|check| check.status),
            Some(TimestampCheckStatus::Pass)
        );
        Ok(())
    }

    #[test]
    fn explicit_anchor_policy_eku_and_gen_time_reach_trusted() -> Result<()> {
        let trust = TsaTrustPolicy::new(
            vec![fixture_signer_certificate()?],
            vec![],
            vec!["1.3.6.1.4.1.55555.1".to_owned()],
        )?;
        let report =
            inspect_response(&signature_fixture(), &fixture_imprint(), None, Some(&trust))?;
        assert_eq!(report.assurance, TimestampAssurance::Trusted);
        for name in ["policy", "timestamping_eku", "tsa_trust"] {
            assert_eq!(
                report
                    .checks
                    .iter()
                    .find(|check| check.name == name)
                    .map(|check| check.status),
                Some(TimestampCheckStatus::Pass),
                "check {name}"
            );
        }
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "historical_revocation")
                .map(|check| check.status),
            Some(TimestampCheckStatus::Indeterminate)
        );
        Ok(())
    }

    #[test]
    fn unaccepted_policy_does_not_reach_trusted() -> Result<()> {
        let trust = TsaTrustPolicy::new(
            vec![fixture_signer_certificate()?],
            vec![],
            vec!["1.3.6.1.4.1.55555.999".to_owned()],
        )?;
        let report =
            inspect_response(&signature_fixture(), &fixture_imprint(), None, Some(&trust))?;
        assert_eq!(report.assurance, TimestampAssurance::SignatureValid);
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "policy")
                .map(|check| check.status),
            Some(TimestampCheckStatus::Fail)
        );
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.name == "tsa_trust")
                .map(|check| check.status),
            Some(TimestampCheckStatus::NotEvaluated)
        );
        Ok(())
    }

    #[test]
    fn binding_distinguishes_failure_unsupported_and_not_evaluated() {
        let imprint = [0x22; 32];
        let nonce = [0x33; 16];
        assert_eq!(
            evaluate_binding(
                TimestampCheckStatus::Pass,
                &imprint,
                &imprint,
                Some(&nonce),
                Some(&nonce),
            ),
            (TimestampCheckStatus::Pass, TimestampCheckStatus::Pass)
        );
        assert_eq!(
            evaluate_binding(
                TimestampCheckStatus::Pass,
                &[0x44; 32],
                &imprint,
                Some(&nonce),
                Some(&[0x55; 16]),
            ),
            (TimestampCheckStatus::Fail, TimestampCheckStatus::Fail)
        );
        assert_eq!(
            evaluate_binding(
                TimestampCheckStatus::Unsupported,
                &imprint,
                &imprint,
                Some(&nonce),
                None,
            ),
            (
                TimestampCheckStatus::Unsupported,
                TimestampCheckStatus::NotEvaluated,
            )
        );
    }
}
