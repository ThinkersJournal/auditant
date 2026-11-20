// SPDX-License-Identifier: MIT OR Apache-2.0
//! ROADMAP row 10: a typed `#[derive(Redact)]` event reaches the chain as dispositions, never as withheld values.
use auditant::{MemTrail, typed_event, verify};
use redactant::{Context, FingerprintKey, Redact, Secret};

const SECRET: &str = "CANARY-SECRET-1101";
const WITHHELD: &str = "CANARY-WITHHELD-2202";
const FP: &str = "CANARY-FP-3303";
const SHOWN: &str = "shown-role";

#[derive(Redact)]
struct Login {
    #[redact(keep)]
    role: String,
    #[redact(fingerprint)]
    email: String,
    #[redact]
    note: String,
    password: Secret<String>,
}

fn login() -> Login {
    Login {
        role: SHOWN.into(),
        email: FP.into(),
        note: WITHHELD.into(),
        password: Secret::new(SECRET.into()),
    }
}

#[test]
fn canaries_never_reach_the_stored_bytes_and_the_kept_field_does() {
    let key = FingerprintKey::new(&[7u8; 32]).unwrap();
    let cx = Context::with_key(&key).with_key_id("k1");
    let mut t = MemTrail::default();
    t.emit(typed_event("2026-11-20T00:00:00Z", &login(), &cx).unwrap())
        .unwrap();
    assert!(verify(&t.records).is_ok());
    let stored = serde_json::to_string(&t.records).unwrap();
    for c in [SECRET, WITHHELD, FP] {
        assert!(!stored.contains(c), "{c} leaked into the trail");
    }
    // Control: the kept value does appear, so the scan can see a value.
    assert!(stored.contains(SHOWN));
    assert!(stored.contains("\"key_id\":\"k1\""));
    assert!(
        t.records[0].schema.starts_with("Login"),
        "schema was {:?}",
        t.records[0].schema
    );
}

#[test]
fn flipping_a_disposition_after_the_fact_breaks_the_chain() {
    let key = FingerprintKey::new(&[7u8; 32]).unwrap();
    let mut t = MemTrail::default();
    t.emit(typed_event("2026-11-20T00:00:00Z", &login(), &Context::with_key(&key)).unwrap())
        .unwrap();
    t.records[0].body["fields"]["note"] = serde_json::json!({"d": "kept", "v": "x"});
    assert!(verify(&t.records).is_err());
}

#[test]
fn without_a_key_a_fingerprint_field_fails_closed_to_redacted() {
    let ev = typed_event("2026-11-20T00:00:00Z", &login(), &Context::without_key()).unwrap();
    assert_eq!(ev.body["fields"]["email"]["d"], "redacted");
}
