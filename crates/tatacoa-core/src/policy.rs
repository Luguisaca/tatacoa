use crate::{Error, ExportMode, Result, SecurityProfile};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlainExportAuthorization {
    pub acknowledged_plaintext: bool,
}

pub fn authorize_plain_export(
    profile: SecurityProfile,
    authorization: PlainExportAuthorization,
) -> Result<()> {
    match profile {
        SecurityProfile::LabLearning => Ok(()),
        SecurityProfile::Professional if authorization.acknowledged_plaintext => Ok(()),
        SecurityProfile::Professional => Err(Error::Conflict(
            "PROFESSIONAL requires explicit acknowledgement for a PLAIN export".to_owned(),
        )),
        SecurityProfile::HighSensitivity => Err(Error::Conflict(
            "HIGH_SENSITIVITY forbids PLAIN export; encrypted export is not implemented".to_owned(),
        )),
        SecurityProfile::Custom => Err(Error::Conflict(
            "CUSTOM has no approved export policy; PLAIN export is denied".to_owned(),
        )),
    }
}

pub fn default_export_mode(profile: SecurityProfile) -> Result<ExportMode> {
    match profile {
        SecurityProfile::LabLearning => Ok(ExportMode::Plain),
        SecurityProfile::Professional | SecurityProfile::HighSensitivity => {
            Ok(ExportMode::Encrypted)
        }
        SecurityProfile::Custom => Err(Error::Conflict(
            "CUSTOM has no approved export policy; export is denied".to_owned(),
        )),
    }
}

pub fn authorize_encrypted_export(profile: SecurityProfile) -> Result<()> {
    match profile {
        SecurityProfile::LabLearning
        | SecurityProfile::Professional
        | SecurityProfile::HighSensitivity => Ok(()),
        SecurityProfile::Custom => Err(Error::Conflict(
            "CUSTOM has no approved export policy; ENCRYPTED export is denied".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_do_not_downgrade_protected_profiles() {
        assert_eq!(
            default_export_mode(SecurityProfile::LabLearning).ok(),
            Some(ExportMode::Plain)
        );
        assert_eq!(
            default_export_mode(SecurityProfile::Professional).ok(),
            Some(ExportMode::Encrypted)
        );
        assert_eq!(
            default_export_mode(SecurityProfile::HighSensitivity).ok(),
            Some(ExportMode::Encrypted)
        );
        assert!(default_export_mode(SecurityProfile::Custom).is_err());
    }

    #[test]
    fn direct_policy_calls_cannot_bypass_profile_rules() {
        assert!(
            authorize_plain_export(
                SecurityProfile::Professional,
                PlainExportAuthorization::default()
            )
            .is_err()
        );
        assert!(
            authorize_plain_export(
                SecurityProfile::HighSensitivity,
                PlainExportAuthorization {
                    acknowledged_plaintext: true,
                }
            )
            .is_err()
        );
        assert!(authorize_encrypted_export(SecurityProfile::Custom).is_err());
        assert!(authorize_encrypted_export(SecurityProfile::HighSensitivity).is_ok());
    }
}
