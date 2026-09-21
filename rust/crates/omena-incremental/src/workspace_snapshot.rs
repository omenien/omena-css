use serde::{Deserialize, Serialize};

use crate::OmenaWorkspaceSnapshotIdV0;

/// A revision is portable only together with the workspace and fixed-input
/// commitment that its owner issued. Decoding this claim does not validate the
/// receiver's actual inputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceSnapshotBindingV0 {
    workspace_root: String,
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
    input_commitment: OmenaWorkspaceInputCommitmentV0,
}

impl OmenaWorkspaceSnapshotBindingV0 {
    pub fn new(
        workspace_root: impl Into<String>,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        input_commitment: OmenaWorkspaceInputCommitmentV0,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            snapshot_id,
            input_commitment,
        }
    }

    pub fn workspace_root(&self) -> &str {
        &self.workspace_root
    }

    pub fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0 {
        self.snapshot_id
    }

    pub fn input_commitment(&self) -> &OmenaWorkspaceInputCommitmentV0 {
        &self.input_commitment
    }
}

/// An integrity commitment is not a canonical module or document identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct OmenaWorkspaceInputCommitmentV0(String);

impl OmenaWorkspaceInputCommitmentV0 {
    pub fn from_sha256(bytes: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(64);
        for byte in bytes {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 15)]));
        }
        Self(encoded)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for OmenaWorkspaceInputCommitmentV0 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(serde::de::Error::custom(
                "workspace input commitment must be a lowercase SHA-256 digest",
            ));
        }
        Ok(Self(value))
    }
}
