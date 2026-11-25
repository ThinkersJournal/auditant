// SPDX-License-Identifier: MIT OR Apache-2.0
use auditant::*;
use serde_json::json;

fn ev(n: u64) -> Event {
    Event {
        time: format!("2026-11-24T00:00:{n:02}Z"),
        schema: "t/1".into(),
        key_id: None,
        body: json!({"action":"a","actor":"u","outcome":"ok","n":n}),
    }
}
fn trail(n: u64) -> MemTrail {
    let mut t = MemTrail::default();
    for i in 0..n {
        t.emit(ev(i)).unwrap();
    }
    t
}
fn key(b: u8) -> SigningKey {
    SigningKey::from_bytes(&[b; 32])
}
const T: &str = "2026-11-24T01:00:00Z";

#[test]
fn good_checkpoint_verifies_on_the_intact_and_on_a_longer_trail() {
    let k = key(1);
    let mut t = trail(5);
    let cp = Checkpoint::sign(&t.records[3], T, &k).unwrap();
    assert!(cp.verify_trail(&t.records, &k.verifying_key()).is_ok());
    t.emit(ev(5)).unwrap(); // appending after the checkpoint is normal
    assert!(cp.verify_trail(&t.records, &k.verifying_key()).is_ok());
}

#[test]
fn t3_tail_removal_is_detected_only_with_the_checkpoint() {
    let k = key(1);
    let mut t = trail(6);
    let cp = Checkpoint::sign(&t.records[5], T, &k).unwrap();
    t.records.truncate(3);
    // Negative control: the shortened chain is itself valid, so verify alone cannot see it.
    assert!(verify(&t.records).is_ok());
    assert_eq!(
        cp.verify_trail(&t.records, &k.verifying_key()),
        Err(Error::Truncated {
            checkpoint_seq: 5,
            trail_len: 3
        })
    );
}

#[test]
fn t4_rewrite_from_a_point_is_detected_without_the_key() {
    let k = key(1);
    let t = trail(6);
    let cp = Checkpoint::sign(&t.records[2], T, &k).unwrap();
    // The attacker rebuilds records 2.. with different bodies; the new chain is internally valid.
    let mut forged = MemTrail {
        records: t.records[..2].to_vec(),
    };
    for i in 2..6 {
        let mut e = ev(i);
        e.body = json!({"action":"a","actor":"evil","outcome":"ok","n":i});
        forged.emit(e).unwrap();
    }
    assert!(verify(&forged.records).is_ok()); // control: chain alone is fooled
    assert_eq!(
        cp.verify_trail(&forged.records, &k.verifying_key()),
        Err(Error::CheckpointMismatch { seq: 2 })
    );
}

#[test]
fn t4_holds_only_while_the_attacker_lacks_the_key() {
    // Control: with the key the attacker re-signs a checkpoint over the forged trail and passes.
    let k = key(1);
    let t = trail(4);
    let mut forged = MemTrail::default();
    for i in 0..4 {
        let mut e = ev(i);
        e.body = json!({"evil": i});
        forged.emit(e).unwrap();
    }
    let cp = Checkpoint::sign(&forged.records[3], T, &k).unwrap();
    assert!(cp.verify_trail(&forged.records, &k.verifying_key()).is_ok());
    assert!(verify(&t.records).is_ok());
}

#[test]
fn signature_is_bound_to_key_and_every_field() {
    let (k, other) = (key(1), key(2));
    let t = trail(3);
    let cp = Checkpoint::sign(&t.records[2], T, &k).unwrap();
    assert_eq!(
        cp.check_signature(&other.verifying_key()),
        Err(Error::BadSignature)
    );
    for edit in [
        |c: &mut Checkpoint| c.seq = 1,
        |c: &mut Checkpoint| c.hash = "0".repeat(64),
        |c: &mut Checkpoint| c.time = "2026-11-24T01:00:01Z".into(),
        |c: &mut Checkpoint| c.sig = "zz".into(),
    ] {
        let mut c = cp.clone();
        edit(&mut c);
        assert_eq!(
            c.check_signature(&k.verifying_key()),
            Err(Error::BadSignature)
        );
    }
    assert!(cp.check_signature(&k.verifying_key()).is_ok());
}

#[test]
fn bad_time_is_refused_and_a_broken_chain_is_still_reported() {
    let k = key(1);
    let mut t = trail(3);
    assert_eq!(
        Checkpoint::sign(&t.records[1], "yesterday", &k),
        Err(Error::BadTime { seq: 1 })
    );
    let cp = Checkpoint::sign(&t.records[1], T, &k).unwrap();
    t.records[0].body["outcome"] = json!("denied");
    assert_eq!(
        cp.verify_trail(&t.records, &k.verifying_key()),
        Err(Error::HashMismatch { seq: 0 })
    );
}
