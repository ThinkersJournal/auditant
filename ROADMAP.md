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
| 9 | **Slice 1:** typed event + RedactAnt derive, canonical JSON (JCS), BLAKE3 chain with `alg` per record, `emit`/`verify`, SQLite via PersistAnt `choose`, single-writer guard (head compare-and-swap behind one function `advance_head`, plus lease), tests T1/T2/T5 and a two-writer test each with a negative control, README hello compiled in CI | **in progress** on `lane-work`: core chain, JCS, BLAKE3 one-hash, `MemTrail` (9 tests pass on Windows); README hello compiled as a doctest (runs in CI `cargo test`); `SqliteTrail` via PersistAnt 0.18.30 (5 tests pass on Windows 2026-11-19, fresh lockfile); derive, T5 and two-writer on Linux outstanding | RedactAnt derive (row 10); body is `serde_json::Value` meanwhile; raw-byte kv CAS meanwhile |
| 10 | RedactAnt field-level derive and serde-capable `Kept` (DESIGN 8, Q2, Q6) | **done on `lane-row10`** (not yet in a PR): redactant 0.1.14 published 2026-11-20 with derive and serde `Keep`; `typed_event` (`src/typed.rs`) stores dispositions, tests in `tests/typed.rs` (canaries never stored, kept value stored, flipped disposition breaks the chain, no key fails closed) | none |
| 11 | Swap `advance_head` to the typed helper; adopt fencing token | **unblocked; assessed 2026-11-19, swap NOT clearly worth it**: `update_record` is async, wraps the value in its own envelope (kind, schema) and retries a lost race by re-running `f` on the latest value, whereas the head guard must FAIL on a stale expected head; our raw `compare_and_swap` already does that, so use `update_record` only if `f` can refuse (return Err) when the live head is not the expected one, and that changes the stored head format. The fencing token (`Lease::token`) is the real gain: decide with the first multi-process user | persistant 0.18.28 published 2026-11-19, 0.18.30 (fresh-lockfile fix) taken; CHANGELOG: `update_record` in 0.18.26, `Lease::token`/`lock.fencing` in 0.18.27; `lane-work` pins 0.18.30 |
| 12 | Other backends (postgres, mysql) in the same tests; plain files when PersistAnt can CAS on blob | todo | row 9; PersistAnt |
| 13 | Slice 3: signed checkpoints, T3/T4 (Ed25519) | todo | row 9 |
| 14 | Slice 4: dynamic (map-fed) events | todo | row 3 says whether a customer needs it |
| 15 | Slice 5: segment rotation, export | todo | row 9 |
| 16 | First publish (0.1.0 pre-release) | todo | PM publishes; rows 6 to 9 done; fresh-eyes trial of the published crate |
| 17 | Drop the `RUSTSEC-2026-0543` (chrono) ignore in `deny.toml` | todo, **expires 2026-12-17** | sqlx/persistant-sqlx must stop pulling chrono; renew with a reason if not |
| 18 | Portfolio sweep (advisory, red on #4 at coderipper c72a6e0): ARC-002 "3 components depend on each other in a cycle: `api`, `chain`, `store`"; ARC-003/006/007 did not run (no volatile, api_budget, owns_data tables) and the job fails on a rule that did not run | tables added on `lane-row10`, **cycle cause found 2026-11-20 (coderipper false positive):** the one `pub(crate) fn check_hash` in `src/chain.rs` is read as a reference to the crate root (edge chain->api); with `pub fn` ARC-001 and ARC-002 are clean (probe, local coderipper 0.47.10; coderipper reads files at HEAD). Minimal repro and finding sent to pm-ants for the coderipper lane; no waiver. `super::` paths made no difference. Keep `pub(crate)` unless told to use a `#[doc(hidden)] pub` workaround; read the next sweep for ARC-003/006/007 **Read 2026-11-21 (PR #5 sweep, coderipper c72a6e0, job green):** ARC-003/006/007 now run (no "did not run"; ARC-007 owned names: head (store)). Still reported: ARC-001 Low "`chain` depends on `api`" and ARC-002 Medium cycle api/chain/store, the same false positive (ARC-002 read 4 components, 4 edges, 1 cycle); SUP-016 Info (LICENSE-MIT names a person as holder, not looked at yet). Row stays open until coderipper's fix release (pm-ants will announce), then rerun. | none  **Rerun 2026-11-22 on published coderipper 0.47.18 (cargo install --locked, lane-slice2 b024c54, `sweep`): ARC-001 no finding, ARC-002 `0 cycle(s)` (false positive fixed, `pub(crate) fn check_hash` kept); ARC-003/006/007 each run clean by `check` (their note says module groups are not judged, only whole crates, so clean is weak). Sweep exit 3 is only WSP-011 (protection read 404 needs admin, repo path ThinkersJournal/auditant). Still open: SUP-015 (10 lock drifts, fresh-lock only), SUP-016 Info, SUP-002 chrono RUSTSEC-2026-0543 (deny.toml ignore, row 17).** |

## Known gaps (not rows)
- A tamper-evident trail does not prevent tampering, nor detect tail truncation without an externally held checkpoint (DESIGN 4, "not claimed").
- Plain-file trails are impossible until PersistAnt can compare-and-swap on blob.
- The lease depends on the backend's TTL clock; safety rests on the head compare-and-swap.
- persistant 0.18.28 failed on fresh lockfiles (matrix keyed to bundled SQLite 3.51.3, 0.40.0 bundles 3.54.1); fixed in 0.18.30, taken 2026-11-19 with no pin. `tests/store.rs` passes on Windows on a fresh lockfile; Linux CI is still to run it. No `Lease` use here, so the 0.18.30 lease changes (10 s hold after a death between claim and token write; zero ttl refused) do not apply until row 11 adopts fencing.
- `SqliteTrail::emit` still takes an untyped `Event`; callers build it with `typed_event`. No dynamic (map-fed) events (row 14); `key_id` is inside the body, not a record field.

— lane: auditant
