//! Stage New request for the inner bash-fence loop.

use super::loop_types::{LoopDriverConfig, LoopDriverSession};

pub(crate) fn stage_user_prompt(
    session: &mut LoopDriverSession,
    _config: &LoopDriverConfig,
    user_prompt: &str,
) {
    session.pending_new_request = Some(user_prompt.to_string());
    session.section_shape_nudged = false;
}
