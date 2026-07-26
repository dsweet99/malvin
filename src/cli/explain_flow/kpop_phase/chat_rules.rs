//! Chat-body rules for explain Review and Plan `KPop` turns.

pub(crate) const EXPLAIN_PHASE_REVIEW: &str = "review";
pub(crate) const EXPLAIN_PHASE_PLAN: &str = "plan";

pub(crate) const REVIEW_CHAT_RULES: &str = "\
Judge lack-of-satisfaction. Do not edit. The entire agent chat body must be exactly `LGTM` \
(and only LGTM) when nothing fails, or else a failure-focused gap list—never a gap list \
followed by LGTM. Missing/empty products ⇒ never LGTM. Hard-fail probes (any one fails the \
review): (1) cold entry (first sentence of the abstract or of any section opens on a \
definition, mechanism, notation, or toy before situating the working setting and naming the \
concrete obstacle, bound, or open question that forces the next move; a warm earlier stretch \
does not license a cold later opening, even if it promised that toy); (2) unpaid debt between \
adjacent paragraphs and between sections (each non-final stretch ends on a claim the next \
opening takes as subject or premise—same referent—and continues or answers; fail if that \
opening ignores it and starts a new locally complete topic); (3) settle-and-stop (adjacent \
stretches reorderable or independently deletable without breaking a load-bearing claim; apply \
an independence self-check); (4) topic-adjacent join (a colon/semicolon clause that is \
on-topic but does not continue or cash the same move as the clause before the punctuation—if \
deleting the trailing clause leaves no hanging obligation, fail; ordinary claim-chain next \
sentences are not this failure); (5) review metalanguage (prose, section titles, captions, or \
abstracts name review checks or use review-lattice surface words as continuity scaffolding, \
including “pressure”, “settle”, “debt”, “through-line”, “landscape”, “co-reason”, \
close paraphrases such as “budget pressure” / “what remains to settle” / “three co-reasons”, \
or labeled checks such as “cold entry”, “unpaid debt”, “settle-and-stop”, or “the debt is \
paid”); (6) broken figures (text overlap, clipped nodes/arrows, or unreadable labels). Once \
these hard checks pass, return `LGTM`—do not fail for residual synonymy, mild hedges, \
optional polish, or other authoring goals already listed in explain_constraints (Work owns \
those).
";

pub(crate) const PLAN_CHAT_RULES: &str = "\
Put the plan only in the agent chat body. Do not edit files. Do not echo an executive summary \
or tl;dr to chat.
";

pub(crate) fn explain_kpop_chat_rules(phase: &str) -> &'static str {
    if phase == EXPLAIN_PHASE_PLAN {
        PLAN_CHAT_RULES
    } else {
        REVIEW_CHAT_RULES
    }
}
