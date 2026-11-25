// SPDX-License-Identifier: MIT OR Apache-2.0

//! Signed checkpoints (DESIGN section 4, slice 3): `sign(key, seq || hash || time)`, kept where the
//! trail's writer cannot reach, then compared against the trail later.
//!
//! Claims (each has a test with a negative control, `tests/checkpoint.rs`): T3, records removed from
//! the END are detected given a checkpoint that covers them; T4, a trail rewritten from some point
//! on is detected given a checkpoint before that point AND that the attacker lacks the signing key.
//! The signing key is the caller's, supplied through [`CheckpointSigner`]; auditant never stores it.

use crate::chain::{Error, Record, valid_time, verify};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Domain separator: part of the signed bytes, so a signature from another use cannot be replayed.
const DOMAIN: &[u8] = b"auditant 2026-11 checkpoint v1\n";

/// Something that can sign checkpoint bytes with an Ed25519 key the caller holds.
pub trait CheckpointSigner {
    /// A 64-byte Ed25519 signature over `msg`.
    fn sign_checkpoint(&self, msg: &[u8]) -> [u8; 64];
}

impl CheckpointSigner for SigningKey {
    fn sign_checkpoint(&self, msg: &[u8]) -> [u8; 64] {
        self.sign(msg).to_bytes()
    }
}

/// A signed statement "the record at `seq` had `hash` at `time`". Copy it somewhere the trail's
/// writer cannot alter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub seq: u64,
    pub hash: String,
    /// UTC RFC 3339 (`...Z`), the signer's clock: claimed, not proven.
    pub time: String,
    /// Ed25519 signature, lowercase hex.
    pub sig: String,
}

fn message(seq: u64, hash: &str, time: &str) -> Vec<u8> {
    let mut m = Vec::with_capacity(DOMAIN.len() + 8 + hash.len() + time.len() + 2);
    m.extend_from_slice(DOMAIN);
    m.extend_from_slice(&seq.to_be_bytes());
    // Length-prefix the variable fields so two different (hash, time) splits never share bytes.
    m.extend_from_slice(&(hash.len() as u32).to_be_bytes());
    m.extend_from_slice(hash.as_bytes());
    m.extend_from_slice(&(time.len() as u32).to_be_bytes());
    m.extend_from_slice(time.as_bytes());
    m
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

impl Checkpoint {
    /// Sign a checkpoint over `record` at the caller's `time`.
    pub fn sign(
        record: &Record,
        time: impl Into<String>,
        signer: &impl CheckpointSigner,
    ) -> Result<Self, Error> {
        let time = time.into();
        if !valid_time(&time) {
            return Err(Error::BadTime { seq: record.seq });
        }
        let sig = signer.sign_checkpoint(&message(record.seq, &record.hash, &time));
        Ok(Checkpoint {
            seq: record.seq,
            hash: record.hash.clone(),
            time,
            sig: hex(&sig),
        })
    }

    /// Check the signature alone, under `key`.
    pub fn check_signature(&self, key: &VerifyingKey) -> Result<(), Error> {
        let bytes: [u8; 64] = unhex(&self.sig)
            .and_then(|b| b.try_into().ok())
            .ok_or(Error::BadSignature)?;
        key.verify(
            &message(self.seq, &self.hash, &self.time),
            &Signature::from_bytes(&bytes),
        )
        .map_err(|_| Error::BadSignature)
    }

    /// Verify `records` as a chain AND against this checkpoint: the signature holds under `key`,
    /// the trail reaches `seq` (else [`Error::Truncated`], T3) and the record there has the signed
    /// hash (else [`Error::CheckpointMismatch`], T4). Returns the trail's head hash.
    pub fn verify_trail(
        &self,
        records: &[Record],
        key: &VerifyingKey,
    ) -> Result<Option<String>, Error> {
        self.check_signature(key)?;
        let head = verify(records)?;
        match records.get(usize::try_from(self.seq).unwrap_or(usize::MAX)) {
            None => Err(Error::Truncated {
                checkpoint_seq: self.seq,
                trail_len: records.len() as u64,
            }),
            Some(r) if r.hash != self.hash => Err(Error::CheckpointMismatch { seq: self.seq }),
            Some(_) => Ok(head),
        }
    }
}
