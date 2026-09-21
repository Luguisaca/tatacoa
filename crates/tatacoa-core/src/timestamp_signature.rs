use crate::timestamp::TimestampCheckStatus;
use cms::cert::CertificateChoices;
use cms::signed_data::{SignedData, SignerIdentifier, SignerInfo};
use der::asn1::{ObjectIdentifier, OctetString};
use der::{Decode, Encode, Sequence};
use pkcs1::RsaPssParams;
use ring::digest;
use rustls::pki_types::{CertificateDer, SignatureVerificationAlgorithm};
use x509_cert::Certificate;
use x509_cert::attr::{Attribute, Attributes};
use x509_cert::ext::pkix::SubjectKeyIdentifier;
use x509_cert::ext::pkix::certpolicy::PolicyInformation;
use x509_cert::ext::pkix::name::{GeneralName, GeneralNames};
use x509_cert::serial_number::SerialNumber;
use x509_cert::spki::AlgorithmIdentifierOwned;

const SHA1_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.14.3.2.26");
const SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
const SHA384_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
const SHA512_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");
const RSA_SHA1_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.5");
const RSA_ENCRYPTION_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");
const RSA_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
const RSA_SHA384_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.12");
const RSA_SHA512_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.13");
const RSA_PSS_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.10");
const MGF1_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.8");
const ECDSA_SHA1_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.1");
const ECDSA_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");
const ECDSA_SHA384_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.3");
const ED25519_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.101.112");
const DSA_SHA1_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10040.4.3");
const DSA_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.3.2");
const CONTENT_TYPE_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.3");
const MESSAGE_DIGEST_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.4");
const SIGNING_CERTIFICATE_OID: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.2.12");
const SIGNING_CERTIFICATE_V2_OID: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.2.47");

#[derive(Debug)]
pub(crate) struct SignatureEvaluation {
    pub algorithm: TimestampCheckStatus,
    pub signed_attributes: TimestampCheckStatus,
    pub content_type: TimestampCheckStatus,
    pub message_digest: TimestampCheckStatus,
    pub signer_certificate: TimestampCheckStatus,
    pub ess_certificate: TimestampCheckStatus,
    pub signature: TimestampCheckStatus,
    pub algorithm_detail: String,
}

impl SignatureEvaluation {
    pub(crate) fn is_valid(&self) -> bool {
        [
            self.algorithm,
            self.signed_attributes,
            self.content_type,
            self.message_digest,
            self.signer_certificate,
            self.ess_certificate,
            self.signature,
        ]
        .into_iter()
        .all(|status| status == TimestampCheckStatus::Pass)
    }
}

#[derive(Clone, Copy)]
enum DigestKind {
    Sha256,
    Sha384,
    Sha512,
}

impl DigestKind {
    fn from_oid(oid: ObjectIdentifier) -> Option<Self> {
        match oid {
            SHA256_OID => Some(Self::Sha256),
            SHA384_OID => Some(Self::Sha384),
            SHA512_OID => Some(Self::Sha512),
            _ => None,
        }
    }

    fn oid(self) -> ObjectIdentifier {
        match self {
            Self::Sha256 => SHA256_OID,
            Self::Sha384 => SHA384_OID,
            Self::Sha512 => SHA512_OID,
        }
    }

    fn output_len(self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    fn digest(self, bytes: &[u8]) -> Vec<u8> {
        let algorithm = match self {
            Self::Sha256 => &digest::SHA256,
            Self::Sha384 => &digest::SHA384,
            Self::Sha512 => &digest::SHA512,
        };
        digest::digest(algorithm, bytes).as_ref().to_vec()
    }
}

pub(crate) fn evaluate_signature(
    signed: &SignedData,
    content: &[u8],
    content_type: ObjectIdentifier,
) -> SignatureEvaluation {
    let Some(signer) = exactly_one(signed.signer_infos.0.iter()) else {
        return failed("ambiguous SignerInfo set");
    };
    let Some(digest_kind) = DigestKind::from_oid(signer.digest_alg.oid) else {
        let status = if signer.digest_alg.oid == SHA1_OID {
            TimestampCheckStatus::Fail
        } else {
            TimestampCheckStatus::Unsupported
        };
        return algorithm_only(status, "signer digest is rejected or unsupported");
    };
    if !signed
        .digest_algorithms
        .iter()
        .any(|algorithm| algorithm.oid == signer.digest_alg.oid)
    {
        return failed("SignerInfo digest is absent from SignedData.digestAlgorithms");
    }
    let Some(attributes) = signer.signed_attrs.as_ref() else {
        return failed("signedAttrs are mandatory for id-ct-TSTInfo");
    };
    let content_type_status = verify_content_type(attributes, content_type);
    let message_digest_status = verify_message_digest(attributes, digest_kind, content);
    let Some(certificate) = find_signer_certificate(signed, signer) else {
        return SignatureEvaluation {
            algorithm: TimestampCheckStatus::NotEvaluated,
            signed_attributes: TimestampCheckStatus::Pass,
            content_type: content_type_status,
            message_digest: message_digest_status,
            signer_certificate: TimestampCheckStatus::Fail,
            ess_certificate: TimestampCheckStatus::NotEvaluated,
            signature: TimestampCheckStatus::NotEvaluated,
            algorithm_detail: "signer certificate is missing or ambiguous".to_owned(),
        };
    };
    let ess_status = verify_ess_certificate(attributes, certificate);
    let (algorithm_status, algorithm, detail) = select_algorithm(signer, digest_kind);
    let signature_status = if content_type_status == TimestampCheckStatus::Pass
        && message_digest_status == TimestampCheckStatus::Pass
        && ess_status == TimestampCheckStatus::Pass
        && algorithm_status == TimestampCheckStatus::Pass
    {
        verify_cryptographic_signature(certificate, signer, attributes, algorithm)
    } else {
        TimestampCheckStatus::NotEvaluated
    };
    SignatureEvaluation {
        algorithm: algorithm_status,
        signed_attributes: TimestampCheckStatus::Pass,
        content_type: content_type_status,
        message_digest: message_digest_status,
        signer_certificate: TimestampCheckStatus::Pass,
        ess_certificate: ess_status,
        signature: signature_status,
        algorithm_detail: detail,
    }
}

fn verify_content_type(
    attributes: &Attributes,
    expected: ObjectIdentifier,
) -> TimestampCheckStatus {
    let Some(attribute) = unique_attribute(attributes, CONTENT_TYPE_OID) else {
        return TimestampCheckStatus::Fail;
    };
    let Some(value) = exactly_one(attribute.values.iter()) else {
        return TimestampCheckStatus::Fail;
    };
    value
        .to_der()
        .ok()
        .and_then(|der| ObjectIdentifier::from_der(&der).ok())
        .map_or(TimestampCheckStatus::Fail, |actual| {
            if actual == expected {
                TimestampCheckStatus::Pass
            } else {
                TimestampCheckStatus::Fail
            }
        })
}

fn verify_message_digest(
    attributes: &Attributes,
    algorithm: DigestKind,
    content: &[u8],
) -> TimestampCheckStatus {
    let Some(attribute) = unique_attribute(attributes, MESSAGE_DIGEST_OID) else {
        return TimestampCheckStatus::Fail;
    };
    let Some(value) = exactly_one(attribute.values.iter()) else {
        return TimestampCheckStatus::Fail;
    };
    value
        .to_der()
        .ok()
        .and_then(|der| OctetString::from_der(&der).ok())
        .map_or(TimestampCheckStatus::Fail, |actual| {
            if actual.as_bytes() == algorithm.digest(content) {
                TimestampCheckStatus::Pass
            } else {
                TimestampCheckStatus::Fail
            }
        })
}

pub(crate) fn find_signer_certificate<'a>(
    signed: &'a SignedData,
    signer: &SignerInfo,
) -> Option<&'a Certificate> {
    let certificates = signed.certificates.as_ref()?;
    let mut matching = certificates.0.iter().filter_map(|choice| {
        let CertificateChoices::Certificate(certificate) = choice else {
            return None;
        };
        let matches = match &signer.sid {
            SignerIdentifier::IssuerAndSerialNumber(id) => {
                certificate.tbs_certificate.issuer == id.issuer
                    && certificate.tbs_certificate.serial_number == id.serial_number
            }
            SignerIdentifier::SubjectKeyIdentifier(id) => certificate
                .tbs_certificate
                .get::<SubjectKeyIdentifier>()
                .ok()
                .flatten()
                .is_some_and(|(_, actual)| actual.0 == id.0),
        };
        matches.then_some(certificate)
    });
    let first = matching.next()?;
    matching.next().is_none().then_some(first)
}

fn verify_ess_certificate(
    attributes: &Attributes,
    certificate: &Certificate,
) -> TimestampCheckStatus {
    if unique_attribute(attributes, SIGNING_CERTIFICATE_OID).is_some() {
        return TimestampCheckStatus::Fail;
    }
    let Some(attribute) = unique_attribute(attributes, SIGNING_CERTIFICATE_V2_OID) else {
        return TimestampCheckStatus::Fail;
    };
    let Some(value) = exactly_one(attribute.values.iter()) else {
        return TimestampCheckStatus::Fail;
    };
    let Ok(encoded) = value.to_der() else {
        return TimestampCheckStatus::Fail;
    };
    let Ok(signing) = SigningCertificateV2::from_der(&encoded) else {
        return TimestampCheckStatus::Fail;
    };
    let Some(identifier) = signing.certs.first() else {
        return TimestampCheckStatus::Fail;
    };
    let algorithm = identifier
        .hash_algorithm
        .as_ref()
        .map(|value| value.oid)
        .unwrap_or(SHA256_OID);
    let Some(algorithm) = DigestKind::from_oid(algorithm) else {
        return TimestampCheckStatus::Unsupported;
    };
    let Ok(certificate_der) = certificate.to_der() else {
        return TimestampCheckStatus::Fail;
    };
    let issuer_serial_matches = identifier.issuer_serial.as_ref().is_none_or(|expected| {
        expected.serial_number == certificate.tbs_certificate.serial_number
            && expected.issuer.iter().any(|name| {
                matches!(name, GeneralName::DirectoryName(actual) if actual == &certificate.tbs_certificate.issuer)
            })
    });
    if issuer_serial_matches
        && identifier.cert_hash.as_bytes() == algorithm.digest(&certificate_der)
    {
        TimestampCheckStatus::Pass
    } else {
        TimestampCheckStatus::Fail
    }
}

fn select_algorithm(
    signer: &SignerInfo,
    digest: DigestKind,
) -> (
    TimestampCheckStatus,
    Option<&'static dyn SignatureVerificationAlgorithm>,
    String,
) {
    select_algorithm_identifier(&signer.signature_algorithm, digest)
}

fn select_algorithm_identifier(
    identifier: &AlgorithmIdentifierOwned,
    digest: DigestKind,
) -> (
    TimestampCheckStatus,
    Option<&'static dyn SignatureVerificationAlgorithm>,
    String,
) {
    let oid = identifier.oid;
    let selected = match (oid, digest) {
        (RSA_ENCRYPTION_OID, DigestKind::Sha256) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA256),
        (RSA_ENCRYPTION_OID, DigestKind::Sha384) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA384),
        (RSA_ENCRYPTION_OID, DigestKind::Sha512) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA512),
        (RSA_SHA256_OID, DigestKind::Sha256) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA256),
        (RSA_SHA384_OID, DigestKind::Sha384) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA384),
        (RSA_SHA512_OID, DigestKind::Sha512) => Some(webpki::ring::RSA_PKCS1_2048_8192_SHA512),
        (ECDSA_SHA256_OID, DigestKind::Sha256) => Some(webpki::ring::ECDSA_P256_SHA256),
        (ECDSA_SHA384_OID, DigestKind::Sha384) => Some(webpki::ring::ECDSA_P384_SHA384),
        (RSA_PSS_OID, _) => return select_pss(identifier, digest),
        (RSA_SHA1_OID | ECDSA_SHA1_OID, _) => {
            return (
                TimestampCheckStatus::Fail,
                None,
                "SHA-1 is rejected".to_owned(),
            );
        }
        (ED25519_OID | DSA_SHA1_OID | DSA_SHA256_OID, _) => {
            return (
                TimestampCheckStatus::Unsupported,
                None,
                "signature algorithm is outside Alpha".to_owned(),
            );
        }
        (RSA_SHA256_OID | RSA_SHA384_OID | RSA_SHA512_OID, _)
        | (ECDSA_SHA256_OID | ECDSA_SHA384_OID, _) => {
            return (
                TimestampCheckStatus::Fail,
                None,
                "signature and digest algorithms are inconsistent".to_owned(),
            );
        }
        _ => None,
    };
    if matches!(
        oid,
        RSA_ENCRYPTION_OID | RSA_SHA256_OID | RSA_SHA384_OID | RSA_SHA512_OID
    ) && identifier
        .parameters
        .as_ref()
        .is_some_and(|parameters| !parameters.is_null())
    {
        return (
            TimestampCheckStatus::Fail,
            None,
            "RSA PKCS#1 v1.5 parameters must be NULL or absent".to_owned(),
        );
    }
    if matches!(oid, ECDSA_SHA256_OID | ECDSA_SHA384_OID) && identifier.parameters.is_some() {
        return (
            TimestampCheckStatus::Fail,
            None,
            "ECDSA signature parameters must be absent".to_owned(),
        );
    }
    selected.map_or_else(
        || {
            (
                TimestampCheckStatus::Unsupported,
                None,
                "signature/digest combination is unsupported".to_owned(),
            )
        },
        |algorithm| {
            (
                TimestampCheckStatus::Pass,
                Some(algorithm),
                format!("accepted signature algorithm {oid}"),
            )
        },
    )
}

fn select_pss(
    identifier: &AlgorithmIdentifierOwned,
    digest: DigestKind,
) -> (
    TimestampCheckStatus,
    Option<&'static dyn SignatureVerificationAlgorithm>,
    String,
) {
    let Some(parameters) = identifier.parameters.as_ref() else {
        return (
            TimestampCheckStatus::Fail,
            None,
            "RSA-PSS parameters are required".to_owned(),
        );
    };
    let Ok(encoded) = parameters.to_der() else {
        return (
            TimestampCheckStatus::Fail,
            None,
            "invalid RSA-PSS parameters".to_owned(),
        );
    };
    let Ok(parameters) = RsaPssParams::from_der(&encoded) else {
        return (
            TimestampCheckStatus::Fail,
            None,
            "invalid RSA-PSS parameters".to_owned(),
        );
    };
    if parameters.hash.oid != digest.oid()
        || parameters.mask_gen.oid != MGF1_OID
        || parameters
            .mask_gen
            .parameters
            .is_none_or(|mgf| mgf.oid != digest.oid())
        || usize::from(parameters.salt_len) != digest.output_len()
    {
        return (
            TimestampCheckStatus::Fail,
            None,
            "RSA-PSS parameters do not match the approved profile".to_owned(),
        );
    }
    let algorithm = match digest {
        DigestKind::Sha256 => webpki::ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        DigestKind::Sha384 => webpki::ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
        DigestKind::Sha512 => webpki::ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    };
    (
        TimestampCheckStatus::Pass,
        Some(algorithm),
        "accepted RSA-PSS parameters".to_owned(),
    )
}

fn verify_cryptographic_signature(
    certificate: &Certificate,
    signer: &SignerInfo,
    attributes: &Attributes,
    algorithm: Option<&'static dyn SignatureVerificationAlgorithm>,
) -> TimestampCheckStatus {
    let Some(algorithm) = algorithm else {
        return TimestampCheckStatus::NotEvaluated;
    };
    let (Ok(certificate_der), Ok(signed_bytes)) = (certificate.to_der(), attributes.to_der())
    else {
        return TimestampCheckStatus::Fail;
    };
    let certificate_der = CertificateDer::from(certificate_der);
    let Ok(end_entity) = webpki::EndEntityCert::try_from(&certificate_der) else {
        return TimestampCheckStatus::Fail;
    };
    if end_entity
        .verify_signature(algorithm, &signed_bytes, signer.signature.as_bytes())
        .is_ok()
    {
        TimestampCheckStatus::Pass
    } else {
        TimestampCheckStatus::Fail
    }
}

fn unique_attribute(attributes: &Attributes, oid: ObjectIdentifier) -> Option<&Attribute> {
    let mut matching = attributes.iter().filter(|attribute| attribute.oid == oid);
    let first = matching.next()?;
    matching.next().is_none().then_some(first)
}

fn exactly_one<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let first = values.next()?;
    values.next().is_none().then_some(first)
}

fn failed(detail: &str) -> SignatureEvaluation {
    SignatureEvaluation {
        algorithm: TimestampCheckStatus::NotEvaluated,
        signed_attributes: TimestampCheckStatus::Fail,
        content_type: TimestampCheckStatus::NotEvaluated,
        message_digest: TimestampCheckStatus::NotEvaluated,
        signer_certificate: TimestampCheckStatus::NotEvaluated,
        ess_certificate: TimestampCheckStatus::NotEvaluated,
        signature: TimestampCheckStatus::NotEvaluated,
        algorithm_detail: detail.to_owned(),
    }
}

fn algorithm_only(status: TimestampCheckStatus, detail: &str) -> SignatureEvaluation {
    SignatureEvaluation {
        algorithm: status,
        signed_attributes: TimestampCheckStatus::NotEvaluated,
        content_type: TimestampCheckStatus::NotEvaluated,
        message_digest: TimestampCheckStatus::NotEvaluated,
        signer_certificate: TimestampCheckStatus::NotEvaluated,
        ess_certificate: TimestampCheckStatus::NotEvaluated,
        signature: TimestampCheckStatus::NotEvaluated,
        algorithm_detail: detail.to_owned(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
struct SigningCertificateV2 {
    certs: Vec<EssCertIdV2>,
    #[asn1(optional = "true")]
    policies: Option<Vec<PolicyInformation>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
struct EssCertIdV2 {
    #[asn1(optional = "true")]
    hash_algorithm: Option<AlgorithmIdentifierOwned>,
    cert_hash: OctetString,
    #[asn1(optional = "true")]
    issuer_serial: Option<EssIssuerSerial>,
}

#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
struct EssIssuerSerial {
    issuer: GeneralNames,
    serial_number: SerialNumber,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use der::asn1::{Any, Null, SetOfVec};

    fn identifier(oid: ObjectIdentifier, parameters: Option<Any>) -> AlgorithmIdentifierOwned {
        AlgorithmIdentifierOwned { oid, parameters }
    }

    fn attribute(oid: ObjectIdentifier, value: Any) -> Attribute {
        Attribute {
            oid,
            values: SetOfVec::try_from(vec![value]).expect("valid singleton attribute"),
        }
    }

    fn attributes(values: Vec<Attribute>) -> Attributes {
        SetOfVec::try_from(values).expect("valid attributes")
    }

    #[test]
    fn accepted_signature_algorithms_match_the_alpha_allowlist() {
        let cases = [
            (RSA_SHA256_OID, DigestKind::Sha256),
            (RSA_SHA384_OID, DigestKind::Sha384),
            (RSA_SHA512_OID, DigestKind::Sha512),
            (RSA_ENCRYPTION_OID, DigestKind::Sha256),
            (RSA_ENCRYPTION_OID, DigestKind::Sha384),
            (RSA_ENCRYPTION_OID, DigestKind::Sha512),
            (ECDSA_SHA256_OID, DigestKind::Sha256),
            (ECDSA_SHA384_OID, DigestKind::Sha384),
        ];
        for (oid, digest) in cases {
            let (status, algorithm, _) =
                select_algorithm_identifier(&identifier(oid, None), digest);
            assert_eq!(status, TimestampCheckStatus::Pass);
            assert!(algorithm.is_some());
        }
    }

    #[test]
    fn rejected_and_unsupported_algorithms_are_distinct() {
        let (sha1, _, _) =
            select_algorithm_identifier(&identifier(RSA_SHA1_OID, None), DigestKind::Sha256);
        let (ed25519, _, _) =
            select_algorithm_identifier(&identifier(ED25519_OID, None), DigestKind::Sha256);
        let (dsa, _, _) =
            select_algorithm_identifier(&identifier(DSA_SHA256_OID, None), DigestKind::Sha256);
        let (unknown, _, _) = select_algorithm_identifier(
            &identifier(ObjectIdentifier::new_unwrap("1.2.3.4.5"), None),
            DigestKind::Sha256,
        );
        assert_eq!(sha1, TimestampCheckStatus::Fail);
        assert_eq!(ed25519, TimestampCheckStatus::Unsupported);
        assert_eq!(dsa, TimestampCheckStatus::Unsupported);
        assert_eq!(unknown, TimestampCheckStatus::Unsupported);
    }

    #[test]
    fn malformed_supported_algorithm_identifiers_fail_closed() {
        let mismatch = identifier(RSA_SHA384_OID, None);
        let (status, _, _) = select_algorithm_identifier(&mismatch, DigestKind::Sha256);
        assert_eq!(status, TimestampCheckStatus::Fail);

        let ecdsa_with_null = identifier(
            ECDSA_SHA256_OID,
            Some(Any::encode_from(&Null).expect("NULL encodes")),
        );
        let (status, _, _) = select_algorithm_identifier(&ecdsa_with_null, DigestKind::Sha256);
        assert_eq!(status, TimestampCheckStatus::Fail);

        let rsa_with_oid = identifier(
            RSA_SHA256_OID,
            Some(Any::encode_from(&SHA256_OID).expect("OID encodes")),
        );
        let (status, _, _) = select_algorithm_identifier(&rsa_with_oid, DigestKind::Sha256);
        assert_eq!(status, TimestampCheckStatus::Fail);
    }

    #[test]
    fn content_type_and_message_digest_require_unique_exact_values() {
        let content = b"authenticated TSTInfo";
        let content_type = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.1.4");
        let good_type = attribute(
            CONTENT_TYPE_OID,
            Any::encode_from(&content_type).expect("OID encodes"),
        );
        let good_digest = attribute(
            MESSAGE_DIGEST_OID,
            Any::encode_from(
                &OctetString::new(DigestKind::Sha256.digest(content)).expect("digest fits"),
            )
            .expect("octet string encodes"),
        );
        let good = attributes(vec![good_type.clone(), good_digest.clone()]);
        assert_eq!(
            verify_content_type(&good, content_type),
            TimestampCheckStatus::Pass
        );
        assert_eq!(
            verify_message_digest(&good, DigestKind::Sha256, content),
            TimestampCheckStatus::Pass
        );

        let conflicting_type = attribute(
            CONTENT_TYPE_OID,
            Any::encode_from(&ObjectIdentifier::new_unwrap("1.2.3.4")).expect("OID encodes"),
        );
        let duplicate = attributes(vec![good_type, conflicting_type, good_digest]);
        assert_eq!(
            verify_content_type(&duplicate, content_type),
            TimestampCheckStatus::Fail
        );
        assert_eq!(
            verify_message_digest(&good, DigestKind::Sha256, b"tampered"),
            TimestampCheckStatus::Fail
        );
    }
}
