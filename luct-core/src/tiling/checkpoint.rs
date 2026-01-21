use std::io::{Cursor, Read, Write};

use crate::{
    CtLog, LogId, SignatureValidationError, Version,
    signature::Signature as Signed,
    tree::{HashOutput, TreeHead},
    utils::codec::{CodecError, Decode, Encode},
    v1::{SignedTreeHead, sth::TreeHeadSignature},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseCheckpointError {
    #[error("No {field_name} contained in the note")]
    MissingField { field_name: &'static str },

    #[error("{field_name} could not be parsed")]
    MalformedField { field_name: &'static str },

    #[error("Unexpected extensions appended to the note. We only expect notes with 3 fields")]
    UnexpectedExtensions,

    #[error("The note contains no signatures.")]
    NoSignatures,

    #[error("The signature at index {index} is malformed")]
    MalformedSignature { index: usize },
}

impl CtLog {
    pub fn validate_checkpoint(
        &self,
        checkpoint: &Checkpoint,
    ) -> Result<SignedTreeHead, SignatureValidationError> {
        // Check that origin line matches the logs submission url
        let origin = Self::url_to_origin(self.config().url())
            .ok_or(SignatureValidationError::MalformedKey)?;
        if origin != checkpoint.origin {
            return Err(SignatureValidationError::MalformedKey);
        }

        // Find exactly one matching key in the list of keys
        // TODO: Precompute id once during initialization, rather than recomputer it here all the time
        let id = Self::compute_checkpoint_key_id(&origin, self.log_id());
        let sigs = checkpoint
            .signatures
            .iter()
            .filter(|sig| sig.name == checkpoint.origin)
            .filter(|sig| sig.id == id)
            .collect::<Vec<_>>();
        if sigs.len() != 1 {
            return Err(SignatureValidationError::MalformedSignature);
        }
        let sig = sigs[0];

        // Parse the key and reconstruct the `TreeHeadSignature`
        let note_sig = NoteSignature::decode(&mut Cursor::new(&sig.body))?;
        let tree_head = TreeHeadSignature {
            version: Version::V1,
            timestamp: note_sig.timestamp,
            tree_size: checkpoint.tree_size,
            sha256_root_hash: checkpoint.root_hash,
        };

        // Validate the signature
        note_sig
            .signature
            .validate(&tree_head, &self.config().key)?;

        Ok(SignedTreeHead {
            tree_size: checkpoint.tree_size,
            timestamp: note_sig.timestamp,
            sha256_root_hash: checkpoint.root_hash.to_vec(),
            tree_head_signature: note_sig.signature,
        })
    }

    fn compute_checkpoint_key_id(origin: &str, log_id: &LogId) -> [u8; 4] {
        let mut hash = Sha256::new();
        hash.update(origin);
        hash.update([0x0A, 0x05]);

        match log_id {
            LogId::V1(log_id) => hash.update(log_id.0),
        }

        let hash: [u8; 32] = hash.finalize().into();
        let id: [u8; 4] = hash[0..4].try_into().unwrap();

        id
    }

    fn url_to_origin(url: &Url) -> Option<String> {
        let path = match url.path() {
            "/" => "",
            other => other,
        };

        url.host_str().map(|host| format!("{}{}", host, path))
    }
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    origin: String,
    tree_size: u64,
    root_hash: HashOutput,
    signatures: Vec<Signature>,
}

impl From<Checkpoint> for TreeHead {
    fn from(checkpoint: Checkpoint) -> Self {
        TreeHead {
            tree_size: checkpoint.tree_size,
            head: checkpoint.root_hash,
        }
    }
}

impl Checkpoint {
    pub fn parse_checkpoint(data: &str) -> Result<Self, ParseCheckpointError> {
        let mut data = data.lines();

        // Parse the origin
        let origin = data
            .next()
            .ok_or(ParseCheckpointError::MissingField {
                field_name: "origin",
            })?
            .to_string();

        // Parse the tree size
        let tree_size = data.next().ok_or(ParseCheckpointError::MissingField {
            field_name: "tree_size",
        })?;
        let tree_size =
            tree_size
                .parse::<u64>()
                .map_err(|_| ParseCheckpointError::MalformedField {
                    field_name: "tree_size",
                })?;

        // Parse the root hash
        let root_hash = data.next().ok_or(ParseCheckpointError::MissingField {
            field_name: "root_hash",
        })?;
        let root_hash = BASE64_STANDARD.decode(root_hash).map_err(|_| {
            ParseCheckpointError::MalformedField {
                field_name: "root_hash",
            }
        })?;
        let root_hash: HashOutput =
            root_hash
                .try_into()
                .map_err(|_| ParseCheckpointError::MalformedField {
                    field_name: "root_hash",
                })?;

        // Check that there is an empty line
        let separator = data.next().ok_or(ParseCheckpointError::NoSignatures)?;
        if !separator.is_empty() {
            return Err(ParseCheckpointError::UnexpectedExtensions);
        }

        // Parse the signatures
        let signatures = data
            .enumerate()
            .map(|(index, signature)| {
                Signature::from_str(signature)
                    .ok_or(ParseCheckpointError::MalformedSignature { index })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if signatures.is_empty() {
            return Err(ParseCheckpointError::NoSignatures);
        }

        Ok(Self {
            origin,
            tree_size,
            root_hash,
            signatures,
        })
    }

    // TODO: `as_string` function and roundtrip test
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Signature {
    name: String,
    id: [u8; 4],
    body: Vec<u8>,
}

impl Signature {
    fn from_str(data: &str) -> Option<Self> {
        let mut data = data.strip_prefix("— ")?.split(" ");
        let name = data.next()?.to_string();

        let mut data = BASE64_STANDARD.decode(data.next()?).ok()?;
        if data.len() < 4 {
            return None;
        }

        let body = data.split_off(4);
        let id: [u8; 4] = data.try_into().unwrap();

        Some(Self { name, id, body })
    }

    // TODO: `as_string` function
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NoteSignature {
    timestamp: u64,
    signature: Signed<TreeHeadSignature>,
}

impl Encode for NoteSignature {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.timestamp.encode(&mut writer)?;
        self.signature.encode(&mut writer)?;

        Ok(())
    }
}

impl Decode for NoteSignature {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            timestamp: u64::decode(&mut reader)?,
            signature: Signed::decode(&mut reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARCHE2026H1_CHECKPOINT: &str =
        include_str!("../../../testdata/arche2026h1-signed-note.txt");

    const ARCHE2026H1: &str = "
    {
          \"description\": \"Google 'Arche2026h1' log\",
          \"log_id\": \"J+sqNJffaHpkC2Q4TkhW/Nyj6H+NzWbzTtbxvkKB7fw=\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZ+3YKoZTMruov4cmlImbk4MckBNzEdCyMuHlwGgJ8BUrzFLlR5U0619xDDXIXespkpBgCNVQAkhMTTXakM6KMg==\",
          \"url\": \"https://arche2026h1.staging.ct.transparency.dev/\",
          \"tile_url\": \"https://storage.googleapis.com/static-ct-staging-arche2026h1-bucket/\",
          \"mmd\": 60
        }
    ";

    #[test]
    fn parse_and_validate_checkpoint() {
        let checkpoint = Checkpoint::parse_checkpoint(ARCHE2026H1_CHECKPOINT).unwrap();

        assert_eq!(checkpoint.origin, "arche2026h1.staging.ct.transparency.dev");
        assert_eq!(checkpoint.tree_size, 1822167730);

        let config = serde_json::from_str(ARCHE2026H1).unwrap();
        let log = CtLog::new(config);

        log.validate_checkpoint(&checkpoint).unwrap();
    }
}
