use crate::{
    CtLog, SignatureValidationError,
    tree::{HashOutput, TreeHead},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use signed_note::{Note, NoteError, Signature};
use thiserror::Error;

impl CtLog {
    pub fn validate_checkpoint(
        &self,
        checkpoint: Checkpoint,
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

        let origin = data.next().ok_or(ParseCheckpointError::MissingField {
            field_name: "origin",
        })?;

        let tree_size = data.next().ok_or(ParseCheckpointError::MissingField {
            field_name: "tree_size",
        })?;
        let tree_size =
            tree_size
                .parse::<u64>()
                .map_err(|_| ParseCheckpointError::MalformedField {
                    field_name: "tree_size",
                })?;

        let root_hash = data
            .next()
            //.map(|root_hash| BASE64_STANDARD.decode(root_hash))
            .ok_or(ParseCheckpointError::MissingField {
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

        todo!()
    }

    pub fn as_string(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Error)]
pub enum ParseCheckpointError {
    #[error("No {field_name} contained in the note")]
    MissingField { field_name: &'static str },

    #[error("{field_name} could not be parsed")]
    MalformedField { field_name: &'static str },
}

// TODO: Test note parsing and validation
