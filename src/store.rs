// SPDX-License-Identifier: MIT OR Apache-2.0

//! A trail stored through PersistAnt's kv (SQLite via the registry).
//!
//! Layout: `rec/<seq:020>` holds a record, written with `put_if_absent`; `head` holds `{seq, hash}`
//! and is advanced with ONE function, [`advance_head`], by compare-and-swap on the raw bytes.
//! (PersistAnt's typed `update_record` and fencing token are unreleased; swap in at ROADMAP row 11.)
//! A crash between the record write and the head advance leaves an orphan record that
//! [`SqliteTrail::open`] adopts after re-checking its link and hash.

use crate::chain::{Error, Event, Record, check_hash, next_record, verify};
use persistant::blocking::registry::{self, KvHandle};
use persistant::capability::{CapabilityId, choose::Durability, provider::ProviderNeeds};
use persistant::registry::Endpoint;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The chain head: the last record's `seq` and `hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Head {
    pub seq: u64,
    pub hash: String,
}

const HEAD_KEY: &str = "head";

fn rec_key(seq: u64) -> String {
    format!("rec/{seq:020}")
}

fn store_err(e: impl std::fmt::Display) -> Error {
    Error::Store(e.to_string())
}

/// A trail on a PersistAnt kv backend.
pub struct SqliteTrail {
    kv: KvHandle,
}

impl std::fmt::Debug for SqliteTrail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteTrail").finish_non_exhaustive()
    }
}

impl SqliteTrail {
    /// Open (creating if needed) a SQLite-backed trail at `path`, adopting an orphan record if a
    /// crash left one.
    pub fn open(path: &Path) -> Result<Self, Error> {
        let id = |s: &str| CapabilityId::new(s).expect("literal capability id");
        let needs = ProviderNeeds::new(
            vec![id("kv.get"), id("kv.put_if_absent"), id("kv.compare_and_swap")],
            Durability::Process,
        );
        let kv = registry::open_named("sqlite", &Endpoint::path(path), &needs).map_err(store_err)?;
        let t = SqliteTrail { kv };
        t.recover()?;
        Ok(t)
    }

    /// The current head, or `None` for an empty trail.
    pub fn head(&self) -> Result<Option<Head>, Error> {
        match self.kv.get(HEAD_KEY).map_err(store_err)? {
            None => Ok(None),
            Some(b) => serde_json::from_slice(&b).map(Some).map_err(|e| Error::Corrupt(e.to_string())),
        }
    }

    fn record(&self, seq: u64) -> Result<Option<Record>, Error> {
        match self.kv.get(&rec_key(seq)).map_err(store_err)? {
            None => Ok(None),
            Some(b) => serde_json::from_slice(&b).map(Some).map_err(|e| Error::Corrupt(e.to_string())),
        }
    }

    /// Append an event after the current head.
    pub fn emit(&self, ev: Event) -> Result<Record, Error> {
        let head = self.head()?;
        self.emit_after(head, ev)
    }

    /// Append an event believing the head is `head`. Fails with [`Error::Conflict`] if another
    /// writer got there first: this is the single-writer guard (record `put_if_absent`, then the
    /// head compare-and-swap).
    pub fn emit_after(&self, head: Option<Head>, ev: Event) -> Result<Record, Error> {
        let prev_rec = match &head {
            Some(h) => Some(self.record(h.seq)?.ok_or_else(|| Error::Corrupt(format!("head names missing record {}", h.seq)))?),
            None => None,
        };
        let r = next_record(prev_rec.as_ref(), ev)?;
        let bytes = serde_json::to_vec(&r).map_err(store_err)?;
        if !self.kv.put_if_absent(&rec_key(r.seq), bytes, None).map_err(store_err)? {
            return Err(Error::Conflict { seq: r.seq });
        }
        self.advance_head(head.as_ref(), &Head { seq: r.seq, hash: r.hash.clone() })?;
        Ok(r)
    }

    /// The ONE place the head moves (ROADMAP row 11 swaps its body for PersistAnt's typed helper).
    fn advance_head(&self, expected: Option<&Head>, new: &Head) -> Result<(), Error> {
        let new_bytes = serde_json::to_vec(new).map_err(store_err)?;
        let ok = match expected {
            None => self.kv.put_if_absent(HEAD_KEY, new_bytes, None).map_err(store_err)?,
            Some(h) => {
                let old = serde_json::to_vec(h).map_err(store_err)?;
                self.kv.compare_and_swap(HEAD_KEY, &old, new_bytes, None).map_err(store_err)?
            }
        };
        if ok { Ok(()) } else { Err(Error::Conflict { seq: new.seq }) }
    }

    /// Adopt an orphan record (written but never made head) after checking it links to the head.
    fn recover(&self) -> Result<(), Error> {
        let head = self.head()?;
        let next = head.as_ref().map_or(0, |h| h.seq + 1);
        let Some(orphan) = self.record(next)? else { return Ok(()) };
        let mut chain = match &head {
            Some(h) => vec![self.record(h.seq)?.ok_or_else(|| Error::Corrupt("head names a missing record".into()))?],
            None => vec![],
        };
        // Check just the link and the hash of the orphan, not the whole trail.
        let anchor = chain.pop();
        let expected_prev = anchor.as_ref().map_or(crate::chain::GENESIS.to_string(), |r| r.hash.clone());
        if orphan.prev != expected_prev || orphan.seq != next {
            return Err(Error::Corrupt(format!("orphan record {next} does not link to the head")));
        }
        check_hash(&orphan)?;
        self.advance_head(head.as_ref(), &Head { seq: orphan.seq, hash: orphan.hash })
    }

    /// Read every record in order and verify the whole trail; returns the head hash.
    pub fn verify_all(&self) -> Result<Option<String>, Error> {
        let mut records = Vec::new();
        let mut seq = 0;
        while let Some(r) = self.record(seq)? {
            records.push(r);
            seq += 1;
        }
        verify(&records)
    }

    /// Test and diagnostic access: write a raw value (used by the negative-control tests).
    #[doc(hidden)]
    pub fn raw(&self) -> &KvHandle {
        &self.kv
    }
}
