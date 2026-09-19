use crate::{Error, Result, SecurityProfile};

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
