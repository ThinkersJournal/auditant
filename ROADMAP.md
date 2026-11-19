# auditant ROADMAP

Written 2026-11-16 by the `auditant` lane on pm-ants's [TASK], at `origin/main` `16cb8fc`. Nothing is built; `docs/DESIGN.md` is the plan.
Rulings: board 423 A (design first), board 456 (CireSnave 2026-11-16, verbatim): *"456: D1: A. D2: B (PersistAnt should include locking
mechanisms anyway). D3: B. D4: B."* Open issues: 0; PRs: #1 (design note) merged 2026-11-16. This is PR #2, a draft until after 00:00Z 2026-11-17 (one PR per project per day). Versions are allocated by the PM.

## README headline claims and what backs each

| Claim (README) | Backed by | Result 2026-11-16 |
|---|---|---|
| "the crate name `auditant` is reserved" | `curl https://crates.io/api/v1/crates/auditant` (control: a made-up name returns "does not exist") | 0.0.0 exists, created 2026-10-22 |
| "version 0.0.0 ... `publish = false`" (old text) | `Cargo.toml` | was stale (gate stamped 0.0.1); **fixed in this PR**: README now says 0.0.1, `publish = false`; check: `grep -n "^version\|^publish" Cargo.toml` |
| "nothing here works yet" | no code beyond `src/lib.rs` stub | true; no test needed |
| licences MIT or Apache-2.0 | `LICENSE-MIT`, `LICENSE-APACHE` present; `cargo deny` job in CI | files present; licence text copied from RedactAnt not re-diffed (**UNVERIFIED**) |

There is no headline feature claim yet, so no command to list. Every claim the README gains in slice 1 gets its command or test in this table first.

## Rows

| # | What | Status | Blocker |
|---|---|---|---|
| 1 | Design note `docs/DESIGN.md` with CireSnave's rulings | **done** (#1) | none |
| 2 | Branch protection requiring CI | done: 7 required checks, enforcement `non_admins` (read from `branches/main` protection, not `.protected`) | none |
| 3 | Replace UNVERIFIED customer rows (auth-framework, SecretAnt needs) by reading their `origin/main` | **done in this PR** (DESIGN section 2; finding: SecretAnt's local-file stub cannot move to a plain-file trail) | none |
| 4 | Crate reviews: JCS and Ed25519 read on crates.io (DESIGN 4c); RFC 8785 test vectors against the chosen crate and the `deny.toml` check of BSD-3-Clause still owed | partly done in this PR | none |
| 5 | Three questions to the PersistAnt lane (DESIGN 5b: does `choose` list `kv-on-relational`, is `lock.lease` reachable through `open_named`, CAS on blob) | todo | none |
| 6 | Patent sweep (DESIGN section 9), recorded in `PATENT-SWEEP.md`, **before slice-1 code ships** | **done** on `lane-work` (22 patents read; one in-force EP, Palantir EP2897051B1, design-around by keeping one hash per entry) | board 464 ruled A (2026-11-19): single hash, never a second level; first publish not held on this |
| 7 | README first screen: plain-file trails impossible, no FIPS claim, BLAKE3 facts | **done in this PR** (placeholder text; extend with slice 1) | none |
| 8 | `[architecture]` declaration in `.coderipper.toml` (ruling 365) | **written, unverified** (first CI run checks it); core split into `src/chain.rs` | none |
| 9 | **Slice 1:** typed event + RedactAnt derive, canonical JSON (JCS), BLAKE3 chain with `alg` per record, `emit`/`verify`, SQLite via PersistAnt `choose`, single-writer guard (head compare-and-swap behind one function `advance_head`, plus lease), tests T1/T2/T5 and a two-writer test each with a negative control, README hello compiled in CI | **in progress** on `lane-work`: core chain, JCS, BLAKE3 one-hash, `MemTrail` (9 tests pass on Windows); README hello compiled as a doctest (runs in CI `cargo test`); `SqliteTrail` via PersistAnt 0.18.28 (5 tests pass on Windows 2026-11-19 with the sqlite pin); derive, T5 and two-writer on Linux outstanding | RedactAnt derive (row 10); body is `serde_json::Value` meanwhile; raw-byte kv CAS meanwhile |
| 10 | RedactAnt field-level derive and serde-capable `Kept` (DESIGN 8, Q2, Q6) | RedactAnt's ROADMAP row 6; slice 1 of it is not built (its DERIVE.md "nothing built") | **Owner: redactant lane.** The 4 asks of DESIGN 8 sent as a [TASK] request 2026-11-16 (cc pm-ants); reply wanted by 2026-11-18, no date committed by them yet |
| 11 | Swap `advance_head` to the typed helper; adopt fencing token | **unblocked, not started** | persistant 0.18.28 published 2026-11-19 (pm-ants/PM); CHANGELOG: `update_record` in 0.18.26, `Lease::token`/`lock.fencing` in 0.18.27; `lane-work` pins 0.18.28 |
| 12 | Other backends (postgres, mysql) in the same tests; plain files when PersistAnt can CAS on blob | todo | row 9; PersistAnt |
| 13 | Slice 3: signed checkpoints, T3/T4 (Ed25519) | todo | row 9 |
| 14 | Slice 4: dynamic (map-fed) events | todo | row 3 says whether a customer needs it |
| 15 | Slice 5: segment rotation, export | todo | row 9 |
| 16 | First publish (0.1.0 pre-release) | todo | PM publishes; rows 6 to 9 done; fresh-eyes trial of the published crate |

## Known gaps (not rows)
- A tamper-evident trail does not prevent tampering, nor detect tail truncation without an externally held checkpoint (DESIGN 4, "not claimed").
- Plain-file trails are impossible until PersistAnt can compare-and-swap on blob.
- The lease depends on the backend's TTL clock; safety rests on the head compare-and-swap.
- `libsqlite3-sys` is pinned `=0.37.0` because persistant 0.18.28's matrix is keyed to bundled SQLite 3.51.3 (0.40.0 bundles 3.54.1: every lookup is "no row"); remove the pin when the persistant fix release is out. With it, `tests/store.rs` passes on Windows (2026-11-19).
- Slice 1 has no seq/time/key_id fields yet and no derive; the record body is an untyped JSON value.

— lane: auditant
