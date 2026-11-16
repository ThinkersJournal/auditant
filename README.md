# auditant

Planned: structured events and an append-only, tamper-evident audit trail, built on RedactAnt so a secret never lands in an event.
Nothing works yet. The crate name `auditant` is reserved on crates.io (published 0.0.0); this repository is at version 0.0.1 with
`publish = false` and a stub library. The plan is `docs/DESIGN.md`; the work list is `ROADMAP.md`.

Two limits, stated up front:

- **A plain-file trail is impossible for now.** The trail's single-writer guard needs compare-and-swap, and PersistAnt's plain-file and
  object-store storage refuses it. The first backend is SQL-backed storage (SQLite) through PersistAnt.
- **No FIPS claim.** The chain hash is BLAKE3, which is not a NIST/FIPS algorithm (and no third-party audit of the crate was found).

Licensed under either of the Apache License, Version 2.0 (`LICENSE-APACHE`) or the MIT licence (`LICENSE-MIT`), at your
option.
