//! Turn classification for the inner bash-fence loop.

use crate::agent_backend::mini::fence_parser::{
    has_mini_done_outside_bash_fences, parse_bash_fences, scan_fence_warnings, FenceParseWarning,
};
use crate::agent_backend::mini::terminal::MiniTerminalReason;
use super::loop_inner_types::TurnAction;
use super::loop_types::LoopDriverConfig;

pub(crate) fn classify_turn(
    assistant_text: &str,
    config: &LoopDriverConfig,
    had_bash_this_prompt: bool,
) -> (TurnAction, Vec<FenceParseWarning>) {
    let mut warnings = scan_fence_warnings(assistant_text);
    if has_mini_done_outside_bash_fences(assistant_text) {
        return (
            TurnAction::Done(MiniTerminalReason::MiniDoneOutsideFence),
            warnings,
        );
    }
    let fences = parse_bash_fences(assistant_text);
    if fences.is_empty() {
        if had_bash_this_prompt {
            warnings.push(FenceParseWarning::FencelessAfterBashOnlyTurn);
        }
        let reason = if config.expects_investigation && had_bash_this_prompt
            || warnings
                .iter()
                .any(|w| matches!(w, FenceParseWarning::UnclosedFence | FenceParseWarning::FencelessAfterBashOnlyTurn))
        {
            MiniTerminalReason::FencelessPremature
        } else {
            MiniTerminalReason::FencelessComplete
        };
        return (TurnAction::Done(reason), warnings);
    }
    (TurnAction::RunBash(fences), warnings)
}
