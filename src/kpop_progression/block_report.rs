pub(crate) struct KpopBlockProgressCtx {
    pub steps_needed: usize,
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
             hypotheses: {before} → {after} (`## Step N — KPOP …` lines; needed ≥1 more this block, block quota {needed})",
            before = self.hypotheses_before,
            after = self.hypotheses_after,
            needed = self.ctx.steps_needed,
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
                "\nThese tool errors may be unrelated to experiment-log progress; \
                 review them for context.",
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
}
