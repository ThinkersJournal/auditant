# HANDOFF (lane auditant)

State 2026-11-12. New repo, nothing built. Board 423 ruled A (CireSnave 2026-11-12): start AuditAnt, **design first: nothing is built before CireSnave has seen the design.** Run on Sonnet only.

## What AuditAnt is
Structured events and an append-only, tamper-evident audit trail, built to use RedactAnt (`ThinkersJournal/redactant`, redaction types and safe Display/Debug) so a secret never lands in an event. Name claimed on crates.io (`auditant`). Consumers that want it: auth-framework and SecretAnt (an audit trail of who did what).

## First task (the only one until the design is reviewed)
Write `docs/DESIGN.md` and open it as a PR (one `changelog.d/<topic>.md` fragment, no version edit; the PM stamps at merge; see `tools/fragment-stamp-kit/README-changelog.d.md` in the portfolio checkout). It states plainly: the event model (what an event is, required fields, how a redacted field is represented), the trail (append-only storage, how tamper evidence works: hash chain or signed checkpoints, and what is claimed and not claimed), where it stores (declare storage needs and pick via PersistAnt `choose`; never hard-wire one backend), the public API sketch, what RedactAnt must give it (RedactAnt's own DERIVE.md section 9 holds the questions only this lane can answer: answer them), and a patent-sweep plan (any append-only/hash-chain prior art is read verbatim first, not from titles).
Also in the design: who the first consumers are and what they need, and the smallest first slice.

## Rules for this repo from day one
CI checks and branch protection that requires them (portfolio rule, ask the PM to set them once the first PR exists); a project architecture declaration (`[architecture]`, ruling 365); licence files are copied from RedactAnt; every PR body and comment ends `— lane: auditant`. Easy is the product: a developer should be able to emit and verify in a few lines.

## Not yours
Merging, versions, publishing (PM). Rulings (CireSnave via the PM board).
