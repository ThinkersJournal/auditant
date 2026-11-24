// SPDX-License-Identifier: MIT OR Apache-2.0
use auditant::*;
use serde_json::json;

fn trail(n: u64) -> MemTrail {
    let mut t = MemTrail::default();
    for i in 0..n {
        t.emit(Event {
            time: format!("2026-11-24T00:00:{i:02}Z"),
            schema: "t/1".into(),
            key_id: None,
            body: json!({"action":"a","actor":"u","outcome":"ok","n":i}),
        })
        .unwrap();
    }
    t
}

#[test]
fn segments_verify_alone_and_chain_to_each_other() {
    let t = trail(7);
    let segs = segments(&t.records, 3).unwrap();
    assert_eq!(segs.len(), 3);
    assert!(segs[0].starts_trail() && !segs[1].starts_trail());
    assert!(segs[1].follows(&segs[0]) && segs[2].follows(&segs[1]));
    assert!(!segs[2].follows(&segs[0])); // control: a gap does not chain
    assert!(!segs[0].follows(&segs[1]));
    assert_eq!(
        segs[2].header.head,
        verify(&t.records).unwrap().unwrap(),
        "last segment head is the trail head"
    );
}

#[test]
fn jsonl_round_trip_and_every_edit_is_refused() {
    let t = trail(5);
    let seg = Segment::new(t.records[2..].to_vec()).unwrap();
    let text = seg.to_jsonl().unwrap();
    assert_eq!(text.lines().count(), 4); // header + 3 records
    assert_eq!(Segment::from_jsonl(&text).unwrap(), seg);

    // edit a record body
    assert!(Segment::from_jsonl(&text.replace("\"n\":3", "\"n\":9")).is_err());
    // drop a middle record
    let mut lines: Vec<&str> = text.lines().collect();
    lines.remove(2);
    assert!(Segment::from_jsonl(&lines.join("\n")).is_err());
    // tamper with the header's head
    let bad = text.replacen(&seg.header.head, &"0".repeat(64), 1);
    assert!(Segment::from_jsonl(&bad).is_err());
    // wrong format marker
    assert!(Segment::from_jsonl(&text.replace("auditant-segment/1", "x/1")).is_err());
    assert!(Segment::from_jsonl("").is_err());
}

#[test]
fn verify_segment_checks_start_seq_and_prev() {
    let t = trail(4);
    let tail = &t.records[2..];
    assert!(verify_segment(tail, 2, &t.records[1].hash).is_ok());
    assert!(verify_segment(tail, 2, GENESIS).is_err());
    assert!(verify_segment(tail, 1, &t.records[1].hash).is_err());
}

#[test]
fn store_exports_a_range() {
    let dir = tempfile::tempdir().unwrap();
    let s = SqliteTrail::open(&dir.path().join("t.db")).unwrap();
    for i in 0..5 {
        s.emit(Event {
            time: format!("2026-11-24T00:00:{i:02}Z"),
            schema: "t/1".into(),
            key_id: None,
            body: json!({"n": i}),
        })
        .unwrap();
    }
    let seg = s.export_segment(1, 100).unwrap(); // clamped at the head
    assert_eq!((seg.header.first_seq, seg.header.last_seq), (1, 4));
    assert!(Segment::from_jsonl(&seg.to_jsonl().unwrap()).is_ok());
    assert!(s.export_segment(9, 12).is_err()); // nothing there
}
