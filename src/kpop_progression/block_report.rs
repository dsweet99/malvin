const KPOP_CATCHUP_CAP: u32 = 0;

pub(crate) struct KpopBlockProgressCtx {
    pub steps_needed: usize,
    pub attempts_so_far: u32,
}

pub(crate) struct KpopBlockMissSnapshot {
    pub exp_log_path: std::path::PathBuf,
    pub hypotheses_before: usize,
    pub hypotheses_after: usize,
    pub ctx: KpopBlockProgressCtx,
    pub tool_health_lines: Vec<String>,
    pub agent_streamed_kpop_solved: bool,
}

impl KpopBlockMissSnapshot {
    #[must_use]
    pub fn format_no_progress_error(&self) -> String {
        let exp = self.exp_log_path.display();
        let mut out = format!(
            "KPOP block prompt made no progress on the experiment log.\n\
             exp_log: {exp}\n\
             hypotheses: {before} → {after} (`## Step N — KPOP …` lines; needed ≥1 more this block, block quota {needed})\n\
             prompt attempt: {attempt} of {max_attempts} (initial + up to {KPOP_CATCHUP_CAP} catch-ups)",
            before = self.hypotheses_before,
            after = self.hypotheses_after,
            needed = self.ctx.steps_needed,
            attempt = self.ctx.attempts_so_far + 1,
            max_attempts = KPOP_CATCHUP_CAP + 1,
        );
        if self.tool_health_lines.is_empty() {
            out.push_str(
                "\nThe agent may have reported results in chat without appending \
                 `## Step N — KPOP …` sections to the experiment log.",
            );
        } else {
            out.push_str("\nACP tool issues during this prompt:");
            for line in &self.tool_health_lines {
                out.push('\n');
                out.push_str(line);
            }
            out.push_str(
                "\nLikely cause: Cursor agent tools could not read/write workspace files. \
                 Retry when file/shell tools are working.",
            );
        }
        if self.agent_streamed_kpop_solved {
            out.push_str(
                "\nNote: agent streamed `## KPOP_SOLVED` in chat, but malvin requires that marker \
                 in the experiment log file (not `[<kpop]` stdout).",
            );
        }
        out
    }

    #[must_use]
    #[cfg(test)]
    pub fn format_catchup_exhausted_error(&self) -> String {
        let exp = self.exp_log_path.display();
        let mut out = format!(
            "KPOP block incomplete after the initial attempt and {KPOP_CATCHUP_CAP} catch-up attempts.\n\
             exp_log: {exp}\n\
             hypotheses: {} total (`## Step N — KPOP …` lines)\n\
             last block still needed: {} step(s)",
            self.hypotheses_after,
            self.ctx.steps_needed,
        );
        if !self.tool_health_lines.is_empty() {
            out.push_str("\nLast prompt ACP tool issues:");
            for line in &self.tool_health_lines {
                out.push('\n');
                out.push_str(line);
            }
        }
        if self.agent_streamed_kpop_solved {
            out.push_str(
                "\nAgent streamed `## KPOP_SOLVED` in chat during the last prompt, but the experiment \
                 log gate was not satisfied.",
            );
        }
        out
    }
}
