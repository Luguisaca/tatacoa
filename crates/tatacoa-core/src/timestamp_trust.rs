use crate::timestamp::{TimestampCheckStatus, TsaTrustPolicy};
use cms::cert::CertificateChoices;
use cms::signed_data::SignedData;
use der::Encode;
use rustls::pki_types::{CertificateDer, SignatureVerificationAlgorithm, UnixTime};
use x509_cert::Certificate;
use x509_cert::ext::pkix::ExtendedKeyUsage;
use x509_tsp::TstInfo;

const TIMESTAMPING_EKU_OID: der::asn1::ObjectIdentifier =
    der::asn1::ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.3.8");
const TIMESTAMPING_EKU_VALUE: &[u8] = &[0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x08];
const ALPHA_PATH_ALGORITHMS: &[&dyn SignatureVerificationAlgorithm] = &[
    webpki::ring::RSA_PKCS1_2048_8192_SHA256,
    webpki::ring::RSA_PKCS1_2048_8192_SHA384,
    webpki::ring::RSA_PKCS1_2048_8192_SHA512,
    webpki::ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    webpki::ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    webpki::ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    webpki::ring::ECDSA_P256_SHA256,
    webpki::ring::ECDSA_P384_SHA384,
];

pub(crate) struct TrustEvaluation {
    pub policy: TimestampCheckStatus,
    pub eku: TimestampCheckStatus,
    pub path: TimestampCheckStatus,
}

impl TrustEvaluation {
    pub(crate) fn is_trusted(&self) -> bool {
        self.policy == TimestampCheckStatus::Pass
            && self.eku == TimestampCheckStatus::Pass
            && self.path == TimestampCheckStatus::Pass
    }
}

pub(crate) fn evaluate_trust(
    signed: &SignedData,
    tst: &TstInfo,
    policy: &TsaTrustPolicy,
    signature_valid: bool,
) -> TrustEvaluation {
    if !signature_valid {
        return TrustEvaluation {
            policy: TimestampCheckStatus::NotEvaluated,
            eku: TimestampCheckStatus::NotEvaluated,
            path: TimestampCheckStatus::NotEvaluated,
        };
    }
    let policy_status = if policy.accepted_policy_oids.contains(&tst.policy) {
        TimestampCheckStatus::Pass
    } else {
        TimestampCheckStatus::Fail
    };
    let Some(signer) = signed.signer_infos.0.iter().next() else {
        return not_evaluated(policy_status);
    };
    let Some(certificate) = crate::timestamp_signature::find_signer_certificate(signed, signer)
    else {
        return not_evaluated(policy_status);
    };
    let eku_status = verify_timestamping_eku(certificate);
    let path_status = if policy_status == TimestampCheckStatus::Pass
        && eku_status == TimestampCheckStatus::Pass
    {
        verify_path_at_gen_time(signed, certificate, tst, policy)
    } else {
        TimestampCheckStatus::NotEvaluated
    };
    TrustEvaluation {
        policy: policy_status,
        eku: eku_status,
        path: path_status,
    }
}

fn verify_timestamping_eku(certificate: &Certificate) -> TimestampCheckStatus {
    certificate
        .tbs_certificate
        .get::<ExtendedKeyUsage>()
        .ok()
        .flatten()
        .map_or(TimestampCheckStatus::Fail, |(critical, usages)| {
            if critical && usages.0.as_slice() == [TIMESTAMPING_EKU_OID] {
                TimestampCheckStatus::Pass
            } else {
                TimestampCheckStatus::Fail
            }
        })
}

fn verify_path_at_gen_time(
    signed: &SignedData,
    signer: &Certificate,
    tst: &TstInfo,
    policy: &TsaTrustPolicy,
) -> TimestampCheckStatus {
    let anchors_der: Vec<_> = policy
        .trust_anchors
        .iter()
        .map(|value| CertificateDer::from(value.as_slice()))
        .collect();
    let Ok(anchors) = anchors_der
        .iter()
        .map(webpki::anchor_from_trusted_cert)
        .collect::<Result<Vec<_>, _>>()
    else {
        return TimestampCheckStatus::Fail;
    };
    let Ok(signer_der) = signer.to_der() else {
        return TimestampCheckStatus::Fail;
    };
    let mut intermediate_bytes = policy.intermediates.clone();
    if let Some(certificates) = signed.certificates.as_ref() {
        for choice in certificates.0.iter() {
            if let CertificateChoices::Certificate(certificate) = choice
                && let Ok(encoded) = certificate.to_der()
                && encoded != signer_der
                && !policy.trust_anchors.iter().any(|anchor| anchor == &encoded)
            {
                intermediate_bytes.push(encoded);
            }
        }
    }
    let intermediates: Vec<_> = intermediate_bytes
        .iter()
        .map(|value| CertificateDer::from(value.as_slice()))
        .collect();
    let signer_der = CertificateDer::from(signer_der.as_slice());
    let Ok(end_entity) = webpki::EndEntityCert::try_from(&signer_der) else {
        return TimestampCheckStatus::Fail;
    };
    let time = UnixTime::since_unix_epoch(tst.gen_time.to_unix_duration());
    let usage = webpki::KeyUsage::required(TIMESTAMPING_EKU_VALUE);
    match end_entity.verify_for_usage(
        ALPHA_PATH_ALGORITHMS,
        &anchors,
        &intermediates,
        time,
        usage,
        None,
        None,
    ) {
        Ok(_) => TimestampCheckStatus::Pass,
        Err(
            webpki::Error::UnsupportedSignatureAlgorithmContext(_)
            | webpki::Error::UnsupportedSignatureAlgorithmForPublicKeyContext(_),
        ) => TimestampCheckStatus::Unsupported,
        Err(_) => TimestampCheckStatus::Fail,
    }
}

fn not_evaluated(policy: TimestampCheckStatus) -> TrustEvaluation {
    TrustEvaluation {
        policy,
        eku: TimestampCheckStatus::NotEvaluated,
        path: TimestampCheckStatus::NotEvaluated,
    }
}
