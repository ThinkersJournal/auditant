# Patent sweep: auditant (design only, nothing built) @ 4e84f0a

Date: 2026-11-19; Run by: auditant lane, Sonnet 5.5; Method: AI-assisted web search (`WebSearch`, mode standard, results from
patents.google.com and USPTO print pages, US-only tool), then the independent claims of the closest hits read with
`tools/patent-claims.py` (Google Patents pages, machine-converted text). **NOT legal advice; NOT a freedom-to-operate opinion;
there is no attorney.** Process: `C:\Projects\PATENT-SWEEP-PROCESS.md`. Plan: `docs/DESIGN.md` section 9.

## STOP: send to the PM before slice 1 code (one item, read it first)

**EP2897051A2 (Palantir, "Verifiable audit log"; a published European application, grant status NOT checked) has a claim 1 that
plausibly reads on the AuditAnt chain as designed.** Its US counterpart as filed, **US20150188715A1**, has the identical claim 1.
Verbatim (Google Patents text of the published claim 1):

> "A computer system comprising: one or more hardware processors configured to cause the computer system to: receive an event from
> an event submission entity, the event comprising event data describing the event; determine one or more metadata items associated
> with the event; calculate a log entry hash associated with the event, the log entry hash dependent on the event data; calculate a
> log summary hash associated with the event, the log summary hash dependent on the log entry hash, at least one metadata item of
> the one or more metadata items, and a second log summary hash associated with a second log entry received previous to the event;
> and store a recorded log entry in an electronic event log, wherein the recorded log entry comprises the event data, the at least one
> metadata item of the one or more metadata items, the log entry hash, and the log summary hash, wherein the electronic event log
> comprises a plurality of log entries organized in a list according to time, each recorded log entry comprising a plurality of
> hashes, wherein at least one of the plurality of hashes is dependent on a previous log entry summary hash."

Why it plausibly reads: AuditAnt stores `seq`, `time`, `prev`, body, and a hash over `prev || canonical_bytes(n)` (DESIGN.md
section 3 and 4), i.e. a record holding event data, envelope metadata, and a hash dependent on the previous record's hash.
The one place it may not read: the claim wants **a log entry hash over the event data and a separate summary hash** (a plurality of
hashes per entry); AuditAnt designs one hash per record. Whether that is a real difference is a lawyer's question, not mine.

What lowers the concern, and its limit: (a) this is a published application claim, not an issued one; the **US grant from the same
application, US9338013B2, was narrowed to require salts** (below) and every issued US claim I read in the family needs random or
pseudo-random salt data, which AuditAnt does not use; (b) I did not check whether EP2897051 was granted, or with what claims.
**Decision asked:** the PM (and CireSnave) say whether to accept the US-grant reading, commission a lawyer's look at EP2897051 as
granted, or change the record to avoid a two-level (entry hash plus summary hash) structure. No code until answered.

## EP2897051 status and the design-around (read 2026-11-19; PM ruling 2026-11-19)

**Opened:** Google Patents pages for EP2897051B1/A2 and US9338013B2, US10027473B2, US11032065B2, US11849023B2 (raw B1 HTML for the claims and legal events).
**Tried and failed (HTTP 403):** Espacenet (INPADOC) and the EPO Register. Status below is Google-compiled, not the official register, and not legal advice.

- EP2897051B1 **granted 2017-08-02**; no opposition filed; Google shows Active, anticipated expiry 2034-12-19. Annual fees shown paid for DE, GB, FR, CH (year 12); most other states lapsed. Divisionals EP3255549B1 and EP3511833B1: status not read.
- Claim 1 as granted is the same text as the published claim quoted above (independent claims 1, 8 method, 15 program). Source: https://patents.google.com/patent/EP2897051B1/en
- US family (Google status Active, not confirmed against USPTO fees): US9338013B2 exp. 2034-03-24; US10027473B2 2034-10-18; US11032065B2 2035-01-07; US11849023B2 2034-10-23.

**Design-around (PM ruling: keep ONE hash per entry).** Claim 1 requires each recorded entry to comprise *a plurality of hashes*: a log entry hash over the event data AND a log summary hash that depends on the previous entry's summary hash. AuditAnt stores one `hash` per record, computed over `prev` and the record's canonical bytes (`src/lib.rs`, `compute_hash`); there is no separate entry hash and no second-level summary hash anywhere in the format. **Plainly: our single hash does depend on the previous record's hash, so the difference is only the second hash level. Whether that avoids the claim is a lawyer's question; there is no attorney. Not legal advice.** No slice may add a second hash level to the record format without going back to the PM.
## Ideas swept (DESIGN.md section 9)
1. A log in which each record includes a hash of the previous record, verified by recomputation.
2. Periodic signed checkpoints over a log head, held externally to detect truncation.
3. Canonical serialisation (RFC 8785 JCS) before hashing, so a record has one byte form.
4. An event whose fields each carry a compile-time redaction disposition (kept / fp / redacted), hashed into the record, chained
   (RedactAnt's derive combined with the chain). The most care went here.
5. A verify report naming the first broken record and the check that failed.

## Queries (WebSearch, mode standard, `allowed_domains: patents.google.com`; date 2026-11-19)
| # | Idea | Query |
|---|---|---|
| Q1 | 1 | `patent hash chain audit log each record includes hash of previous record tamper-evident verify` |
| Q2 | 2 | `patent periodic signed checkpoint of log head stored externally detect truncation of append-only log` |
| Q3 | 3 | `patent canonical JSON serialization before hashing audit record single byte representation` |
| Q4 | 4 | `patent audit event fields annotated with redaction disposition at compile time, redacted form hashed into tamper-evident log record` |
| Q5 | 5 | `patent verification report identifies first corrupted record in hash-chained log and which check failed` |
| Q6 | 4 | `patent structured log event schema field-level attributes mark sensitive fields masked hashed before storing, tamper-evident audit trail integrity` |
| Q7 | 4 | `patent redaction policy per field of data record determined by data type annotation, redacted record hash chained for audit, redaction decision included in integrity hash` |
| Q8 | 4 | `patent source code annotation compile time generates code to mask personally identifiable fields in log statements, macro derive attribute` |
| Q9 | 2 | `patent signed tree head checkpoint transparency log append-only consistency proof detect rollback truncation` |
| Q10 | 5 | `patent audit log verification outputs index of first record failing hash check, tamper location, error report identifying entry where chain broken` |

**Positive controls:** Q1, Q4, Q5, Q6, Q7 and Q10 all returned the Palantir "Verifiable redactable audit log" family, whose claims
the tool then printed, so the queries reach real audit-log patents. A null does not mean none exists: Q2, Q3 and Q8 surfaced nothing
close on their exact combination (no patent found for compile-time derive plus redaction; no JCS-specific patent), and Q5's
specific "names the first broken record" report returned only general recompute-and-compare disclosures. Several result summaries
quoted patent text I did not verify; I relied only on the claims printed by the tool.

## Hits: independent claims read verbatim with the tool
Status and expiry are Google Patents' fields, not the official record. "Reads on design?" compares with: hash-chained append-only
trail, JCS canonical JSON, per-field dispositions hashed into the record, signed checkpoints, verify report naming first break.

| # | patent | title | assignee | status | independent claims read | key limitation (quoted) | reads on design? |
|---|---|---|---|---|---|---|---|
| 1 | EP2897051A2 | Verifiable audit log | Palantir | "Active" per Google; **application text, grant not checked** | claim 1 | see STOP block above | **plausibly yes (two-level hashes arguable); STOP** |
| 2 | US20150188715A1 | Verifiable redactable audit log (application) | Palantir | application; superseded by US9338013B2; Google shows "Active" | 1, 8, 14 | claim 1 as in hit 1; claims 8 and 14 add "a salt comprising pseudo-random data" | claim 1 as filed: same as hit 1; claims 8, 14 no (salt) |
| 3 | US9338013B2 | Verifiable redactable audit log | Palantir | Active; est. 2034-10-18 | 1, 8, 14 | "determine a first salt comprising first pseudo-random data; combine the event data and the first salt"; "a second salt ... determined independently from the first salt"; claim 8 adds "commit ... based on a verifiable digital signature" and "redact one or more log entries in the segment" | no: no salts; no export-with-redaction |
| 4 | US10027473B2 | same family | Palantir | Active; est. 2034-10-18 | 1, 6, 13 | "first random or pseudo-random data"; "previously signed with a first ephemeral key"; "a digital signature with a second ephemeral key" | no: no random data, no ephemeral keys |
| 5 | US20160254906A1 | same family | Palantir | Active per Google (application) | 1, 6, 13 | "determining first pseudo-random data"; "signing the first log entry with a first ephemeral key" | no |
| 6 | US11849023B2 | same family | Palantir | Active; est. 2034-10-23 | 1, 9 | "a salt comprising random or pseudo-random data"; "redact at least a portion of one or more log entries in the segment, wherein each redacted log entry that comprises duplicate log entry data ... comprise distinct log hashes"; "provide the selected segment ... to a computing device" | no (watch: redacting an exported segment) |
| 7 | US11032065B2 | same family | Palantir | Active; est. 2035-01-07 | 1, 8, 15 | per-object logs: a third entry's hash is computed "based at least in part on the first log summary hash, the second log summary hash, and the third log data" (each entry hashes ALL earlier summary hashes) | no: AuditAnt links to `prev` only; one trail, not per object (watch: do not hash in earlier-than-previous summaries) |
| 8 | US5978475A | Event auditing system (Schneier/Kelsey line) | Counterpane, now BT | Expired - Lifetime | 1, 23, 25, 43, 46, 53, 55, 61 | "an entry-specific cryptographic key having been derived by computing a cryptographic hash on at least a cryptographic key for a preceding entry"; claims 46, 53, 55, 61 add an encrypted datum and "permission mask" | no, and expired: AuditAnt has no per-entry evolving key |
| 9 | US10075425B1 | Verifiable log service | Amazon | Active; est. 2037-02-21 | 1, 5, 12 | a logging service receiving "a batch of log entries from a client", batch hash from "a prior-log-batch hash value", returned with a range of identifiers; client confirms integrity by matching | no: no batch service, no client-held batch hash |
| 10 | US20100115284A1 | Support of tamper detection for a log of records | IBM | Expired - Fee Related | 1, 11, 20 | signatures over subsets of records giving "an overlapping chain", "at least one designated carryover audit record that overlaps into a next subset ... associated with at least two signatures", via a "tamper evidence adding proxy" | no (watch idea 2: slice 3 checkpoints must not overlap with carryover records) |
| 11 | US11593317B2 | Tamper-resistant activity logging | Palantir | Active; est. 2038-09-18 | 1, 12 | logs to "a first datastore and in a second datastore"; a second hash "according to a second algorithm" after a deletion or restoration; "perform a lockdown, the lockdown comprising building a firewall" | no |
| 12 | US20140245020A1 | Verification ... for Lower-Entropy Input Records | Guardtime | Abandoned | 1, 19 | blinding mask hash tree over record blocks to a root hash | no (hash tree, not chain) |
| 13 | US12368576B2 | Peer-to-peer key verification | Apple | Active; est. 2044-01-17 | 1, 15 | a device "receiving ... a second signed log head of an identifier log" from another device, then a consistency check with a key transparency server | no: key-transparency gossip between devices; our checkpoint is a held head, no consistency-proof exchange (watch idea 2) |
| 14 | US8291230B2 | Signing JSON messages | IBM | Active; est. 2031-07-07 | 1, 10, 16, 23 | per-object "reference element" with digest, "a signature information element ... and a canonical form of one or more reference elements" | no: an XML-DSig-style JSON signature structure; we hash one JCS record (idea 3) |
| 15 | US9853956B2 | JSON encryption and hashing with indication added to key-value | Texas Instruments | Active; est. 2035-10-19 | 1 | generator circuitry that "insert[s] the hash value in the key in addition to the key" | no (idea 3) |
| 16 | US11860863B1 | Data redaction in a journal-based database | Amazon | Active; est. 2042-06-30 | 1, 5, 15 | "receive a request to redact first data from a version of an item in the journal"; "replace the version of the item ... with a hash value ... retaining the corresponding hash value in the metadata of the block" | no: later redaction of a stored hash-chained journal on request. AuditAnt redacts at write time and never rewrites (watch: slice 5 export or any after-the-fact redaction) |
| 17 | US12640933B1 | Operational permit-receipt gate | Gate Of Remembrance LLC | Active | 1, 22 | non-bypassable interceptor, "deterministically canonicalize the external-effect request ... action digest", signed permit receipts | no (idea 3 only; AI execution gating) |
| 18 | US12676749B1 | Pre-inference execution compliance for AI systems | Lamb Within Lion LLC | Active | 1, 16 | "Root-Origin Hash Signature", "Immutable Constraint Chain", quorum-signed verdict | no |
| 19 | US12657264B1 | Activation-keyed dual-channel watermarking (VIC-WM) | Gate Of Remembrance LLC | Active | 1, 13, 21 | sampled model activations, error-correcting digest, session key derivation | no |
| 20 | US20130262871A1 | Tamper proof camera logs | Ricoh | Active; est. 2028-11-11 | 1, 7 | verifying media files from log entries extracted from a known media file | no (idea 5 hit; unrelated) |

Idea 4 result: **no patent found that combines a per-field redaction disposition chosen at compile time (a derive) with hashing the
resulting form into a chained record.** The closest are the Palantir family (redaction of an exported log segment using salted
hashes) and Amazon US11860863B1 (later replacement of an item with its hash). Both redact AFTER the chain is built, by removing
or replacing data while the chain still verifies; AuditAnt records the disposition and the already-redacted form INSIDE the hashed
bytes at write time. That is a difference in claim language, not a clearance, and the search was one route (see Not run).
Idea 5 result: no patent found claiming a report that names the first failing record plus the failed check; the Palantir family
discloses that a recomputed hash differing from the stored one shows tampering, but I saw no claim to the report.

## Seen in results, NOT read (no verdict, `unread`)
US20170149793A1, US9582680B2, US20170177904A1, US20220164474A1, US7805673B2, US10091212B2, US8447989, US8027927, US12430317,
US12383320, US11403605, US20140157407A1, WO2020104935A1, EP1759258A1 and US7770032B2 (both titled "Secure logging for
irrefutable administration"; the tool printed US7770032B2 claim 1, which is about fake information for unauthorised auditors, not a
hash chain; the EP was not read). Non-patent results (arXiv papers) were not read in full and are not relied on.

## Old foundational prior art
Schneier and Kelsey's secure audit logs, Certificate Transparency and RFC 6962: **UNREAD**. I did not open any of those documents
in this sweep; the only Schneier/Kelsey material read is US5978475A's claims (hit 8), a patent, not the paper. Nothing here is
cited from them.

## Design constraints that come out of this sweep (so a later change does not walk into a claim)
- Keep ONE hash per record over `prev` and the canonical bytes. Do not add a separate entry hash plus a summary hash (hit 1).
- No random salt per record, no ephemeral signing keys that are discarded on a schedule (hits 3 to 6).
- Link to the previous record only; do not hash all earlier summaries into a record (hit 7); one trail, not a chain per object.
- No redaction of an already-written or exported record (slice 5 export, hits 6, 16): needs a re-sweep first.
- Checkpoints (slice 3): no overlapping signed subsets with carryover records (hit 10); no device-to-device signed-head gossip
  with a consistency-check server (hit 13). Signing a head and storing it elsewhere is the design.
- No evolving per-entry key (hit 8, expired but the family has ongoing relatives not searched).

## Not run
USPTO full-text and Patent Public Search, Espacenet, WIPO PATENTSCOPE, Google Patents' own query endpoint (my tool is US-only and
cannot drive these); non-US grants other than what the search incidentally surfaced (EP2897051A2, EP1759258A1, CN/WO hits
unread); **EP2897051 grant status and granted claims**; assignee searches (Palantir, Amazon QLDB, Google Trillian and Certificate
Transparency, Guardtime, Codenotary/immudb, Microsoft Azure confidential ledger, Splunk, Datadog, HashiCorp); the `extended`
search mode (mode standard returned enough, but the combination for idea 4 has only one family of near hits); Rust-specific
prior-art (crates) as non-patent art.

## Needs review (to the PM)
Hit 1 (EP2897051A2 and its US filing US20150188715A1): see STOP at the top.

## Limitations
- AI-assisted; ten queries, claims of 22 documents read via a machine-converted source. "No" in the last column means no overlap
  I could find on the independent claims I read; it is not a clearance. A tool-printed claim block can run several claims together;
  I read only each independent claim's opening and its early limitations, not every dependent claim.
- Statuses and expiries are Google Patents' fields. EP and application statuses especially need the official record.
- Re-sweep when a new family of capability is added (see constraints) and at least yearly after release.

— lane: auditant
