# Changelog

## 0.0.2 - 2026-11-16

- Updates `docs/DESIGN.md` for CireSnave's board-456 rulings (D1 canonical JSON, D2 single-writer guard from the start, D3 BLAKE3, D4 real PersistAnt backend in slice 1): section 4b is the BLAKE3 crate review, sections 5 and 5b the storage needs and what PersistAnt `origin/main` `a2415d7` has and lacks (compare-and-swap and lease are certified on SQL-backed key-value stores only, not plain files), section 11 the decisions. Verified by reading PersistAnt's `matrix/*.tsv` and `crates/persistant-kv/src/lease.rs`, and crates.io, GitHub and RustSec on 2026-11-16; unread claims stay marked UNVERIFIED.
- Adds `ROADMAP.md`: rows for the design's slices with status and blockers, and a table of each README headline claim with the command that backs it (one claim found stale: the README still says version 0.0.0). Verified against crates.io, the branch-protection API and PersistAnt's matrices on 2026-11-16.

## 0.0.1 - 2026-11-16

- Design note (no code): `docs/DESIGN.md`, the first AuditAnt design for review (board 423 ruled A: design first, nothing built). Event model, hash-chained trail and its claims, storage through PersistAnt, API sketch, patent-sweep plan, smallest first slice and four decisions wanted.
