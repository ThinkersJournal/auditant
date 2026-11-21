// SPDX-License-Identifier: MIT OR Apache-2.0

//! auditant: structured events and an append-only, tamper-evident (hash-chained) trail.
//!
//! Slice 1, core only (ROADMAP row 9): one hash per record. Each record's hash covers the previous
//! record's hash and the record's canonical (RFC 8785) JSON bytes. There is deliberately NO second
//! "summary hash" level (PATENT-SWEEP.md, design-around of EP2897051 claim 1).
//!
//! Claims: edits, reorderings and mid-trail deletions are detected by [`verify`]. Not claimed:
//! prevention, tail-truncation detection without an externally held head, authenticity, time,
//! confidentiality, multi-writer safety (see `docs/DESIGN.md` section 4).

// The README's example is compiled and run as a doctest, so it cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

mod chain;
mod store;
mod typed;

pub use chain::{
    ALG_BLAKE3_V1, CONTEXT, Error, Event, GENESIS, MemTrail, Record, next_record, verify,
};
pub use store::{Head, SqliteTrail};
pub use typed::typed_event;
