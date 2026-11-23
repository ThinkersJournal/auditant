// SPDX-License-Identifier: MIT OR Apache-2.0
//! The store tests against server backends (ROADMAP row 12). Each backend reads its URL from an
//! environment variable; CI sets them (service containers) and `AUDITANT_REQUIRE_BACKENDS=1`, which
//! turns a missing URL into a failure. Without it a missing URL is printed and the check passes, so
//! a laptop without a server can run `cargo test --all-features`.
#![cfg(any(feature = "postgres", feature = "mysql"))]
use auditant::*;
use persistant::registry::Endpoint;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

fn ev(n: u64) -> Event {
    Event {
        time: format!("2026-11-23T00:00:{n:02}Z"),
        schema: "t/1".into(),
        key_id: None,
        body: json!({"action":"a","actor":"u","outcome":"ok","n":n}),
    }
}

/// A namespace no earlier run used (the servers outlive the tests).
fn fresh_ns(tag: &str) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!(
        "{tag}-{t}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    )
}

fn run(id: &str, env: &str) {
    if std::env::var_os(env).is_none() {
        assert!(
            std::env::var_os("AUDITANT_REQUIRE_BACKENDS").is_none(),
            "{env} is not set but AUDITANT_REQUIRE_BACKENDS is: the {id} backend did not run"
        );
        eprintln!("{id}: {env} not set, backend NOT exercised");
        return;
    }
    let ep = Endpoint::url_env(env);
    let ns = fresh_ns(id);
    let open = || SqliteTrail::open_backend(id, &ep, &ns).unwrap();

    // emit, verify, reopen
    let a = open();
    for i in 0..4 {
        a.emit(ev(i)).unwrap();
    }
    assert_eq!(a.head().unwrap().unwrap().seq, 3);
    assert!(a.verify_all().unwrap().is_some());
    drop(a);
    let a = open();
    a.emit(ev(4)).unwrap();
    assert_eq!(a.head().unwrap().unwrap().seq, 4);
    a.verify_all().unwrap();

    // two writers from the same head: exactly one wins
    let head = a.head().unwrap();
    let b = open();
    let ra = a.emit_after(head.clone(), ev(5));
    let rb = b.emit_after(head, ev(6));
    assert!(ra.is_ok());
    assert_eq!(rb, Err(Error::Conflict { seq: 5 }));
    assert_eq!(a.verify_all().unwrap(), b.verify_all().unwrap());

    // negative control: a plain put replaces a record and verify notices
    let mut forged = a.emit(ev(7)).unwrap();
    forged.body = json!({"action":"x","actor":"evil","outcome":"ok"});
    a.raw()
        .put(
            &format!("{ns}/rec/{:020}", forged.seq),
            serde_json::to_vec(&forged).unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(a.verify_all(), Err(Error::HashMismatch { seq: forged.seq }));

    // another namespace in the same database is a separate, empty trail
    let other = SqliteTrail::open_backend(id, &ep, &fresh_ns(id)).unwrap();
    assert!(other.head().unwrap().is_none());
}

#[cfg(feature = "postgres")]
#[test]
fn postgres_trail() {
    run("postgres", "AUDITANT_PG_URL");
}

#[cfg(feature = "mysql")]
#[test]
fn mysql_trail() {
    run("mysql", "AUDITANT_MYSQL_URL");
}
