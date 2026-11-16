# AuditAnt design note

Status: **draft for CireSnave's review, nothing built** (board 423 ruled A, 2026-11-12: design first, no code before he has
seen this). Written 2026-11-16 by the `auditant` lane. Facts about other projects were read from their `origin/main` on
2026-11-16 (RedactAnt `docs/DERIVE.md`, `docs/DESIGN.md`; PersistAnt `README.md`, `docs/CONSUMERS.md`, `matrix/opendal.tsv`);
re-check before relying on them. Anything not read is marked **UNVERIFIED**. Code is illustrative; names are not final.

## 1. What it is, in one paragraph

A small library for recording *who did what* so that the record can later be checked. You define an event type once, emit it in
one line, and AuditAnt stores it in an append-only trail where every record is chained to the one before. Later, one call
verifies the trail and says whether anything was changed, removed, reordered or inserted. Secrets never reach the trail
because event types are built on RedactAnt: every field must say whether it is kept, fingerprinted or withheld, and forgetting
is a compile error. **Easy is the product**: emit and verify are each a few lines, with no required configuration beyond
"where does it live".

```rust
#[derive(Event)]
#[event(action = "login")]
struct Login { #[redact(keep)] user: String, #[redact(fingerprint)] email: String, token: Secret<String> }

let trail = Trail::open(&config)?;                     // backend chosen through PersistAnt, see section 5
trail.emit(Login { user, email, token })?;
let report = trail.verify()?;                          // report.is_intact(), or the first broken record
```

## 2. Customers and the smallest first slice

| Customer | Wants | Status |
|---|---|---|
| auth-framework | who authenticated, who was refused, token issue/revoke, without any credential in the record | **UNVERIFIED**: the PM's word (HANDOFF); its own needs have not been read. First task of slice 1: read its `origin/main` and replace this row |
| SecretAnt | an audit trail of who did what to which secret; never the secret | **UNVERIFIED**, same |
| RedactAnt | none (AuditAnt is a consumer of it) | `docs/DERIVE.md` section 9 questions answered in section 8 below |

**Smallest first slice (the one thing that must work end to end):** a typed event with the derive, a hash chain over its
canonical bytes, `emit`, `verify`, and an in-memory store used by the tests. No signing, no PersistAnt backend, no multi-writer.
It is enough to show the product (three-line emit, three-line verify, a tamper test that fails) without choosing any backend.
Slice order is section 10.

## 3. Event model

**An event is an immutable record of one action, built from a typed Rust struct** (the answer to DERIVE.md Q1; a dynamic
map-built event is a later slice, fed through the same RedactAnt `Sink`, because auth-framework may need it: UNVERIFIED).

A stored record has an **envelope** AuditAnt fills in and a **body** the author defines:

| Part | Field | Who sets it | Note |
|---|---|---|---|
| envelope | `seq` | trail | 0, 1, 2, ... no gaps; a gap is a verify failure |
| envelope | `time` | trail | UTC, from the caller's clock; **claimed, not proven** (section 6) |
| envelope | `schema` | derive | event type name plus a schema fingerprint (section 8, Q6) |
| envelope | `key_id` | trail | which fingerprint key was in force (section 8, Q3) |
| envelope | `prev` | trail | hash of the previous record; fixed genesis value for `seq` 0 |
| body | `action` | author | a stable code identifier, e.g. `login`; **required** |
| body | `actor` | author | who did it; **required**; a type the author derives, not a bare string |
| body | `target` | author | what it was done to; optional |
| body | `outcome` | author | `Success` / `Failure` / `Denied`, `#[non_exhaustive]`; **required** |
| body | other fields | author | each with a RedactAnt disposition |

`action`, `actor` and `outcome` are required because an audit record without them cannot answer "who did what, and did it
work". The derive refuses an event type missing them (compile-fail test).

**How a redacted field is represented.** The stored record carries the field's *path* and its *disposition*, never its value:
`{ "path": "login.token", "d": "redacted" }`, `{ "path": "login.email", "d": "fp", "v": "fp:3f9a1c7e" }`,
`{ "path": "login.user", "d": "kept", "v": "alice" }`. The field's existence is therefore visible (answer to Q4, section 8).
The placeholder is RedactAnt's `<redacted>` text only when a reader asks for a rendering; on disk it is the `redacted` tag, so
tests assert bytes and a reader cannot confuse a withheld value with the literal string.

**Canonical bytes.** The hashed form is the record serialised deterministically: sorted keys, no insignificant whitespace,
numbers and strings per RFC 8785 (JSON Canonicalization Scheme) **UNVERIFIED as the right choice**: the alternative is
canonical CBOR (RFC 8949 section 4.2). JSON wins on "a developer can read the trail with `cat`"; CBOR on size. Decision for
CireSnave in section 11.

## 4. The trail: append-only and tamper-evident

**Chain.** `hash(n) = H(domain_tag || hash(n-1) || canonical_bytes(n))`, `domain_tag` a fixed string naming the format
version so a hash from another use cannot be replayed here. `H` is SHA-256 or BLAKE3: **UNVERIFIED**, I have not read either
crate's current version, MSRV or audit status; choosing is slice 1's first task, crates.io, newest (CireSnave's dependency rules).
Verify walks the trail, recomputes every hash, and checks `seq` is gapless and `prev` links.

**Signed checkpoints (slice 3, optional).** Every N records, or on demand, the trail writes a checkpoint
`sign(key, seq || hash(seq) || time)`. A checkpoint can be copied somewhere the trail's writer cannot reach (another
machine, a ticket, a printout) and later compared. The signature primitive is Ed25519 from a reviewed crate (**UNVERIFIED**,
same as `H`); the signing key is the caller's, supplied through a trait, never stored by AuditAnt (the same stance as
RedactAnt's fingerprint key).

**What is claimed** (each has a test with a negative control, as in RedactAnt design section 7):

| Id | Claim | Holds when |
|---|---|---|
| T1 | changing any byte of any record is detected | always |
| T2 | deleting or inserting a record in the middle, or reordering two, is detected | always |
| T3 | removing records from the **end** is detected | a checkpoint or head hash recorded outside the trail covers them |
| T4 | rewriting the **whole** trail from some point on is detected | a checkpoint before that point is held where the attacker cannot alter it, **and** the attacker lacks the signing key |
| T5 | no secret declared `redact`/`fingerprint` is in any record | the author's `keep` claims are right (RedactAnt rule 3: `impl Keep` is the audit point) |

**What is NOT claimed** (stated in the README, not discovered by a user):

- **Not prevention.** Someone with write access to the storage can still delete or alter it; AuditAnt makes that *visible*, not impossible.
- **Not truncation-proof alone.** Without an externally held checkpoint, dropping the tail leaves a valid shorter chain (T3).
- **Not authenticity of content.** A false event written honestly into the chain is a valid record. The chain proves "this is what was
  written, in this order", not "this happened".
- **Not time.** `time` is the caller's clock; ordering is `seq`, which is trustworthy only inside an intact chain.
- **Not confidentiality.** Records are stored in clear (redaction aside); encryption at rest is the storage backend's or SecretAnt's job.
- **Not multi-writer consensus.** One logical writer per trail (section 5). Two processes appending to one trail is refused, not merged.
- **Fingerprints are the existing 4-byte keyed tag** (RedactAnt design section 6): identify within a few values, not a security boundary.

## 5. Where it stores

CireSnave's standing order (2026-10-21, quoted in PersistAnt `docs/CONSUMERS.md`): *"I don't want to arbitrarily limit any project
to a specific backend when there are multiple options available."* So AuditAnt **declares what it needs and lets PersistAnt's
registry pick**; it names no backend in code.

Declared needs (capability ids read from `matrix/opendal.tsv` on 2026-11-16; their exact use is **UNVERIFIED** until a slice-2
spike):

| Need | Capability id | Why |
|---|---|---|
| append a record | `blob.append` | the trail is a sequence of appended segments |
| read it back | `blob.read`, `blob.read_range` | verify, and resume after restart |
| list segments | `blob.list` | find the head after a restart |
| refuse overwrite | `blob.create_exclusive` | start a new segment without clobbering one |
| single-writer head | `kv.compare_and_swap` (alternative to a lock file) | prove one writer; **open question**, section 11 |
| durability | `Durability::Persistent` class | a volatile trail is not an audit trail; `Volatile` is accepted **only** in tests |

`blob.append.concurrent_whole` is deliberately **not** needed: one writer. A backend that cannot append (the matrix shows
`opendal-memory` cannot) is refused at open, by name, with PersistAnt's message.

Storage layout: segments of bounded size (`trail-000001`, ...); a sealed segment is never written again; the head hash is
recomputed by verify, so no separate "head" file can disagree with the records. The smallest slice uses an in-memory store
behind a small internal trait; the PersistAnt adapter is slice 2.

## 6. Public API sketch

```rust
// the author's side
pub trait Event: Redact { const ACTION: &'static str; fn actor(&self) -> &Actor; fn outcome(&self) -> Outcome; }
pub struct Trail { /* opaque */ }
impl Trail {
    pub fn open(config: &TrailConfig) -> Result<Trail, Error>;      // backend + fingerprint key come from config
    pub fn emit<E: Event>(&self, event: E) -> Result<Receipt, Error>; // Receipt { seq, hash }
    pub fn verify(&self) -> Result<Report, Error>;                   // Report::is_intact(), first_failure(), len()
    pub fn checkpoint(&self, signer: &dyn Signer) -> Result<Checkpoint, Error>;     // slice 3
    pub fn verify_against(&self, cp: &Checkpoint, v: &dyn Verifier) -> Result<Report, Error>; // slice 3
}
```

Rules that fall out of "easy": one entry type (`Trail`), one error type, `emit` returns only after the record is durable
(no hidden buffering that a crash can lose silently; a buffered mode is a later, explicit opt-in); `verify` returns a report,
never a bare bool, so a failure says *which record* and *which check*; errors say what to do ("the backend `memory` cannot append;
choose one that offers `blob.append`"). A fingerprint key is supplied at open; with none, `fingerprint` fields degrade to
`redacted` (RedactAnt fails closed).

## 7. Architecture declaration (ruling 365)

One crate to start. A `[architecture]` table in `.coderipper.toml` (the format in coderipper#209) will declare components
`event` (policy, stable: envelope, derive glue, canonical bytes), `chain` (policy, stable: hashing, verify), `store` (adapter,
volatile: the PersistAnt adapter and the in-memory test store) and `sign` (adapter, volatile), layered
`event, chain` above `store, sign`, with `volatile.namespaces = ["persistant::"]` so no storage name leaks into the policy
components. **Not in this PR**: the file lands with slice 1, when there are modules to place.

## 8. Answers to RedactAnt `docs/DERIVE.md` section 9

1. **Event shape: typed structs known at compile time.** The derive fits. A dynamic map-fed builder is a later slice over the same `Sink`.
2. **Typed kept values: yes, `Serialize` is needed.** `Kept(&dyn Debug)` loses number-versus-string, and a stored trail must keep types.
   We ask for `Kept` to carry a serialisable view (so `Keep` implies `serde::Serialize` under RedactAnt's `serde` feature).
3. **Fingerprint key: one key per trail, with a `key_id` in every record.** Rotation: a new key gets a new id; old records keep
   theirs, so old and new fingerprints are *not* comparable across ids and the reader is told which id applies. Per-tenant is the
   caller's job (one trail per tenant).
4. **Field existence is not sensitive in v1.** The path appears with a `redacted` tag. No `Omit` disposition; the "no skip" stance
   in DERIVE.md section 3 holds. Revisit only on a named customer whose field *names* are secret.
5. **Integrity: the chain covers the redacted form, marker inside the hashed bytes.** Redaction is deterministic, so the hash is
   reproducible; because the disposition tag and `key_id` are inside the bytes, flipping a field from `redacted` to `kept`
   later breaks the chain.
6. **Schema evolution: yes, a changed disposition is a schema change.** We ask the derive to emit a per-type schema fingerprint
   (type name + field paths + dispositions) so each record's `schema` field lets a reader tell which rules applied.
7. **Default policy: forced decision per field** is the documented style. A struct-level `default = "redact"` is allowed but not recommended
   for audit events, because the point of an audit event is that each field was chosen.
8. **Ownership and async: `describe` runs on the caller's thread and AuditAnt turns the result into owned canonical bytes immediately**,
   so no borrowed value crosses a thread. The borrowed `Kept(&dyn ...)` sketch is enough, provided it is serialisable (Q2).
9. **Other consumers of `Redact`:** auth-framework and SecretAnt reach it only through AuditAnt for now (**UNVERIFIED**). Keep the trait narrow;
   the serde adapter is not needed by AuditAnt.

## 9. Patent-sweep plan

Run per `C:\Projects\PATENT-SWEEP-PROCESS.md` before slice 1 ships code, and recorded in `PATENT-SWEEP.md`. Method: the
`WebSearch` route limited to patents.google.com (US), then the **independent claims of every close hit read verbatim**
(`tools/patent-claims.py`), never judged from a title or abstract. Not legal advice; no attorney. Ideas to sweep:

1. A log in which each record includes a hash of the previous record, verified by recomputation (hash-chained audit logs; Schneier and
   Kelsey's secure audit logs and Certificate Transparency-style append-only logs are expected prior art: cited from the documents,
   **read before relying on them**, not assumed).
2. Periodic signed checkpoints over a log head, held externally to detect truncation.
3. Canonical serialisation before hashing so a record has one byte form.
4. An event whose fields each carry a compile-time redaction disposition that is hashed into the record (the combination of
   RedactAnt's derive with a chain; the likeliest novel-looking idea, so the one most worth a careful search).
5. A verify report naming the first broken record and the check that failed.

Routes to run first: Google Patents queries for ideas 1 to 5 (queries and dates recorded in the sweep file), then the
non-US and assignee routes RedactAnt's sweep skipped, listed as **not run** if skipped. Any hit whose independent claims read
on the design is stopped and sent to the PM before code, as with RedactAnt's detector family.

## 10. Slices (proposed; each its own PR with a fragment; the PM allocates numbers)

1. Event derive consumer, canonical bytes, hash chain, `emit`/`verify`, in-memory store, T1/T2/T5 with negative controls, README
   hello-world compiled in CI, patent sweep, `[architecture]` declaration. 2. PersistAnt adapter, declared needs, durable
   restart, single-writer guard. 3. Signed checkpoints, T3/T4. 4. Dynamic (map-fed) events. 5. Segment sealing and rotation, export.

## 11. Decisions wanted from CireSnave

- **D1: canonical form**: JSON (JCS, readable with `cat`) **recommended**, or CBOR (smaller)?
- **D2: single-writer guard**: a PersistAnt lock/CAS record (needs `kv.compare_and_swap` on top of blob), or "the caller promises one
  writer" and AuditAnt only detects a fork at verify? Recommend: detect-at-verify in slice 2, a guard later.
- **D3: hash**: SHA-256 (ubiquitous, reviewed) or BLAKE3 (faster)? Recommend SHA-256 unless the crate review says otherwise.
- **D4: does he accept slice 1 without a PersistAnt backend** (in-memory only), so no backend choice is made before he has seen the product?

Nothing here is built. Slice 1 starts only after he has seen this.

— lane: auditant
