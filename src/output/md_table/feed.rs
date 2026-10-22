use std::cell::RefCell;

use super::stream::{MdPiece, MdTableStream};

struct Pending {
    who: String,
    width: usize,
    stream: MdTableStream,
}

thread_local! {
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

pub(crate) struct ReadyLine {
    pub who: String,
    pub piece: MdPiece,
}

pub(crate) fn push_stdout_markdown(who: &str, line: &str, width: usize) -> Vec<ReadyLine> {
    PENDING.with(|slot| {
        let mut slot = slot.borrow_mut();
        let mut out = switch_who(&mut slot, who);
        if slot.is_none() {
            *slot = Some(Pending {
                who: who.to_string(),
                width,
                stream: MdTableStream::default(),
            });
        }
        let pending = slot.as_mut().expect("pending markdown table");
        pending.width = width;
        let pieces = pending.stream.push(line, width);
        out.extend(ready_lines(who, pieces));
        out
    })
}

pub(crate) fn flush_stdout_markdown() -> Vec<ReadyLine> {
    PENDING.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(pending) = slot.as_mut() else {
            return Vec::new();
        };
        let who = pending.who.clone();
        let pieces = pending.stream.flush(pending.width);
        ready_lines(&who, pieces)
    })
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
