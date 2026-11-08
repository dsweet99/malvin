use std::sync::{Mutex, MutexGuard, PoisonError};

use super::stream::{MdPiece, MdTableStream};

struct Pending {
    who: String,
    width: usize,
    stream: MdTableStream,
}

static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

fn pending_slot() -> MutexGuard<'static, Option<Pending>> {
    PENDING.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) struct ReadyLine {
    pub who: String,
    pub piece: MdPiece,
}

pub(crate) fn push_stdout_markdown(who: &str, line: &str, width: usize) -> Vec<ReadyLine> {
    let mut slot = pending_slot();
    let mut out = switch_who(&mut slot, who);
    let pending = slot.get_or_insert_with(|| Pending {
        who: who.to_string(),
        width,
        stream: MdTableStream::default(),
    });
    pending.width = width;
    let pieces = pending.stream.push(line, width);
    drop(slot);
    out.extend(ready_lines(who, pieces));
    out
}

pub(crate) fn flush_stdout_markdown() -> Vec<ReadyLine> {
    let mut slot = pending_slot();
    let Some(pending) = slot.as_mut() else {
        return Vec::new();
    };
    let who = pending.who.clone();
    let pieces = pending.stream.flush(pending.width);
    drop(slot);
    ready_lines(&who, pieces)
}

pub(crate) fn end_stdout_markdown_message() -> Vec<ReadyLine> {
    let mut slot = pending_slot();
    let Some(pending) = slot.as_mut() else {
        return Vec::new();
    };
    let who = pending.who.clone();
    let pieces = pending.stream.end_message(pending.width);
    drop(slot);
    ready_lines(&who, pieces)
}

fn switch_who(slot: &mut Option<Pending>, who: &str) -> Vec<ReadyLine> {
    let Some(pending) = slot.as_mut() else {
        return Vec::new();
    };
    if pending.who == who {
        return Vec::new();
    }
    let previous = pending.who.clone();
    let pieces = pending.stream.flush(pending.width);
    *slot = None;
    ready_lines(&previous, pieces)
}

fn ready_lines(who: &str, pieces: Vec<MdPiece>) -> Vec<ReadyLine> {
    pieces
        .into_iter()
        .map(|piece| ReadyLine {
            who: who.to_string(),
            piece,
        })
        .collect()
}
