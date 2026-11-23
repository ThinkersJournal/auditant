// SPDX-License-Identifier: MIT OR Apache-2.0

//! The chain core: records, hashing, `verify`, and the in-memory trail. No I/O.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Hash algorithm id stored per record, so the algorithm can change without breaking old trails.
pub const ALG_BLAKE3_V1: &str = "blake3-derive-key-v1";
/// The BLAKE3 `derive_key` context string; it is part of the format.
pub const CONTEXT: &str = "auditant 2026-11 record chain v1";
/// `prev` of the first record.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One stored record. `hash` covers every other field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub seq: u64,
    pub time: String,
    pub schema: String,
    /// Which fingerprint key was in force (DESIGN section 3); covered by `hash`. Absent when none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    pub prev: String,
    pub alg: String,
    pub body: Value,
    pub hash: String,
}

/// What a caller supplies; `seq`, `prev`, `alg` and `hash` are filled in by the trail.
#[derive(Debug, Clone)]
pub struct Event {
    pub time: String,
    pub schema: String,
    /// The fingerprint key id, if any; stored in the record envelope, not the body.
    pub key_id: Option<String>,
    pub body: Value,
}

/// Why a verification or an append failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A record's stored hash is not the hash of its contents (it was edited).
    HashMismatch { seq: u64 },
    /// A record's `prev` is not the previous record's hash (reordered or deleted).
    BrokenLink { seq: u64 },
    /// A record's `seq` is not the expected one.
    BadSeq { seq: u64, expected: u64 },
    /// The record names a hash algorithm this build does not know.
    UnknownAlg { seq: u64, alg: String },
    /// `time` is not UTC RFC 3339 of the form `YYYY-MM-DDTHH:MM:SS[.f+]Z`.
    BadTime { seq: u64 },
    /// `seq` exceeds 2^53 - 1, past which canonical JSON numbers are not exact.
    SeqTooLarge { seq: u64 },
    /// The record could not be canonicalised.
    Canonical(String),
    /// Another writer appended first (the single-writer guard refused this append).
    Conflict { seq: u64 },
    /// Stored data is not what the trail wrote.
    Corrupt(String),
    /// The backing store failed.
    Store(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::HashMismatch { seq } => write!(f, "record {seq}: hash does not match contents"),
            Error::BrokenLink { seq } => write!(
                f,
                "record {seq}: prev does not match the previous record's hash"
            ),
            Error::BadSeq { seq, expected } => write!(f, "record {seq}: expected seq {expected}"),
            Error::UnknownAlg { seq, alg } => write!(f, "record {seq}: unknown alg {alg:?}"),
            Error::BadTime { seq } => write!(f, "record {seq}: time is not UTC RFC 3339 (...Z)"),
            Error::SeqTooLarge { seq } => write!(f, "record {seq}: seq exceeds 2^53 - 1"),
            Error::Canonical(e) => write!(f, "canonicalisation failed: {e}"),
            Error::Conflict { seq } => write!(f, "append of record {seq} lost to another writer"),
            Error::Corrupt(e) => write!(f, "corrupt store: {e}"),
            Error::Store(e) => write!(f, "store error: {e}"),
        }
    }
}
impl std::error::Error for Error {}

/// Largest `seq` that RFC 8785 (IEEE-754 doubles) writes exactly.
const MAX_SEQ: u64 = (1 << 53) - 1;

/// `YYYY-MM-DDTHH:MM:SS[.digits]Z`, ranges checked (no calendar-day check beyond 1..=31).
fn valid_time(t: &str) -> bool {
    let b = t.as_bytes();
    if b.len() < 20 || b[b.len() - 1] != b'Z' {
        return false;
    }
    let d = |i: usize, n: usize| -> Option<u32> {
        b.get(i..i + n)?.iter().try_fold(0u32, |a, c| {
            c.is_ascii_digit().then(|| a * 10 + u32::from(c - b'0'))
        })
    };
    let ok = b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':' && b[16] == b':';
    let frac = &b[19..b.len() - 1];
    let frac_ok = frac.is_empty()
        || (frac[0] == b'.' && frac.len() > 1 && frac[1..].iter().all(u8::is_ascii_digit));
    ok && frac_ok
        && d(0, 4).is_some()
        && matches!(d(5, 2), Some(1..=12))
        && matches!(d(8, 2), Some(1..=31))
        && matches!(d(11, 2), Some(0..=23))
        && matches!(d(14, 2), Some(0..=59))
        && matches!(d(17, 2), Some(0..=60))
}

fn to_hex(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Hash of a record: BLAKE3 `derive_key(CONTEXT, prev_hash_bytes || canonical_json(record without hash))`.
fn compute_hash(r: &Record) -> Result<String, Error> {
    let mut v = serde_json::to_value(r).map_err(|e| Error::Canonical(e.to_string()))?;
    v.as_object_mut()
        .expect("record is an object")
        .remove("hash");
    let canon =
        serde_json_canonicalizer::to_vec(&v).map_err(|e| Error::Canonical(e.to_string()))?;
    let mut input = Vec::with_capacity(r.prev.len() + canon.len());
    input.extend_from_slice(r.prev.as_bytes());
    input.extend_from_slice(&canon);
    Ok(to_hex(&blake3::derive_key(CONTEXT, &input)))
}

/// Build the next record after `head` (`None` for an empty trail).
pub fn next_record(head: Option<&Record>, ev: Event) -> Result<Record, Error> {
    let (seq, prev) = match head {
        Some(h) => (h.seq + 1, h.hash.clone()),
        None => (0, GENESIS.to_string()),
    };
    if seq > MAX_SEQ {
        return Err(Error::SeqTooLarge { seq });
    }
    if !valid_time(&ev.time) {
        return Err(Error::BadTime { seq });
    }
    let mut r = Record {
        seq,
        time: ev.time,
        schema: ev.schema,
        key_id: ev.key_id,
        prev,
        alg: ALG_BLAKE3_V1.into(),
        body: ev.body,
        hash: String::new(),
    };
    r.hash = compute_hash(&r)?;
    Ok(r)
}

pub(crate) fn check_hash(r: &Record) -> Result<(), Error> {
    if compute_hash(r)? == r.hash {
        Ok(())
    } else {
        Err(Error::HashMismatch { seq: r.seq })
    }
}

/// Verify a whole trail from the start. Returns the head hash, or the FIRST broken record and why.
pub fn verify(records: &[Record]) -> Result<Option<String>, Error> {
    let mut prev = GENESIS.to_string();
    for (i, r) in records.iter().enumerate() {
        let expected = i as u64;
        if r.seq != expected {
            return Err(Error::BadSeq {
                seq: r.seq,
                expected,
            });
        }
        if r.seq > MAX_SEQ {
            return Err(Error::SeqTooLarge { seq: r.seq });
        }
        if !valid_time(&r.time) {
            return Err(Error::BadTime { seq: r.seq });
        }
        if r.alg != ALG_BLAKE3_V1 {
            return Err(Error::UnknownAlg {
                seq: r.seq,
                alg: r.alg.clone(),
            });
        }
        if r.prev != prev {
            return Err(Error::BrokenLink { seq: r.seq });
        }
        if compute_hash(r)? != r.hash {
            return Err(Error::HashMismatch { seq: r.seq });
        }
        prev = r.hash.clone();
    }
    Ok(records.last().map(|r| r.hash.clone()))
}

/// An in-memory trail: a test double for the real store (SQLite via PersistAnt, next step).
#[derive(Debug, Default)]
pub struct MemTrail {
    pub records: Vec<Record>,
}

impl MemTrail {
    pub fn emit(&mut self, ev: Event) -> Result<&Record, Error> {
        let r = next_record(self.records.last(), ev)?;
        self.records.push(r);
        Ok(self.records.last().expect("just pushed"))
    }
}
