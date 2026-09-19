use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

macro_rules! logical_id {
    ($name:ident, $prefix:literal, $kind:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}{}", $prefix, Uuid::new_v4().simple()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self> {
                let suffix = value
                    .strip_prefix($prefix)
                    .ok_or_else(|| Error::InvalidId {
                        kind: $kind,
                        value: value.to_owned(),
                    })?;
                if suffix.len() != 32 || !suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(Error::InvalidId {
                        kind: $kind,
                        value: value.to_owned(),
                    });
                }
                Ok(Self(value.to_owned()))
            }
        }
    };
}

logical_id!(EngagementId, "eng_", "engagement");
logical_id!(ExecutionId, "exe_", "execution");
logical_id!(ArtifactId, "art_", "artifact");
