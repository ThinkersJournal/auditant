# auditant

Structured events and an append-only, tamper-evident (hash-chained) audit trail. **Pre-release, unpublished:** slice 1 is a one-hash chain over
RFC 8785 canonical JSON (BLAKE3), an in-memory trail, and a SQLite-backed trail through PersistAnt (Linux only for now). `time` must be UTC RFC 3339 (`...Z`). The record body is an untyped
JSON value; the RedactAnt derive is not built, so nothing here stops a secret landing in a body yet. The crate name `auditant` is reserved on
crates.io (published 0.0.0); this repository has `publish = false`. The plan is `docs/DESIGN.md`; the work list is `ROADMAP.md`.

```rust
use auditant::{verify, Event, MemTrail};
use serde_json::json;

let mut trail = MemTrail::default();
trail.emit(Event {
    time: "2026-11-19T00:00:00Z".into(),
    schema: "login/1".into(),
    key_id: None,
    body: json!({"actor": "u1", "outcome": "ok"}),
}).unwrap();

assert!(verify(&trail.records).is_ok());

// Editing a stored record is detected at that record.
trail.records[0].body["outcome"] = json!("denied");
assert!(verify(&trail.records).is_err());
```

Two limits, stated up front:

- **A plain-file trail is impossible for now.** The trail's single-writer guard needs compare-and-swap, and PersistAnt's plain-file and
  object-store storage refuses it. The first backend is SQL-backed storage (SQLite) through PersistAnt.
- **No FIPS claim.** The chain hash is BLAKE3, which is not a NIST/FIPS algorithm (and no third-party audit of the crate was found).

Licensed under either of the Apache License, Version 2.0 (`LICENSE-APACHE`) or the MIT licence (`LICENSE-MIT`), at your
option.
