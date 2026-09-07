use crate::{
    CtLog, LogId, SignatureValidationError, Version,
    signature::Signature as Signed,
    tree::{HashOutput, TreeHead},
    utils::codec::{CodecError, Decode, Encode},
    v1::{SignedTreeHead, sth::TreeHeadSignature},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    io::{Cursor, Read, Write},
};
use thiserror::Error;

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
    ) -> Result<(), SignatureValidationError> {
        // Check that origin line matches the logs submission url
        let origin = self
            .origin_url()
            .ok_or(SignatureValidationError::MalformedKey)?;

        if origin != checkpoint.origin {
            return Err(SignatureValidationError::MalformedKey);
        }

        let note_sig = checkpoint.get_node_signature(self.log_id())?;

        // Parse the key and reconstruct the `TreeHeadSignature`
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

        Ok(())
    }

    pub fn cp_to_sth(
        &self,
        checkpoint: &Checkpoint,
    ) -> Result<SignedTreeHead, SignatureValidationError> {
        let note_sig = checkpoint.get_node_signature(self.log_id())?;

        Ok(SignedTreeHead {
            tree_size: checkpoint.tree_size,
            timestamp: note_sig.timestamp,
            sha256_root_hash: checkpoint.root_hash,
            tree_head_signature: note_sig.signature,
        })
    }

    pub fn sth_to_cp(&self, sth: &SignedTreeHead) -> Result<Checkpoint, SignatureValidationError> {
        let origin = self
            .origin_url()
            .ok_or(SignatureValidationError::MalformedKey)?;

        let id = Checkpoint::compute_checkpoint_key_id(&origin, self.log_id());
        let note_sig = NoteSignature {
            timestamp: sth.timestamp,
            signature: sth.tree_head_signature.clone(),
        };
        let mut body = Cursor::new(vec![]);
        note_sig.encode(&mut body)?;

        Ok(Checkpoint {
            origin: origin.clone(),
            tree_size: sth.tree_size,
            root_hash: sth.sha256_root_hash,
            signatures: vec![Signature {
                name: origin,
                id,
                body: body.into_inner(),
            }],
        })
    }

    fn origin_url(&self) -> Option<String> {
        let url = self.config.url();

        let path = url.path().strip_suffix("/")?;
        url.host_str().map(|host| format!("{}{}", host, path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn tree_size(&self) -> u64 {
        self.tree_size
    }

    pub fn parse(data: &str) -> Result<Self, ParseCheckpointError> {
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

    pub fn to_checkoint_string(&self) -> String {
        let mut output: Vec<Cow<str>> = vec![
            Cow::Borrowed(&self.origin),
            Cow::Owned(self.tree_size.to_string()),
            Cow::Owned(BASE64_STANDARD.encode(self.root_hash)),
            Cow::Borrowed(""),
        ];

        output.extend(
            self.signatures
                .iter()
                .map(|signature| Cow::Owned(signature.to_sig_string())),
        );

        output.join("\n")
    }

    /// Find the signature matching the log
    fn get_node_signature(
        &self,
        log_id: &LogId,
    ) -> Result<NoteSignature, SignatureValidationError> {
        let id = Self::compute_checkpoint_key_id(&self.origin, log_id);
        let sigs = self
            .signatures
            .iter()
            .filter(|sig| sig.name == self.origin)
            .filter(|sig| sig.id == id)
            .collect::<Vec<_>>();

        if sigs.len() != 1 {
            return Err(SignatureValidationError::MalformedSignature);
        }

        let sig = NoteSignature::decode(&mut Cursor::new(&sigs[0].body))?;

        Ok(sig)
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

    fn to_sig_string(&self) -> String {
        let mut data = vec![];
        data.extend_from_slice(&self.id);
        data.extend_from_slice(&self.body);
        let data = BASE64_STANDARD.encode(&data);

        format!("— {} {}", self.name, data)
    }
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
    use crate::{
        tests::{ARGON2025H1_STH2806, get_log_argon2025h1},
        v1::responses::GetSthResponse,
    };

    use super::*;
    use rand::{
        Rng, RngExt, SeedableRng,
        distr::{Alphanumeric, SampleString},
        rngs::ChaCha8Rng,
    };

    const ARCHE2026H1_CHECKPOINT: &str =
        include_str!("../../../testdata/arche2026h1-signed-note.txt");

    const ARCHE2026H1: &str = "
    {
          \"description\": \"Google 'Arche2026h1' log\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZ+3YKoZTMruov4cmlImbk4MckBNzEdCyMuHlwGgJ8BUrzFLlR5U0619xDDXIXespkpBgCNVQAkhMTTXakM6KMg==\",
          \"url\": \"https://arche2026h1.staging.ct.transparency.dev/\",
          \"tile_url\": \"https://storage.googleapis.com/static-ct-staging-arche2026h1-bucket/\",
          \"mmd\": 60
        }
    ";

    const SYCAMORE2026H1_CHECKPOINT: &str =
        include_str!("../../../testdata/sycamore2026h1-signed-note.txt");

    const SYCAMORE2026H1: &str = "{
            \"description\": \"Let's Encrypt 'Sycamore2026h1'\",
            \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEfEEe0JZknA91/c6eNl1aexgeKzuGQUMvRCXPXg9L227O5I4Pi++Abcpq6qxlVUKPYafAJelAnMfGzv3lHCc8gA==\",
            \"url\": \"https://log.sycamore.ct.letsencrypt.org/2026h1/\",
            \"tile_url\": \"https://mon.sycamore.ct.letsencrypt.org/2026h1/\",
            \"mmd\": 60
        }
    ";

    #[test]
    fn signature_serialization_roundtrip() {
        let mut rng = ChaCha8Rng::seed_from_u64(6767);

        for _ in 0..1000 {
            let sig = random_signature(&mut rng);

            let sig_str = sig.to_sig_string();
            let new_sig = Signature::from_str(&sig_str).unwrap();

            assert_eq!(sig, new_sig);
        }
    }

    #[test]
    fn checkpoint_serialization_roundtrip() {
        let mut rng = ChaCha8Rng::seed_from_u64(6767);

        for _ in 0..1000 {
            let cp = Checkpoint {
                origin: Alphanumeric.sample_string(&mut rng, 16),
                tree_size: rng.random(),
                root_hash: rng.random(),
                signatures: std::iter::repeat_with(|| random_signature(&mut rng))
                    .take(10)
                    .collect(),
            };

            let cp_string = cp.to_checkoint_string();
            let new_cp = Checkpoint::parse(&cp_string).unwrap();

            assert_eq!(cp, new_cp);
        }
    }

    #[test]
    fn parse_and_validate_checkpoint_arche2026h1() {
        let checkpoint = Checkpoint::parse(ARCHE2026H1_CHECKPOINT).unwrap();

        assert_eq!(checkpoint.origin, "arche2026h1.staging.ct.transparency.dev");
        assert_eq!(checkpoint.tree_size, 1822167730);

        let config = serde_json::from_str(ARCHE2026H1).unwrap();
        let log = CtLog::new(config);

        log.validate_checkpoint(&checkpoint).unwrap();
    }

    #[test]
    fn parse_and_validate_checkpoint_sycamore2026h1() {
        let checkpoint = Checkpoint::parse(SYCAMORE2026H1_CHECKPOINT).unwrap();

        assert_eq!(checkpoint.origin, "log.sycamore.ct.letsencrypt.org/2026h1");
        assert_eq!(checkpoint.tree_size, 804475391);

        let config = serde_json::from_str(SYCAMORE2026H1).unwrap();
        let log = CtLog::new(config);

        log.validate_checkpoint(&checkpoint).unwrap();
    }

    fn random_signature(rng: &mut impl Rng) -> Signature {
        let mut body = vec![0u8; 100];
        rng.fill(&mut body);

        Signature {
            name: Alphanumeric.sample_string(rng, 16),
            id: rng.random(),
            body,
        }
    }

    #[test]
    fn sth_to_checkpoint_roundtrip() {
        let log = get_log_argon2025h1();
        let sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        let sth = SignedTreeHead::try_from(sth).unwrap();

        let cp = log.sth_to_cp(&sth).unwrap();
        let new_sth = log.cp_to_sth(&cp).unwrap();

        assert_eq!(sth, new_sth);
    }
}
