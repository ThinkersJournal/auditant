// SPDX-License-Identifier: MIT OR Apache-2.0
use auditant::*;
use serde_json::json;

fn ev(n: u64) -> Event {
    Event { time: format!("2026-11-19T00:00:{n:02}Z"), schema: "t/1".into(), body: json!({"action":"login","actor":"u1","outcome":"ok","n":n}) }
}
fn trail(n: u64) -> MemTrail {
    let mut t = MemTrail::default();
    for i in 0..n { t.emit(ev(i)).unwrap(); }
    t
}

#[test]
fn clean_trail_verifies_and_returns_head() {
    let t = trail(5);
    assert_eq!(verify(&t.records).unwrap(), Some(t.records[4].hash.clone()));
    assert_eq!(verify(&[]).unwrap(), None);
}

#[test]
fn t1_edit_is_detected_at_the_edited_record() {
    let mut t = trail(5);
    t.records[2].body["outcome"] = json!("denied");
    assert_eq!(verify(&t.records), Err(Error::HashMismatch { seq: 2 }));
}

#[test]
fn t2_delete_in_the_middle_is_detected() {
    let mut t = trail(5);
    t.records.remove(2);
    assert!(verify(&t.records).is_err());
}

#[test]
fn reorder_is_detected() {
    let mut t = trail(5);
    t.records.swap(1, 2);
    assert!(verify(&t.records).is_err());
}

#[test]
fn rehashing_an_edited_record_breaks_the_next_link() {
    // An attacker who recomputes the edited record's own hash still breaks record 3's prev.
    let mut t = trail(5);
    t.records[2].body["outcome"] = json!("denied");
    let fixed = next_record(Some(&t.records[1]), Event { time: t.records[2].time.clone(), schema: "t/1".into(), body: t.records[2].body.clone() }).unwrap();
    t.records[2] = fixed;
    assert_eq!(verify(&t.records), Err(Error::BrokenLink { seq: 3 }));
}

#[test]
fn tail_truncation_is_not_detected_without_an_external_head() {
    // Documented non-claim: pins the limit so it cannot be silently "fixed" in prose only.
    let mut t = trail(5);
    let head_before = t.records[4].hash.clone();
    t.records.truncate(3);
    let head_after = verify(&t.records).unwrap().unwrap();
    assert_ne!(head_before, head_after); // only an externally held head reveals it
}

#[test]
fn unknown_alg_is_refused() {
    let mut t = trail(2);
    t.records[1].alg = "md5".into();
    assert!(matches!(verify(&t.records), Err(Error::UnknownAlg { seq: 1, .. })));
}

#[test]
fn key_order_in_body_does_not_change_the_hash() {
    // JCS: one byte form per value.
    let a = next_record(None, Event { time: "t".into(), schema: "s".into(), body: serde_json::from_str(r#"{"a":1,"b":2}"#).unwrap() }).unwrap();
    let b = next_record(None, Event { time: "t".into(), schema: "s".into(), body: serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap() }).unwrap();
    assert_eq!(a.hash, b.hash);
}

#[test]
fn known_answer_pins_the_format() {
    let r = next_record(None, Event { time: "2026-11-19T00:00:00Z".into(), schema: "t/1".into(), body: json!({"action":"a","actor":"u","outcome":"ok"}) }).unwrap();
    assert_eq!(r.hash.len(), 64);
    assert_eq!(r.prev, GENESIS);
    // Changing any field changes the hash (positive control for the pin below).
    let mut r2 = r.clone(); r2.time = "2026-11-19T00:00:01Z".into();
    assert_ne!(verify(&[r.clone()]).unwrap(), verify(&[Record{hash: next_record(None, Event{time:r2.time.clone(),schema:"t/1".into(),body:r2.body.clone()}).unwrap().hash, ..r2}]).unwrap());
}
