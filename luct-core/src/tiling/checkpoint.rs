use crate::{
    CtLog, SignatureValidationError,
    tree::{HashOutput, TreeHead},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd, Error)]
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
    ) -> Result<(), SignatureValidationError> {
        todo!()
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
