// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed events: a `#[derive(Redact)]` value becomes an [`Event`] whose body holds each field's
//! DISPOSITION, never a withheld value (ROADMAP row 10, DESIGN section 8).
//!
//! Body shape: `{"fields": {"<path>": {"d": "redacted"} | {"d": "fp", "v": "<tag>"} | {"d": "kept", "v": <json>}}}`;
//! the context's `key_id` goes in the record envelope (hashed), not the body. The disposition tag is inside the hashed
//! bytes, so flipping a field from `redacted` to `kept` later breaks the chain. The event's `schema`
//! is the derive's per-type schema string (type name, field paths, dispositions), so a reader can
//! tell which rules applied to a record.

use crate::chain::{Error, Event};
use redactant::{Context, Disposition, Redact, Segment, Sink};
use serde_json::{Map, Value, json};

struct Collect {
    fields: Map<String, Value>,
    error: Option<Error>,
}

fn path_string(path: &[Segment<'_>]) -> String {
    path.iter()
        .map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join("/")
}

impl Sink for Collect {
    fn field(&mut self, path: &[Segment<'_>], d: Disposition<'_>) {
        let entry = match d {
            Disposition::Redacted => json!({"d": "redacted"}),
            Disposition::Fingerprint(f) => json!({"d": "fp", "v": f.to_string()}),
            Disposition::Kept(k) => match serde_json::to_value(k as &dyn erased_serde::Serialize) {
                Ok(v) => json!({"d": "kept", "v": v}),
                Err(e) => {
                    self.error.get_or_insert(Error::Canonical(e.to_string()));
                    return;
                }
            },
            // Disposition is non-exhaustive: an unknown one fails closed.
            _ => json!({"d": "redacted"}),
        };
        self.fields.insert(path_string(path), entry);
    }
}

/// Build an [`Event`] from a typed value. `time` is the caller's timestamp.
pub fn typed_event<T: Redact>(
    time: impl Into<String>,
    value: &T,
    cx: &Context<'_>,
) -> Result<Event, Error> {
    let mut sink = Collect {
        fields: Map::new(),
        error: None,
    };
    value.describe(cx, &mut sink);
    if let Some(e) = sink.error {
        return Err(e);
    }
    let mut body = Map::new();
    body.insert("fields".into(), Value::Object(sink.fields));
    Ok(Event {
        time: time.into(),
        schema: T::schema(),
        key_id: cx.key_id().map(String::from),
        body: Value::Object(body),
    })
}
