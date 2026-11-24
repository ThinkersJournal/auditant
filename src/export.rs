// SPDX-License-Identifier: MIT OR Apache-2.0

//! Segments and export (ROADMAP row 15). A trail is cut into SEGMENTS: runs of consecutive records.
//! A segment is exported as JSON lines (a header line, then one record per line, so `cat`/`grep`
//! read it) and can be verified on its own, and chained to the segment before it, WITHOUT the rest
//! of the trail. Export never deletes; pruning old segments from the store is not offered (it would
//! weaken the append-only guard and `verify_all`, and needs a ruling).

use crate::chain::{Error, GENESIS, Record, verify_segment};
use serde::{Deserialize, Serialize};

/// Format marker of the header line.
const FORMAT: &str = "auditant-segment/1";

/// First line of an export; every field is re-derived and checked on import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentHeader {
    pub format: String,
    pub first_seq: u64,
    pub last_seq: u64,
    /// The `prev` of the first record (the previous segment's `head`, or [`GENESIS`]).
    pub prev: String,
    /// The hash of the last record.
    pub head: String,
}

/// A verified run of records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub header: SegmentHeader,
    pub records: Vec<Record>,
}

impl Segment {
    /// Cut a segment from records that are already consecutive; checks them as a segment.
    pub fn new(records: Vec<Record>) -> Result<Self, Error> {
        let first = records
            .first()
            .ok_or_else(|| Error::Corrupt("a segment needs at least one record".into()))?;
        let head = verify_segment(&records, first.seq, &first.prev)?.expect("non-empty");
        Ok(Segment {
            header: SegmentHeader {
                format: FORMAT.into(),
                first_seq: first.seq,
                last_seq: records[records.len() - 1].seq,
                prev: first.prev.clone(),
                head,
            },
            records,
        })
    }

    /// True when this segment directly continues `earlier` (its first record links to `earlier`'s
    /// head). Both are already verified, so this is the whole chain check between two segments.
    pub fn follows(&self, earlier: &Segment) -> bool {
        self.header.first_seq == earlier.header.last_seq + 1
            && self.header.prev == earlier.header.head
    }

    /// True when this is the start of a trail (seq 0, genesis `prev`).
    pub fn starts_trail(&self) -> bool {
        self.header.first_seq == 0 && self.header.prev == GENESIS
    }

    /// Write as JSON lines: the header, then each record.
    pub fn to_jsonl(&self) -> Result<String, Error> {
        let mut out = serde_json::to_string(&self.header).map_err(canon)?;
        for r in &self.records {
            out.push('\n');
            out.push_str(&serde_json::to_string(r).map_err(canon)?);
        }
        out.push('\n');
        Ok(out)
    }

    /// Read JSON lines back and verify everything: the header against the records, and the records
    /// as a segment. Any edit to a record, the header or the line count is an error.
    pub fn from_jsonl(text: &str) -> Result<Self, Error> {
        let mut lines = text.lines().filter(|l| !l.is_empty());
        let header: SegmentHeader = serde_json::from_str(
            lines
                .next()
                .ok_or_else(|| Error::Corrupt("empty export".into()))?,
        )
        .map_err(|e| Error::Corrupt(format!("header: {e}")))?;
        if header.format != FORMAT {
            return Err(Error::Corrupt(format!(
                "unknown format {:?}",
                header.format
            )));
        }
        let records = lines
            .map(|l| serde_json::from_str::<Record>(l).map_err(|e| Error::Corrupt(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        let seg = Segment::new(records)?;
        if seg.header != header {
            return Err(Error::Corrupt(
                "header does not match the records it heads".into(),
            ));
        }
        Ok(seg)
    }
}

/// Cut `records` (a whole trail, from seq 0) into segments of `size` records (the last may be
/// shorter). Panics if `size` is 0.
pub fn segments(records: &[Record], size: usize) -> Result<Vec<Segment>, Error> {
    assert!(size > 0, "segment size must be at least 1");
    records
        .chunks(size)
        .map(|c| Segment::new(c.to_vec()))
        .collect()
}

fn canon(e: serde_json::Error) -> Error {
    Error::Canonical(e.to_string())
}
