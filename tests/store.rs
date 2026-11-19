// SPDX-License-Identifier: MIT OR Apache-2.0
// PersistAnt 0.18.23's capability matrix has certification rows for `os=linux` only; on Windows and macOS
// the registry refuses `sqlite` ("unverified (no row)"), so these run on Linux (CI ubuntu) until PersistAnt
// certifies other platforms (ASK sent 2026-11-19; ROADMAP known gap).
#![cfg(target_os = "linux")]
use auditant::*;
use serde_json::json;

fn ev(n: u64) -> Event {
    Event { time: format!("2026-11-19T00:00:{n:02}Z"), schema: "t/1".into(), body: json!({"action":"a","actor":"u","outcome":"ok","n":n}) }
}

#[test]
fn emit_verify_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trail.db");
    {
        let t = SqliteTrail::open(&path).unwrap();
        for i in 0..4 { t.emit(ev(i)).unwrap(); }
        assert_eq!(t.head().unwrap().unwrap().seq, 3);
        assert!(t.verify_all().unwrap().is_some());
    }
    let t = SqliteTrail::open(&path).unwrap();
    t.emit(ev(4)).unwrap();
    assert_eq!(t.head().unwrap().unwrap().seq, 4);
    t.verify_all().unwrap();
}

#[test]
fn two_writers_from_the_same_head_exactly_one_wins() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trail.db");
    let a = SqliteTrail::open(&path).unwrap();
    a.emit(ev(0)).unwrap();
    let head = a.head().unwrap();
    let b = SqliteTrail::open(&path).unwrap();
    let ra = a.emit_after(head.clone(), ev(1));
    let rb = b.emit_after(head, ev(2));
    assert!(ra.is_ok());
    assert_eq!(rb, Err(Error::Conflict { seq: 1 }));
    assert_eq!(a.verify_all().unwrap(), b.verify_all().unwrap());
    assert_eq!(a.head().unwrap().unwrap().seq, 1);
}

#[test]
fn negative_control_a_plain_put_would_silently_overwrite() {
    // Shows the guard is what stops the loss: a raw `put` (no put_if_absent) replaces record 1,
    // and the trail then fails verification, so the two-writer test above is not vacuous.
    let dir = tempfile::tempdir().unwrap();
    let t = SqliteTrail::open(&dir.path().join("t.db")).unwrap();
    t.emit(ev(0)).unwrap();
    let r1 = t.emit(ev(1)).unwrap();
    let mut forged = r1.clone();
    forged.body = json!({"action":"x","actor":"evil","outcome":"ok"});
    t.raw().put(&format!("rec/{:020}", 1), serde_json::to_vec(&forged).unwrap(), None).unwrap();
    assert_eq!(t.verify_all(), Err(Error::HashMismatch { seq: 1 }));
}

#[test]
fn crash_orphan_is_adopted_on_open() {
    // Simulate a crash after the record write and before the head advance.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.db");
    let t = SqliteTrail::open(&path).unwrap();
    let r0 = t.emit(ev(0)).unwrap();
    let orphan = next_record(Some(&r0), ev(1)).unwrap();
    assert!(t.raw().put_if_absent(&format!("rec/{:020}", 1), serde_json::to_vec(&orphan).unwrap(), None).unwrap());
    assert_eq!(t.head().unwrap().unwrap().seq, 0); // head did not move
    drop(t);
    let t = SqliteTrail::open(&path).unwrap();
    assert_eq!(t.head().unwrap().unwrap().seq, 1); // adopted
    t.verify_all().unwrap();
    t.emit(ev(2)).unwrap();
    t.verify_all().unwrap();
}

#[test]
fn a_forged_orphan_is_refused_on_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.db");
    let t = SqliteTrail::open(&path).unwrap();
    let r0 = t.emit(ev(0)).unwrap();
    let mut bad = next_record(Some(&r0), ev(1)).unwrap();
    bad.body = json!({"tampered": true}); // hash no longer matches
    t.raw().put_if_absent(&format!("rec/{:020}", 1), serde_json::to_vec(&bad).unwrap(), None).unwrap();
    drop(t);
    assert!(SqliteTrail::open(&path).is_err());
}
