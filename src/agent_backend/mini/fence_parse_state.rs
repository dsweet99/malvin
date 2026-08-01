//! Mutable state for [`super::fence_parser::parse_bash_fences`].

use super::{BashFence, comment_from_pending, is_bash_fence_open};

pub(crate) struct FenceParseState {
    pub out: Vec<BashFence>,
    pending_comment: Vec<String>,
    inside_fence: bool,
    cmd: String,
    fence_comment: Option<String>,
}

impl FenceParseState {
    pub(crate) const fn new() -> Self {
        Self {
            out: Vec::new(),
            pending_comment: Vec::new(),
            inside_fence: false,
            cmd: String::new(),
            fence_comment: None,
        }
    }

    pub(crate) fn handle_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if self.inside_fence {
            self.handle_inside(line, trimmed);
            return;
        }
        if is_bash_fence_open(trimmed) {
            self.open_fence();
            return;
        }
        if trimmed == "```" {
            self.pending_comment.clear();
            return;
        }
        if !trimmed.is_empty() {
            self.pending_comment.push(trimmed.to_string());
        }
    }

    fn handle_inside(&mut self, line: &str, trimmed: &str) {
        if trimmed == "```" {
            self.close_fence();
            return;
        }
        if !self.cmd.is_empty() {
            self.cmd.push('\n');
        }
        self.cmd.push_str(line);
    }

    fn open_fence(&mut self) {
        self.fence_comment = comment_from_pending(&self.pending_comment);
        self.pending_comment.clear();
        self.inside_fence = true;
        self.cmd.clear();
    }

    fn close_fence(&mut self) {
        if !self.cmd.trim().is_empty() {
            self.out.push(BashFence {
                command: self.cmd.clone(),
                comment: self.fence_comment.clone(),
            });
        }
        self.cmd.clear();
        self.fence_comment = None;
        self.inside_fence = false;
        self.pending_comment.clear();
    }
}
