# Changelog

## 0.1.2 - 2026-11-25 (allocated by pm-assist; segments and export, rows 13/15/18/19)

- Added: segments and export (row 15): `Segment`, `segments`, `verify_segment`, JSONL `to_jsonl`/`from_jsonl`, `SqliteTrail::export_segment`; a segment verifies alone and chains to the one before it. No pruning. `verify` now delegates to `verify_segment`. Lock only: `cargo update` (10 packages, SUP-015).
- Added: signed checkpoints (`Checkpoint`, `CheckpointSigner`; Ed25519 via `ed25519-dalek` 3.0.0): `verify_trail` detects records removed from the end (T3) and a trail rewritten from a checkpointed point (T4, while the attacker lacks the key). `Error` gains `BadSignature`, `Truncated`, `CheckpointMismatch` (Breaking for exhaustive matches, pre-release). `SigningKey`/`VerifyingKey` re-exported. CI `portfolio.yml` pin moved to coderipper dc45e36.

## 0.1.1 - 2026-11-24 (allocated by pm-assist; postgres and mysql backends)

- Added: `SqliteTrail::open_backend(id, endpoint, namespace)` opens a trail on any PersistAnt kv backend; cargo features `postgres` and `mysql` enable the server backends (PersistAnt's matrix covers postgres 16.15, mysql 8.4.11); `Endpoint` is re-exported. A namespace prefixes keys so trails can share a database (default none: existing keys unchanged). CI runs the store checks against both servers.

## 0.1.0 (breaking format polish; allocated by pm-assist 2026-11-23)

- **Breaking (record format, pre-release, no stored trail yet):** `key_id` is now a hashed record field (`Record::key_id`, `Event::key_id`), no longer inside the body (`typed_event` sets it from the context); `Event` gains a field. `next_record` and `verify` now refuse a `time` that is not UTC RFC 3339 (`...Z`; `Error::BadTime`) and a `seq` above 2^53 - 1, where canonical JSON numbers stop being exact (`Error::SeqTooLarge`). A `None` key_id is omitted from the stored record. Board-464 one-hash rule unchanged (guard test passes).
- Lock only: `url` 2.5.9 (pm-ants TASK).

## 0.0.5 - 2026-11-21

- Added: `typed_event`, building an `Event` from a `#[derive(Redact)]` value (redactant 0.1.14): the body holds each field's disposition (redacted, fingerprint tag, or kept value), never a withheld value; the derive's schema string becomes the record's `schema`. Allocated by pm-assist.

## 0.0.4 - 2026-11-20

- Added: slice 1 of the audit trail: one-hash BLAKE3 chain over RFC 8785 canonical JSON (`Record`, `next_record`, `verify`, `MemTrail`) and a SQLite-backed `SqliteTrail` on PersistAnt with a single-writer head compare-and-swap and crash recovery (Linux only; its tests have not yet run).
- Docs: PATENT-SWEEP.md records the AuditAnt patent sweep (one STOP item sent to the PM; no code until answered).

## 0.0.3 - 2026-11-19

- Docs: ROADMAP row 11 records that PersistAnt's typed CAS and fencing token landed on its main (unreleased).

## 0.0.2 - 2026-11-16

- Updates `docs/DESIGN.md` for CireSnave's board-456 rulings (D1 canonical JSON, D2 single-writer guard from the start, D3 BLAKE3, D4 real PersistAnt backend in slice 1): section 4b is the BLAKE3 crate review, sections 5 and 5b the storage needs and what PersistAnt `origin/main` `a2415d7` has and lacks (compare-and-swap and lease are certified on SQL-backed key-value stores only, not plain files), section 11 the decisions. Verified by reading PersistAnt's `matrix/*.tsv` and `crates/persistant-kv/src/lease.rs`, and crates.io, GitHub and RustSec on 2026-11-16; unread claims stay marked UNVERIFIED.
- Adds `ROADMAP.md`: rows for the design's slices with status and blockers, and a table of each README headline claim with the command that backs it (one claim found stale: the README still says version 0.0.0). Verified against crates.io, the branch-protection API and PersistAnt's matrices on 2026-11-16.

## 0.0.1 - 2026-11-16

- Design note (no code): `docs/DESIGN.md`, the first AuditAnt design for review (board 423 ruled A: design first, nothing built). Event model, hash-chained trail and its claims, storage through PersistAnt, API sketch, patent-sweep plan, smallest first slice and four decisions wanted.
